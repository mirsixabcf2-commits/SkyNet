//! Delimiter detection, missing value imputation, Min-Max normalization, and E.164 phone formatting.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImputeStrategy {
    Mean,
    Median,
    Zero,
    DropRow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanCsvOutput {
    pub headers: Vec<String>,
    pub data: Vec<Vec<f32>>,
    pub rows: usize,
    pub cols: usize,
}

/// Detects the most probable delimiter from a CSV sample string.
pub fn detect_delimiter(sample: &str) -> char {
    let candidates = [',', ';', '\t', '|'];
    let mut scores = [0usize; 4];

    for line in sample.lines().take(5) {
        for (i, &cand) in candidates.iter().enumerate() {
            scores[i] += line.chars().filter(|&c| c == cand).count();
        }
    }

    let mut max_idx = 0;
    let mut max_score = scores[0];
    for (i, &score) in scores.iter().enumerate().skip(1) {
        if score > max_score {
            max_score = score;
            max_idx = i;
        }
    }

    if max_score > 0 { candidates[max_idx] } else { ',' }
}

/// Cleans raw CSV text, imputes missing values, and optionally normalizes numerical columns to [0.0, 1.0].
pub fn clean_csv(
    raw: &str,
    impute: ImputeStrategy,
    normalize_minmax: bool,
) -> Result<CleanCsvOutput, String> {
    let delim = detect_delimiter(raw);
    let mut lines = raw.lines().filter(|l| !l.trim().is_empty());
    let header_line = lines.next().ok_or_else(|| "Empty CSV input".to_string())?;

    let headers: Vec<String> = header_line
        .split(delim)
        .map(|h| h.trim().trim_matches('"').to_string())
        .collect();
    let num_cols = headers.len();
    if num_cols == 0 {
        return Err("CSV has no columns".to_string());
    }

    let mut parsed_matrix: Vec<Vec<Option<f32>>> = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split(delim).collect();
        let mut row = Vec::with_capacity(num_cols);
        for col_idx in 0..num_cols {
            let val_opt = parts.get(col_idx).and_then(|s| {
                let clean = s.trim().trim_matches('"');
                if clean.is_empty() || clean.eq_ignore_ascii_case("null") || clean == "NaN" {
                    None
                } else {
                    clean.parse::<f32>().ok()
                }
            });
            row.push(val_opt);
        }
        parsed_matrix.push(row);
    }

    let mut filled_data = impute_matrix(&parsed_matrix, num_cols, impute);
    if normalize_minmax {
        apply_minmax_scaling(&mut filled_data, num_cols);
    }

    let row_count = filled_data.len();
    Ok(CleanCsvOutput {
        headers,
        data: filled_data,
        rows: row_count,
        cols: num_cols,
    })
}

fn impute_matrix(matrix: &[Vec<Option<f32>>], num_cols: usize, strategy: ImputeStrategy) -> Vec<Vec<f32>> {
    if strategy == ImputeStrategy::DropRow {
        return matrix
            .iter()
            .filter(|row| row.iter().all(|val| val.is_some()))
            .map(|row| row.iter().map(|v| v.unwrap_or(0.0)).collect())
            .collect();
    }

    let col_stats: Vec<f32> = (0..num_cols)
        .map(|c| compute_col_statistic(matrix, c, strategy))
        .collect();

    matrix
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(c, opt)| opt.unwrap_or(col_stats[c]))
                .collect()
        })
        .collect()
}

fn compute_col_statistic(matrix: &[Vec<Option<f32>>], col_idx: usize, strategy: ImputeStrategy) -> f32 {
    let mut values: Vec<f32> = matrix
        .iter()
        .filter_map(|r| r.get(col_idx).copied().flatten())
        .collect();

    if values.is_empty() || strategy == ImputeStrategy::Zero {
        return 0.0;
    }

    if strategy == ImputeStrategy::Mean {
        let sum: f32 = values.iter().sum();
        return sum / values.len() as f32;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if (values.len() & 1) == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn apply_minmax_scaling(data: &mut [Vec<f32>], num_cols: usize) {
    if data.is_empty() {
        return;
    }
    for c in 0..num_cols {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for row in data.iter() {
            if row[c] < min { min = row[c]; }
            if row[c] > max { max = row[c]; }
        }
        let range = max - min;
        if range > 1e-6 {
            for row in data.iter_mut() {
                row[c] = (row[c] - min) / range;
            }
        } else {
            for row in data.iter_mut() {
                row[c] = 0.0;
            }
        }
    }
}

/// Formats Vietnamese phone numbers into the standardized international E.164 format (+84...).
pub fn standardize_e164_vietnam(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.starts_with("84") && (digits.len() == 11 || digits.len() == 12) {
        return Some(format!("+{digits}"));
    }
    if digits.starts_with('0') && (digits.len() == 10 || digits.len() == 11) {
        let rest = &digits[1..];
        return Some(format!("+84{rest}"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_delimiter() {
        let sample = "id;name;price;score\n1;itemA;10.5;99\n2;itemB;20.0;88";
        assert_eq!(detect_delimiter(sample), ';');
    }

    #[test]
    fn test_clean_csv_minmax() {
        let csv = "feature1,feature2\n10,100\n20,200\n30,300";
        let out = clean_csv(csv, ImputeStrategy::Mean, true).unwrap();
        assert_eq!(out.rows, 3);
        assert_eq!(out.cols, 2);
        assert_eq!(out.data[0], vec![0.0, 0.0]);
        assert_eq!(out.data[1], vec![0.5, 0.5]);
        assert_eq!(out.data[2], vec![1.0, 1.0]);
    }

    #[test]
    fn test_standardize_e164_vietnam() {
        assert_eq!(standardize_e164_vietnam("0912.345.678"), Some("+84912345678".to_string()));
        assert_eq!(standardize_e164_vietnam("84912345678"), Some("+84912345678".to_string()));
        assert_eq!(standardize_e164_vietnam("123"), None);
    }
}

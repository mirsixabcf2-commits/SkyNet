//! `ws_rfc6455`: Pure-Rust RFC 6455 WebSocket upgrade handshake (FIPS 180-4 SHA-1 +
//! RFC 4648 Base64, `0` external crates) and `16ms` (`~60fps`) generic `&str` text streamer.

use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub const WS_FRAME_INTERVAL_MS: u64 = 16;
const RFC6455_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const B64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Computes the RFC 6455 `Sec-WebSocket-Accept` digest for a client `Sec-WebSocket-Key`.
pub fn compute_ws_accept_key(client_key: &str) -> String {
    let mut combined = String::with_capacity(client_key.trim().len() + RFC6455_GUID.len());
    combined.push_str(client_key.trim());
    combined.push_str(RFC6455_GUID);
    base64_encode(&sha1_digest(combined.as_bytes()))
}

/// Extracts `Sec-WebSocket-Key` when `Upgrade: websocket` is present in `raw_http_req`.
pub fn extract_ws_upgrade_key(raw_http_req: &str) -> Option<String> {
    let mut is_ws = false;
    let mut ws_key: Option<&str> = None;
    for line in raw_http_req.lines().skip(1) {
        let Some((k, v)) = line.split_once(':') else { continue };
        if k.trim().eq_ignore_ascii_case("upgrade") && v.trim().eq_ignore_ascii_case("websocket") {
            is_ws = true;
        } else if k.trim().eq_ignore_ascii_case("sec-websocket-key") && !v.trim().is_empty() {
            ws_key = Some(v.trim());
        }
    }
    ws_key.filter(|_| is_ws).map(str::to_owned)
}

/// Builds the `HTTP/1.1 101 Switching Protocols` WebSocket handshake response.
pub fn build_ws_upgrade_response(client_key: &str) -> String {
    let accept = compute_ws_accept_key(client_key);
    format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n"
    )
}

/// Encodes an unmasked server-to-client RFC 6455 UTF-8 text frame (`FIN=1`, `opcode=0x1`).
pub fn encode_ws_text_frame(payload: &str) -> Vec<u8> {
    let bytes = payload.as_bytes();
    let len = bytes.len();
    let mut frame = Vec::with_capacity(len + 10);
    frame.push(0x81);
    if len <= 125 {
        frame.push(len as u8);
    } else if len <= u16::MAX as usize {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }
    frame.extend_from_slice(bytes);
    frame
}

/// Decodes one unmasked server WebSocket frame, returning `(opcode, utf8_payload, bytes_used)`.
pub fn decode_ws_server_frame(buf: &[u8]) -> Option<(u8, String, usize)> {
    if buf.len() < 2 { return None; }
    let opcode = buf[0] & 0x0F;
    let len_code = (buf[1] & 0x7F) as usize;
    let (hdr_len, payload_len) = match len_code {
        0..=125 => (2usize, len_code),
        126 if buf.len() >= 4 => (4, u16::from_be_bytes([buf[2], buf[3]]) as usize),
        127 if buf.len() >= 10 => {
            let mut b = [0u8; 8];
            b.copy_from_slice(&buf[2..10]);
            (10, u64::from_be_bytes(b) as usize)
        }
        _ => return None,
    };
    let end = hdr_len.checked_add(payload_len)?;
    if buf.len() < end { return None; }
    let text = String::from_utf8_lossy(&buf[hdr_len..end]).into_owned();
    Some((opcode, text, end))
}

/// Upgrades `stream` via RFC 6455 and pushes JSON text frames every `16ms` (`~60fps`).
pub async fn stream_ws_telemetry_60fps<F>(
    mut stream: TcpStream,
    client_key: &str,
    render_json: F,
) -> Result<(), std::io::Error>
where
    F: Fn() -> String,
{
    stream.write_all(build_ws_upgrade_response(client_key).as_bytes()).await?;
    stream.flush().await?;
    let mut ticker = tokio::time::interval(Duration::from_millis(WS_FRAME_INTERVAL_MS));
    let mut rx_buf = [0u8; 256];
    loop {
        ticker.tick().await;
        match stream.try_read(&mut rx_buf) {
            Ok(0) => return Ok(()),
            Ok(_n) if (rx_buf[0] & 0x0F) == 0x8 => return Ok(()),
            Ok(n) if n > 0 && (rx_buf[0] & 0x0F) == 0x9 => {
                stream.write_all(&[0x8A, 0x00]).await?;
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }
        let frame = encode_ws_text_frame(&render_json());
        stream.write_all(&frame).await?;
    }
}

/// Pure-Rust FIPS 180-4 SHA-1 message digest (`20 bytes`).
pub fn sha1_digest(data: &[u8]) -> [u8; 20] {
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity((data.len() + 9).div_ceil(64) * 64);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0x00); }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = [0x6745_2301u32, 0xEFCD_AB89, 0x98BA_DCFE, 0x1032_5476, 0xC3D2_E1F0];
    for off in (0..msg.len()).step_by(64) {
        sha1_compress(&mut state, &msg[off..off + 64]);
    }
    let mut out = [0u8; 20];
    for (i, word) in state.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn sha1_compress(s: &mut [u32; 5], chunk: &[u8]) {
    let mut w = [0u32; 80];
    for i in 0..16 {
        w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
    }
    for i in 16..80 {
        w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
    }
    let [mut a, mut b, mut c, mut d, mut e] = *s;
    for (i, &wi) in w.iter().enumerate() {
        let (f, k) = match i {
            0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999u32),
            20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
            40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
            _ => (b ^ c ^ d, 0xCA62_C1D6),
        };
        let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(wi);
        e = d; d = c; c = b.rotate_left(30); b = a; a = temp;
    }
    for (dst, add) in s.iter_mut().zip([a, b, c, d, e]) { *dst = dst.wrapping_add(add); }
}

/// Pure-Rust RFC 4648 standard Base64 encoder.
pub fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(*chunk.get(1).unwrap_or(&0));
        let b2 = u32::from(*chunk.get(2).unwrap_or(&0));
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_TABLE[((n >> 18) & 63) as usize] as char);
        out.push(B64_TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { B64_TABLE[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64_TABLE[(n & 63) as usize] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc6455_accept_key_matches_canonical_vector() {
        assert_eq!(compute_ws_accept_key("dGhlIHNhbXBsZSBub25jZQ=="), "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
        let req = "GET /ws/telemetry HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n";
        assert_eq!(extract_ws_upgrade_key(req).as_deref(), Some("dGhlIHNhbXBsZSBub25jZQ=="));
        assert!(build_ws_upgrade_response("dGhlIHNhbXBsZSBub25jZQ==").contains("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));
    }

    #[test]
    fn text_frame_codec_roundtrips_short_and_extended_json() {
        let short = r#"{"tick":1,"ok":true}"#;
        let (op1, dec1, used1) = decode_ws_server_frame(&encode_ws_text_frame(short)).expect("short frame");
        assert_eq!((op1, dec1.as_str(), used1), (0x1, short, short.len() + 2));

        let extended = format!("{{\"data\":\"{}\"}}", "x".repeat(400));
        let raw_ext = encode_ws_text_frame(&extended);
        let (op2, dec2, used2) = decode_ws_server_frame(&raw_ext).expect("extended 16-bit frame");
        assert_eq!((op2, dec2, used2), (0x1, extended, raw_ext.len()));
    }
}

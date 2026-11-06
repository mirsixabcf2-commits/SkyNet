# Technical Implementation Plan: Hệ Thống Tiền Xử Lý Tự Hành, Presets Danh Mục & Bộ Lệnh CLI Thực Chiến

## 1. Kiến Trúc & Thiết Kế Tổng Thể (System Architecture)

Tính năng này mở rộng `crates/skynet-node` và `crates/skynet-dashboard` để cung cấp bộ giải pháp thực tế hoàn chỉnh từ tầng tiền xử lý dữ liệu, đóng gói preset, thực thi dòng lệnh CLI đến giao diện trực quan:

```
[Người Dùng / AI Agent / Dashboard]
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│                 Turnkey Preprocessor Engine                 │
│  - text_pii.rs        : PII Redaction, Sliding Chunking    │
│  - tabular_csv.rs     : Auto-Delimiter, Impute, MinMax/Z    │
│  - crypto_envelope.rs : Symmetric Stream Cipher Session     │
│  - mod.rs             : Pipeline orchestrator, Tensor/Fount │
└──────────────────────────────┬──────────────────────────────┘
                               │
               ┌───────────────┴───────────────┐
               ▼                               ▼
┌─────────────────────────────┐ ┌─────────────────────────────┐
│    Turnkey Presets Catalog  │ │    CLI Real-World Tooling   │
│  (presets.rs)               │ │  - bin/skynet.rs (Client)   │
│  - text-audit               │ │  - bin/skynet_agent.rs (Wrk)│
│  - csv-clean-normalize      │ │  - main.rs (Node Daemon)    │
│  - image-batch-upscale      │ └──────────────┬──────────────┘
│  - log-anomaly-detection    │                │
└──────────────┬──────────────┘                │
               │                               ▼
               └──────────────────► [SkyNet Wire & Cell Mesh]
```

### Cấu Trúc File & Module Kế Hoạch
```
crates/skynet-node/
├── Cargo.toml                      # Bổ sung [[bin]] skynet và [[bin]] skynet-agent
└── src/
    ├── lib.rs                      # Re-export preprocessor, presets, storage, identity, runtime
    ├── preprocessor/
    │   ├── mod.rs                  # Preprocessor orchestrator, PipelineOptions, PreprocessResult
    │   ├── text_pii.rs             # PII Redaction regex (Phone, Email, CCCD, Cards, Keys), Chunking
    │   ├── tabular_csv.rs          # CSV delimiter detect, missing value impute, MinMax/Z-score, E.164
    │   └── crypto_envelope.rs      # Session key generator, ChaCha symmetric envelope encrypt/decrypt
    ├── presets.rs                  # TurnkeyPreset catalog, registry & factory
    ├── bin/
    │   ├── skynet.rs               # User Client CLI (wallet, airdrop, opcode, compute presets/dispatch)
    │   └── skynet_agent.rs         # Worker Daemon CLI (init-worker, auto-bid, hardware guard)
    └── main.rs                     # Giữ nguyên skynet-node daemon
```

Toàn bộ các file mới đều được thiết kế dưới 250 dòng (tối đa 350 dòng), mỗi hàm dưới 40 dòng, số lượng tham số $\le 3\text{--}4$ theo chuẩn Constitution.

---

## 2. Thiết Kế Chi Tiết Từng Phân Hệ

### 2.1. Động Cơ Tiền Xử Lý (`crates/skynet-node/src/preprocessor/`)
- **`text_pii.rs`**:
  - `redact_pii(text: &str) -> String`: Dùng regex mẫu chuẩn hóa che giấu SĐT (`[REDACTED_PHONE]`), Email (`[REDACTED_EMAIL]`), CCCD 9-12 số (`[REDACTED_ID]`), Credit Card (`[REDACTED_CARD]`), Secret Keys (`[REDACTED_KEY]`).
  - `sliding_window_chunks(text: &str, window_size: usize, overlap: usize) -> Vec<String>`: Cắt lát văn bản bảo toàn ngữ cảnh.
- **`tabular_csv.rs`**:
  - `detect_delimiter(sample: &str) -> char`: Phân tích tần suất `,`, `;`, `\t`, `|`.
  - `ImputeStrategy`: `Mean`, `Median`, `Zero`, `DropRow`.
  - `clean_csv(raw: &str, impute: ImputeStrategy, normalize_minmax: bool) -> Result<CleanCsvResult, PreprocessError>`:
    - Làm sạch hàng tiêu đề, chuẩn hóa số thực khuyết thiếu.
    - Đưa các cột số về thang đo $[0.0, 1.0]$.
  - `standardize_e164_vietnam(phone: &str) -> Option<String>`: Chuyển đổi số nội địa (`09...`, `03...`) sang quốc tế (`+84...`).
- **`crypto_envelope.rs`**:
  - `encrypt_envelope(payload: &[u8], key_32: &[u8; 32]) -> Vec<u8>`: Mã hóa dòng đối xứng bảo vệ dữ liệu trước khi gửi lên mạng phân tán.
  - `decrypt_envelope(encrypted: &[u8], key_32: &[u8; 32]) -> Result<Vec<u8>, PreprocessError>`.
- **`mod.rs`**:
  - `PreprocessPipeline`: Chạy tuần tự các bước cấu hình.
  - `to_candle_tensor_input(data: &[f32], dim_in: usize, dim_hidden: usize, dim_out: usize) -> CandleTaskInput`.

### 2.2. Sổ Bộ Cấu Hình Sẵn (`crates/skynet-node/src/presets.rs`)
- Struct `TurnkeyPreset`:
  - `id: &'static str`
  - `title: &'static str`
  - `description: &'static str`
  - `default_opcode: u16`
  - `suggested_bounty_skn: u64`
  - `category: &'static str`
- Các Presets mặc định:
  1. `text-audit` (`0x0215` - Private Document Audit & PII Scrubbing)
  2. `csv-clean-normalize` (`0x0210` - Tabular ML & Predictive Normalization)
  3. `image-batch-upscale` (`0x0200` - Real-ESRGAN Super-Resolution Batch)
  4. `log-anomaly-detection` (`0x0220` - Distributed Log Anomaly Detection)
  5. `phone-standardize` (`0x0230` - E.164 CRM Data Cleansing)
- Hàm tra cứu: `list_all_presets() -> &'static [TurnkeyPreset]`, `find_preset(id: &str) -> Option<&'static TurnkeyPreset>`.

### 2.3. Nhị Phân CLI Người Dùng & AI Dispatcher (`skynet.rs`)
- Hỗ trợ các subcommands:
  - `wallet init [--path <path>]`: Sinh và lưu khóa Ed25519 cục bộ.
  - `wallet balance [--wallet <path>]`: Kiểm tra số dư On-chain và BloodPool.
  - `airdrop claim [--wallet <path>]`: Nhận airdrop tiếp thị theo HWID.
  - `opcode list [--domain <filter>]`: Liệt kê các opcode chuẩn hóa.
  - `opcode search <keyword>`: Tìm kiếm opcode theo từ khóa.
  - `opcode info <hex>`: Xem chi tiết yêu cầu VRAM/RAM và giá sàn của opcode.
  - `compute presets`: Liệt kê các preset tiền xử lý sẵn sàng dùng.
  - `compute dispatch --input <path> --preset <name> [--opcode <hex>] [--bounty <skn>] [--encrypt]`:
    - Đọc file đầu vào.
    - Thực thi tiền xử lý tương ứng với preset.
    - Hiển thị kết quả làm sạch, số tensor tạo thành và dispatch task.

### 2.4. Nhị Phân CLI Thợ Đào & Worker Tự Động (`skynet_agent.rs`)
- Hỗ trợ cờ khởi tạo và vận hành:
  - `--init-worker [--wallet <path>]`: Nhận diện phần cứng (CPU, RAM, Core count) và ghi cấu hình thợ đào.
  - `--listen <port>`: Lắng nghe UDP (mặc định: `8889`).
  - `--auto-bid`: Bật chế độ tự động bắt task và đấu thầu tính toán.
  - `--tags <list>`: Nhãn năng lực (VD: `cuda,upscale,llm,blender`).
  - `--min-bounty-rate <rate>`: Giá sàn nhận việc.
  - `--max-vram-gb <gb>`: Ngưỡng VRAM an toàn.
  - `--battery-guard`: Tự động dừng khi chạy pin.
  - `--temp-limit <celsius>`: Ngắt khi quá nhiệt.
  - `--schedule <range>`: Giới hạn khung giờ nhàn rỗi.

### 2.5. Tích Hợp Dashboard Preprocessing Studio (`crates/skynet-dashboard`)
- Thêm thẻ UI `Turnkey Preprocessing Studio` trong `assets/dashboard.html`:
  - Bộ chọn danh sách Preset.
  - Checkboxes cho các bộ lọc (PII, MinMax, E.164, Encrypt, Sliding Chunking).
  - Vùng so sánh 2 cột trực quan (Raw Input vs Processed Result).
  - Bảng thống kê tài nguyên (Kích thước byte, Tokens, Tensors, Fountain Droplets).
  - Nút "Xem Trước Tiền Xử Lý", "Phát Lệnh Tức Thì", và "Sao Chép Lệnh CLI Cho Agent".
- Bổ sung HTTP handler `/api/presets` và `/api/preprocess` trong `crates/skynet-dashboard/src/main.rs`.

---

## 3. Rà Soát Tuân Thủ Constitution (Constitution Checklist)

| Yêu Cầu | Giải Pháp Thiết Kế | Trạng Thái |
|---|---|---|
| **File $\le 250$ dòng (cứng 350)** | Chia module tiền xử lý thành `text_pii.rs`, `tabular_csv.rs`, `crypto_envelope.rs`, `mod.rs`. Tách CLI thành `bin/skynet.rs` và `bin/skynet_agent.rs`. | Đạt |
| **Hàm $\le 40$ dòng** | Tách nhỏ logic parse CLI, từng bước lọc PII regex, từng bước chuẩn hóa CSV thành các hàm thuần túy (pure functions). | Đạt |
| **Tham số $\le 3\text{--}4$** | Gom nhóm các cờ vào Struct DTO (`PreprocessOptions`, `WorkerCliConfig`, `DispatchCliConfig`). | Đạt |
| **Nesting $\le 2\text{--}3$** | Áp dụng triệt để Guard Clauses (`let ... else { return ... }`, `?`). | Đạt |
| **Cấm File utils rác** | Đặt tên module theo đúng miền nghiệp vụ (`text_pii.rs`, `tabular_csv.rs`, `crypto_envelope.rs`). | Đạt |
| **Xử lý lỗi tường minh** | Sử dụng `Result<T, PreprocessError>` với `thiserror`, không dùng `.unwrap()`. | Đạt |

---

## 4. Kế Hoạch Kiểm Thử & Kiểm Chứng (Testing Strategy)

1. **Unit Tests Tiền Xử Lý (`preprocessor` tests):**
   - Test che giấu PII (SĐT, Email, CCCD, Thẻ tín dụng, API Key).
   - Test cắt lát sliding window văn bản bảo toàn độ gối đầu.
   - Test tự động nhận diện delimiter CSV và điền giá trị khuyết thiếu.
   - Test chuẩn hóa số học MinMax $[0.0, 1.0]$.
   - Test chuẩn hóa SĐT Việt Nam sang E.164.
   - Test mã hóa/giải mã phong bì ChaCha đối xứng.
2. **Integration Tests CLI (`skynet` & `skynet-agent`):**
   - Test `skynet wallet init` và `wallet balance` khôi phục chính xác.
   - Test `skynet opcode list`, `search`, `info` trả về đầy đủ metadata.
   - Test `skynet compute presets` và `compute dispatch` với file dữ liệu mẫu.
   - Test `skynet-agent --init-worker` và các cờ cấu hình.
3. **Workspace Verification:**
   - Chạy `cargo check --workspace`.
   - Chạy `cargo clippy --workspace --all-targets` (0 warnings).
   - Chạy `cargo test --workspace` (100% tests pass).

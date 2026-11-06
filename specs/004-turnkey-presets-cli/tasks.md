# Tasks: Hệ Thống Tiền Xử Lý Tự Hành, Presets Danh Mục & Bộ Lệnh CLI Thực Chiến

**Input**: Tài liệu thiết kế từ `specs/004-turnkey-presets-cli/` (`spec.md`, `plan.md`)

## Format: `- [ ] [TaskID] [P?] [US?] Mô tả với đường dẫn file cụ thể`
- **`[P]`**: Có thể thực hiện song song (khác file, không phụ thuộc lẫn nhau)
- **`[US#]`**: Ánh xạ tới User Story / Nhóm chức năng chính

---

## Phase 1: Setup & Crate Configuration

- [x] T001 Cập nhật `crates/skynet-node/Cargo.toml` bổ sung các cấu hình target `[[bin]]` (`skynet-node`, `skynet`, `skynet-agent`).

---

## Phase 2: Foundational Preprocessing Engine (`[FR-001]`, `[SC-001]`)

- [x] T002 [P] Hiện thực `crates/skynet-node/src/preprocessor/text_pii.rs` (Khử PII regex cho SĐT, Email, CCCD, Thẻ tín dụng, API Key và cắt lát văn bản Sliding-Window Chunking).
- [x] T003 [P] Hiện thực `crates/skynet-node/src/preprocessor/tabular_csv.rs` (Tự nhận diện Delimiter, điền giá trị khuyết thiếu Mean/Median/Zero/Drop, chuẩn hóa MinMax/Z-Score, chuẩn hóa SĐT E.164).
- [x] T004 [P] Hiện thực `crates/skynet-node/src/preprocessor/crypto_envelope.rs` (Sinh khóa phiên đối xứng và mã hóa phong bì ChaCha/stream bảo mật dữ liệu nhạy cảm).
- [x] T005 Hiện thực `crates/skynet-node/src/preprocessor/mod.rs` (Pipeline orchestrator `PreprocessPipeline`, `PreprocessOptions`, `PreprocessResult`, chuyển đổi vector `Vec<f32>` tương thích `CandleTaskInput`).
- [x] T006 Thêm unit tests trong `crates/skynet-node/src/preprocessor/mod.rs` kiểm chứng toàn diện các kịch bản tiền xử lý.

---

## Phase 3: Presets Catalog & Re-exports (`[FR-002]`)

- [x] T007 Hiện thực `crates/skynet-node/src/presets.rs` định nghĩa danh mục `TurnkeyPreset` (`text-audit`, `csv-clean-normalize`, `image-batch-upscale`, `log-anomaly-detection`, `phone-standardize`), hàm tra cứu và unit tests.
- [x] T008 Cập nhật `crates/skynet-node/src/lib.rs` re-export các submodules `preprocessor` và `presets` để phục vụ ứng dụng nhị phan và integration tests.

---

## Phase 4: User Client CLI `skynet` (`[FR-003]`, `[SC-002]`)

- [x] T009 Hiện thực `crates/skynet-node/src/bin/skynet.rs` hỗ trợ các lệnh: `wallet (init, balance)`, `airdrop claim`, `opcode (list, search, info)`, `compute (presets, dispatch)` đồng bộ với tài liệu `guide/`.
- [x] T010 Viết integration test `crates/skynet-node/tests/skynet_cli_test.rs` kiểm chứng các luồng lệnh của `skynet`.

---

## Phase 5: Worker Daemon CLI `skynet-agent` (`[FR-004]`, `[SC-002]`)

- [x] T011 Hiện thực `crates/skynet-node/src/bin/skynet_agent.rs` hỗ trợ `--init-worker`, `--listen`, `--auto-bid`, `--tags`, `--min-bounty-rate`, `--max-vram-gb`, `--battery-guard`, `--temp-limit`, `--schedule`.
- [x] T012 Viết integration test `crates/skynet-node/tests/skynet_agent_cli_test.rs` kiểm chứng khởi tạo worker profile và nhận diện phần cứng.

---

## Phase 6: Dashboard Turnkey Preprocessing Studio (`[FR-005]`, `[SC-003]`)

- [x] T013 Mở rộng `crates/skynet-dashboard/src/main.rs` và `crates/skynet-dashboard/src/preprocess_api.rs` bổ sung API endpoints `/api/presets` và `/api/preprocess`.
- [x] T014 Cập nhật `crates/skynet-dashboard/assets/dashboard.html` bổ sung thẻ giao diện `Turnkey Preprocessing Studio` với bộ chọn preset, checkboxes bộ lọc, so sánh 2 cột trực quan, số liệu tài nguyên, nút phát lệnh và sao chép tập lệnh CLI.

---

## Phase 7: Polish & Quality Verification (`[SC-004]`)

- [x] T015 Chạy kiểm tra kiểu `cargo check --workspace` (Passed 0 errors).
- [x] T016 Chạy linter `cargo clippy --workspace --all-targets` và xử lý triệt để cảnh báo (Passed 0 warnings).
- [x] T017 Chạy toàn bộ test suite `cargo test --workspace` đạt 100% passed (69/69 tests passed).
- [x] T018 Kiểm toán giới hạn vật lý (File <= 250 dòng, hàm <= 40 dòng) theo chuẩn Constitution.

# Tasks: Đóng Gói Daemon `skynet-node` & Tầng Lưu Trữ Bền Vững

**Input**: Tài liệu thiết kế từ `specs/003-node-daemon-persistence/` (`spec.md`, `plan.md`)

## Format: `- [ ] [TaskID] [P?] [US?] Mô tả với đường dẫn file cụ thể`
- **`[P]`**: Có thể thực hiện song song (khác file, không phụ thuộc lẫn nhau)
- **`[US#]`**: Ánh xạ tới User Story / Nhóm chức năng chính

---

## Phase 1: Setup & Crate Scaffolding

- [x] T001 Cập nhật `Cargo.toml` root workspace bổ sung thành viên `"crates/skynet-node"` và kích hoạt feature `signal` cho `tokio`.
- [x] T002 Tạo thư mục `crates/skynet-node` và cấu hình `crates/skynet-node/Cargo.toml` kết nối các dependency nội bộ và ngoài.

---

## Phase 2: Foundational Storage & Identity Modules

- [x] T003 [P] Hiện thực `crates/skynet-node/src/config.rs` định nghĩa `NodeConfig`, `CliCommand`, các cờ dòng lệnh và xử lý parse CLI an toàn. (`FR-001`)
- [x] T004 [P] Hiện thực `crates/skynet-node/src/identity.rs` định nghĩa `NodeIdentity`, load hoặc generate khóa Ed25519 với quyền file 0600. (`FR-002`)
- [x] T005 [P] Hiện thực `crates/skynet-node/src/storage_types.rs` định nghĩa các DTO tuần tự hóa: `LedgerSnapshot`, `OrganismSnapshot`, `PeerTableSnapshot`, `NodeMetadata`. (`FR-003`)
- [x] T006 Hiện thực `crates/skynet-node/src/storage.rs` định nghĩa `StorageEngine` với cơ chế ghi file nguyên tử `.tmp -> rename` và các phương thức `save_state` / `load_state`. (`FR-003`, `SC-002`)
- [x] T007 Thêm unit tests cho `identity.rs` và `storage.rs` kiểm chứng tính toàn vẹn dữ liệu và ghi nguyên tử. (`SC-001`, `SC-002`)

---

## Phase 3: Daemon Runtime & Tokio UDP Event Loop

- [x] T008 Hiện thực `crates/skynet-node/src/runtime.rs` tích hợp `tokio` event loop điều phối nhận gói tin UDP, nhịp sinh học biological ticks và định kỳ auto-flush. (`FR-004`)
- [x] T009 Bổ sung cơ chế Graceful Shutdown (`tokio::signal::ctrl_c()`) trong `crates/skynet-node/src/runtime.rs` đảm bảo flush toàn bộ trạng thái xuống đĩa trước khi thoát. (`FR-005`)
- [x] T010 Xuất bản thư viện `crates/skynet-node/src/lib.rs` re-export các submodules cho việc kiểm thử tích hợp.

---

## Phase 4: CLI Application & Status Subcommand

- [x] T011 Hiện thực `crates/skynet-node/src/main.rs` điều hướng lệnh `run` (khởi chạy daemon) và `status` (đọc đĩa in thông số node). (`FR-006`)
- [x] T012 Viết integration test `crates/skynet-node/tests/persistence_roundtrip_test.rs` kiểm chứng khôi phục 100% trạng thái Node sau khi restart. (`SC-001`)
- [x] T013 Viết integration test `crates/skynet-node/tests/multi_node_local_test.rs` kiểm chứng 2 node chạy song song 2 cổng trên 2 data-dir khác nhau. (`SC-003`)

---

## Phase 5: Polish & Quality Verification

- [x] T014 Chạy kiểm tra kiểu `cargo check --workspace`.
- [x] T015 Chạy linter `cargo clippy --workspace --all-targets` và giải quyết cảnh báo.
- [x] T016 Chạy toàn bộ test suite `cargo test --workspace` đạt 100% passed.
- [x] T017 Kiểm toán giới hạn vật lý (File <= 250 dòng, hàm <= 40 dòng) theo chuẩn Constitution. (`SC-004`)

# Tasks: Nghi Thức Khởi Nguyên Genesis & Đục Lỗ NAT UDP P2P Rendezvous

**Input**: Tài liệu thiết kế từ `specs/005-genesis-and-p2p-rendezvous/` (`spec.md`, `plan.md`)

## Format: `- [ ] [TaskID] [P?] [US?] Mô tả với đường dẫn file cụ thể`
- **`[P]`**: Có thể thực hiện song song (khác file, không phụ thuộc lẫn nhau)
- **`[US#]`**: Ánh xạ tới User Story / Nhóm chức năng chính

---

## Phase 1: Setup & Crate Configuration

- [ ] T001 Cập nhật `crates/skynet-ledger/Cargo.toml` và `crates/skynet-wire/Cargo.toml` kiểm tra dependencies cần thiết (`blake3`, `ed25519-dalek`, `serde_json`).

---

## Phase 2: Foundational Genesis Engine (`[FR-001]`, `[SC-001]`, `[US1]`)

- [ ] T002 [P] [US1] Hiện thực `crates/skynet-ledger/src/genesis.rs` định nghĩa `GenesisConfig`, `GenesisAllocation`, `GenesisUtxoEntry`, hàm tính toán `compute_genesis_hash` (Blake3) và kiểm chứng bất biến `validate_genesis_invariants`.
- [ ] T003 [US1] Cập nhật `crates/skynet-ledger/src/lib.rs` re-export module `genesis` và các kiểu dữ liệu liên quan.
- [ ] T004 [US1] Viết unit tests trong `crates/skynet-ledger/src/genesis.rs` kiểm chứng bất biến bảo toàn $21\text{T SKN}$ (85% Vault + 15% circulating), phát hiện sai lệch và băm canonical Blake3.

---

## Phase 3: Wire Protocol & Epidermal Membrane Filter (`[FR-002]`, `[FR-003]`, `[SC-002]`, `[SC-003]`, `[US2]`)

- [ ] T005 [P] [US2] Hiện thực `crates/skynet-wire/src/rendezvous_wire.rs` định nghĩa các struct bản tin `PheromoneHello`, `PunchRendezvous` kèm phương thức serialize/deserialize nhị phân nhỏ gọn ($\le 300\text{ bytes}$).
- [ ] T006 [US2] Bổ sung module màng lọc tế bào Da `EpidermalFilter` trong `crates/skynet-wire/src/rendezvous_wire.rs` thực hiện kiểm tra `network_id == GENESIS_HASH`, kiểm tra dung sai thời gian $\pm 30\text{s}$ và xác thực chữ ký số Ed25519 trên stack buffer với 0 cấp phát heap.
- [ ] T007 [US2] Cập nhật `crates/skynet-wire/src/lib.rs` re-export các kiểu bản tin rendezvous và màng lọc Da.
- [ ] T008 [US2] Viết unit tests trong `crates/skynet-wire/src/rendezvous_wire.rs` kiểm chứng lọc gói tin sai mạng, gói tin quá hạn và kiểm chứng kích thước MTU $\le 300\text{ bytes}$.

---

## Phase 4: Rendezvous Broker Engine (`[FR-004]`, `[US3]`)

- [ ] T009 [P] [US3] Hiện thực `crates/skynet-agent/src/rendezvous_broker.rs` cung cấp `RendezvousRegistry` quản lý danh bạ peer điểm hẹn, ghép cặp hai node sau NAT và tạo bản tin đục lỗ `PunchRendezvous`.
- [ ] T010 [US3] Cập nhật `crates/skynet-agent/src/lib.rs` re-export module `rendezvous_broker`.
- [ ] T011 [US3] Viết unit tests trong `crates/skynet-agent/src/rendezvous_broker.rs` kiểm chứng đăng ký peer, hết hạn TTL tự động và logic ghép cặp 2 bên.

---

## Phase 5: CLI Genesis Tool & Daemon Seed Integration (`[FR-005]`, `[US1]`, `[US3]`)

- [ ] T012 [US1] Mở rộng `crates/skynet-node/src/bin/skynet.rs` bổ sung nhánh lệnh `genesis`: `generate --output <PATH> [--network <NAME>]` và `verify --path <PATH>`.
- [ ] T013 [US3] Mở rộng `crates/skynet-node/src/config.rs` bổ sung các cờ `--seed <ADDR>` và `--genesis <PATH>` vào `NodeConfig`.
- [ ] T014 [US3] Cập nhật `crates/skynet-node/src/runtime.rs` tích hợp tác vụ định kỳ bắn `PheromoneHello` tới Seed Node và phản ứng bắn xung đục lỗ đồng thời khi nhận `PunchRendezvous`.

---

## Phase 6: Integration Tests & Cross-Node Hole Punch Test (`[SC-001]`, `[SC-002]`, `[SC-003]`)

- [ ] T015 Viết integration test `crates/skynet-node/tests/genesis_cli_test.rs` kiểm chứng trọn vẹn luồng sinh file genesis, kiểm tra băm Blake3 và xác thực bất biến.
- [ ] T016 Viết integration test `crates/skynet-node/tests/p2p_rendezvous_hole_punch_test.rs` mô phỏng 1 Seed Node và 2 Node NAT đục lỗ UDP thành công qua điểm hẹn.

---

## Phase 7: Polish & Quality Verification (`[SC-004]`)

- [ ] T017 Chạy kiểm tra kiểu `cargo check --workspace` (0 errors).
- [ ] T018 Chạy linter `cargo clippy --workspace --all-targets` và xử lý triệt để cảnh báo (0 warnings).
- [ ] T019 Chạy toàn bộ test suite `cargo test --workspace` đạt 100% passed.
- [ ] T020 Kiểm toán giới hạn vật lý (File $\le 250$ dòng, hàm $\le 40$ dòng) theo chuẩn Constitution.

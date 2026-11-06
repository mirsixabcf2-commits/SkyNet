# Kế Hoạch Kỹ Thuật: Giao Thức Điện Toán Từ Xa & Ký Quỹ Tiền Thưởng (Remote Compute & Candle Execution)

**Feature Directory**: `specs/001-remote-compute-candle` | **Date**: 2026-11-04 | **Spec**: [spec.md](./spec.md)

---

## 1. Tóm Tắt (Summary)

Hiện thực hóa toàn diện cơ chế đóng gói, phân loại, ký quỹ hai chiều và thực thi tính toán từ xa tập trung vào mạng nơ-ron `Candle` (hỗ trợ tự động CUDA / Metal / CPU).
Kiến trúc phân rã theo 4 crates độc lập nhằm tuân thủ nghiêm ngặt giới hạn vật lý của Hiến pháp SkyNet:
1. `skynet-wire::compute_wire`: Định nghĩa gói tin `ComputeTaskManifest`, các bản tin đấu thầu, hoàn tất và tích hợp `ImpulseBody::Compute`.
2. `skynet-ledger::compute_escrow`: Ký quỹ tiền thưởng `NanoSkn` hai chiều (`Bounty` từ Client, `Bond` từ Worker), thanh toán và trừng phạt slashing.
3. `skynet-cell::compute_candle`: Động cơ thực thi tensor `Candle` với phát hiện phần cứng thông minh (`Device::new_cuda` -> `Device::new_metal` -> `Device::Cpu`) và băm kết quả kiểm chứng `BLAKE3`.
4. `skynet-agent::compute_flow`: Bộ điều phối `ComputeCoordinator` kết nối vòng đời end-to-end từ lúc tạo task, truyền gói tin UDP, xử lý đến khi quyết toán tiền thưởng.

---

## 2. Bối Cảnh Kỹ Thuật (Technical Context)

- **Ngôn ngữ / Runtime:** Rust 2021 (MSRV `1.83.0`), `tokio` async runtime.
- **Thư viện AI & Toán học:** `candle-core` (Tensor inference), `blake3` (Cryptographic hashing), `ed25519-dalek` (Chữ ký điện tử).
- **Thư viện Sổ cái & Mã hóa gói tin:** `bincode`, `serde`, `thiserror`.
- **Giao thức Mạng:** UDP phi trạng thái, giới hạn cứng `MAX_UDP_DATAGRAM_BYTES = 1200`, mã hóa xóa `FountainDroplet` $\text{GF}(2)$ cho payload lớn.
- **Ràng buộc Hiến pháp:**
  - Mỗi file mới $\le 250$ dòng (giới hạn cứng $\le 350$ dòng).
  - Mỗi hàm $\le 40$ dòng, $\le 3-4$ tham số, độ sâu lồng nhau $\le 2-3$ cấp (Early Return).
  - Không dùng `.unwrap()` runtime, bắt lỗi tường minh với `Result<T, E>`.

---

## 3. Constitution Check

*GATE: Kiểm tra trước khi triển khai.*

- [x] **File & Function limits:** Toàn bộ logic mới được tách thành các file submodule riêng biệt (`compute_wire.rs`, `compute_escrow.rs`, `compute_candle.rs`, `compute_flow.rs`), không nhồi nhét vào `lib.rs` cũ.
- [x] **Single Concept per File:** Mỗi submodule đảm nhận một vai trò duy nhất trong tầng kiến trúc tương ứng.
- [x] **Blast Radius:** Chỉ thêm submodule mới và export tại `lib.rs`, không làm xáo trộn các unit test hiện có.
- [x] **Error Handling:** Định nghĩa các enum lỗi chi tiết (`ComputeWireError`, `EscrowError`, `CandleComputeError`).
- [x] **Verification Gate:** Phải vượt qua `cargo check --workspace`, `cargo clippy --workspace --all-targets` và `cargo test --workspace`.

---

## 4. Cấu Trúc File & Mã Nguồn (Project Structure)

### Tài Liệu Đặc Tả (Specs)
```text
specs/001-remote-compute-candle/
├── spec.md                       # Đặc tả yêu cầu & kịch bản chấp thuận
├── checklists/requirements.md   # Bảng kiểm tra chất lượng spec
├── plan.md                       # Kế hoạch kỹ thuật (file này)
└── tasks.md                      # Danh mục tác vụ triển khai
```

### File Mã Nguồn Dự Kiến (Target Source Code)
```text
crates/
├── skynet-wire/src/
│   ├── compute_wire.rs           # NEW: TaskManifest, ComputeMessage, Droplet wrapping (<= 250 lines)
│   └── lib.rs                    # MODIFIED: export compute_wire và ImpulseBody::Compute
├── skynet-ledger/src/
│   ├── compute_escrow_types.rs   # NEW: Escrow data models, split config & breakdown (<= 250 lines)
│   ├── compute_escrow.rs         # NEW: ComputeEscrowVault, Dual-Staking Escrow, Slashing (<= 350 lines)
│   └── lib.rs                    # MODIFIED: export compute_escrow
├── skynet-cell/src/
│   ├── compute_candle.rs         # NEW: CandleComputeExecutor, CUDA/Metal/CPU forward, Digest (<= 250 lines)
│   └── lib.rs                    # MODIFIED: export compute_candle
└── skynet-agent/src/
    ├── compute_flow.rs           # NEW: ComputeCoordinator kết nối Client <-> Worker (<= 350 lines)
    ├── auditor_flow.rs           # NEW: AuditorCoordinator kết nối Quorum kiểm chứng (<= 250 lines)
    └── lib.rs                    # MODIFIED: export compute_flow & auditor_flow
```

# [TASK-20261106-GEN01] Nghi Thức Khởi Nguyên Genesis & Đục Lỗ NAT UDP P2P Rendezvous

## Thông Tin Chung
- **Mã Định Danh:** TASK-20261106-GEN01
- **Mức Độ Ưu Tiên:** High (Mainnet Readiness Wave 1)
- **Thời Điểm Mở (Open Time):** `2026-11-06 09:15:00 +07:00`
- **Thời Điểm Đóng (Closed Time):** `2026-11-12 09:08:54 +07:00`
- **Trạng Thái Hiện Tại:** Đã Hoàn Thành
- **Mục Tiêu:** Thiết lập cấu hình khởi nguyên sổ cái Genesis chuẩn hóa (`genesis.json`), sinh mã băm định danh mạng bất biến `GENESIS_HASH` qua Blake3, màng lọc tế bào Da chống giả mạo zero-allocation, và quy trình 4 bước đục lỗ NAT UDP P2P qua Pioneer Seed Rendezvous theo tài liệu kiến trúc.
- **Crates Liên Quan:** `skynet-ledger`, `skynet-wire`, `skynet-agent`, `skynet-node`.

---

## Bảng Checklist Chi Tiết Nhiệm Vụ

| Mã | Hạng Mục Công Việc | Trạng Thái | Thời Điểm Mở (Open Time) | Thời Điểm Đóng (Closed Time) | Phạm Vi Triển Khai & Giải Pháp |
| :---: | :--- | :---: | :---: | :---: | :--- |
| **T01** | **Cấu hình Crate Dependencies** | [x] | 2026-11-06 09:15:00 | 2026-11-12 08:05:00 | Kiểm tra dependencies `blake3`, `ed25519-dalek`, `serde_json` trong `skynet-ledger` và `skynet-wire`. |
| **T02** | **Hiện thực Sổ cái Genesis (`genesis.rs`)** | [x] | 2026-11-06 09:15:00 | 2026-11-12 08:05:00 | `crates/skynet-ledger/src/genesis.rs`: Cấu trúc `GenesisConfig`, hàm băm canonical Blake3 `compute_genesis_hash`, kiểm tra bất biến 21T SKN (85% Vault + 15% circulating). |
| **T03** | **Re-export & Unit Tests Genesis** | [x] | 2026-11-06 09:15:00 | 2026-11-12 08:05:00 | `crates/skynet-ledger/src/lib.rs`: Xuất bản module và kiểm chứng toán học bảo toàn 100% `NanoSkn(u128)`. |
| **T04** | **Khung tin Rendezvous & Màng lọc Da** | [x] | 2026-11-06 09:15:00 | 2026-11-12 08:05:00 | `crates/skynet-wire/src/rendezvous_wire.rs` & `epidermal_filter.rs`: Định nghĩa `PheromoneHello`, `PunchRendezvous` ($\le 300\text{B}$) và màng lọc Da `EpidermalFilter` (stack zero-alloc). |
| **T05** | **Re-export & Unit Tests Wire Rendezvous** | [x] | 2026-11-06 09:15:00 | 2026-11-12 08:05:00 | `crates/skynet-wire/src/lib.rs`: Kiểm chứng lọc sai `GENESIS_HASH`, lệch thời gian $\pm 30\text{s}$ và xác thực chữ ký Ed25519. |
| **T06** | **Điều phối Điểm hẹn (`rendezvous_broker.rs`)** | [x] | 2026-11-06 09:15:00 | 2026-11-12 08:20:07 | `crates/skynet-agent/src/rendezvous_broker.rs`: `RendezvousRegistry` quản lý peer, ghép cặp 2 node sau NAT và sinh token đục lỗ. |
| **T07** | **Re-export & Unit Tests Rendezvous Broker** | [x] | 2026-11-06 09:15:00 | 2026-11-12 08:20:07 | `crates/skynet-agent/src/lib.rs`: Kiểm chứng TTL hết hạn, ghép cặp peer và tạo bản tin đục lỗ. |
| **T08** | **Mở rộng CLI `skynet genesis`** | [x] | 2026-11-06 09:15:00 | 2026-11-12 08:55:00 | `crates/skynet-node/src/cli/genesis_airdrop_cmd.rs`: Lệnh `genesis generate` và `genesis verify`. |
| **T09** | **Tích hợp cờ `--seed` & `--genesis` vào NodeConfig** | [x] | 2026-11-06 09:15:00 | 2026-11-12 08:55:00 | `crates/skynet-node/src/config.rs`: Bổ sung cấu hình seed peers và file genesis. |
| **T10** | **Vòng lặp Bắn xung Đục lỗ trong Daemon Runtime** | [x] | 2026-11-06 09:15:00 | 2026-11-12 08:55:00 | `crates/skynet-node/src/runtime_p2p.rs`: Tự động bắn `PheromoneHello` định kỳ và bắn xung đục lỗ đồng thời khi nhận `PunchRendezvous`. |
| **T11** | **Integration Tests & Kiểm chứng Hệ Thống** | [x] | 2026-11-06 09:15:00 | 2026-11-12 09:08:54 | Viết `tests/genesis_cli_test.rs` và `tests/p2p_rendezvous_hole_punch_test.rs`, chạy `cargo check`, `cargo clippy`, `cargo test` xanh 100%. |

---

## Nhật Ký Thực Hiện (Activity Log)
- `2026-11-06 09:15:00`: Khởi tạo nhiệm vụ theo yêu cầu kích hoạt Spec 005 (Genesis Ceremony & P2P UDP Rendezvous).
- `2026-11-12 09:08:54`: Hoàn tất và nghiệm thu 100% toàn bộ hạng mục `T01`–`T11` trong đợt triển khai hợp nhất cùng `specs/006-opportunistic-edge-network`.

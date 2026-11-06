# [SEC-HARDENING-01] Khắc Phục 11 Lỗ Hổng Bảo Mật & Chống Gian Lận Hệ Thống

## Thông Tin Chung
- **Mã Định Danh:** TASK-20261104-SEC01
- **Mức Độ Ưu Tiên:** Critical
- **Thời Điểm Mở (Open Time):** `2026-11-04 18:25:00 +07:00`
- **Thời Điểm Đóng (Closed Time):** `2026-11-04 18:38:00 +07:00`
- **Trạng Thái Hiện Tại:** Đã Hoàn Thành
- **Mục Tiêu:** Khắc phục toàn bộ các bề mặt tấn công gây mất an toàn đồng thuận, rò rỉ quỹ ký quỹ, gian lận bằng chứng điện toán và tấn công từ chối dịch vụ (DoS) đã phát hiện trong quá trình kiểm toán mã nguồn.
- **Crates Liên Quan:** `skynet-agent`, `skynet-wire`, `skynet-ledger`, `skynet-cell`, `skynet-dashboard`, `skynet-node`.

---

## Bảng Checklist Chi Tiết Nhiệm Vụ

| Mã | Hạng Mục Công Việc | Trạng Thái | Thời Điểm Mở (Open Time) | Thời Điểm Đóng (Closed Time) | Phạm Vi Triển Khai & Giải Pháp |
| :---: | :--- | :---: | :---: | :---: | :--- |
| **T01** | **Vá lỗ hổng đồng thuận Snowball (SEC-01)**<br>Kiểm tra Query ID, danh sách $k$ peer lấy mẫu hợp lệ và tính trọng số phiếu theo Stake SKN | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-agent/src/lib.rs`<br>Thêm tracking `PendingSnowballQuery`, lọc `query_id` hợp lệ, deduplicate sender và chỉ nhận phiếu từ các peer được chọn. |
| **T02** | **Sửa lỗi dìm điểm Hebbian trong routing table (SEC-02)**<br>Chống tấn công Eclipse, chỉ trừ điểm khi node không phản hồi | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-wire/src/peer_table.rs`<br>Xóa việc dìm điểm incumbent khi nhận gói yếu hơn; cap `stake_bonus <= 250`, phạt qua `penalize_unresponsive_peer` và `decay_synapses`. |
| **T03** | **Xác thực khoản ký quỹ Auditor trên chuỗi (SEC-03)**<br>Ngăn chặn tạo tiền khống qua `register_auditor` | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-ledger/src/compute_escrow.rs` & `skynet-agent`<br>Bổ sung `total_accounted_vault_balance` kiểm soát bảo toàn tài sản on-chain; thêm domain separators chống giả mạo chứng thực. |
| **T04** | **Cơ chế thanh toán vi mô đơn phương (SEC-04)**<br>Bảo vệ Worker khỏi bị bùng nợ SynapticVoucher | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-ledger/src/lib.rs`<br>Bổ sung `VoucherChannelTracker::build_unilateral_settlement_tx` và `UtxoDagLedger::admit_unilateral_voucher_settlement` cho phép Payee đơn phương thanh toán. |
| **T05** | **Chứng thực phần cứng thật cho Airdrop (SEC-05)**<br>Chống bòn rút quỹ tiếp thị Genesis bằng mã băm HWID ảo | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-ledger/src/hwid_airdrop.rs`<br>Chặn zero HWID/proof, thêm chữ ký Ed25519 `attestation_sig` và kiểm thực chữ ký bảo chứng phần cứng. |
| **T06** | **Khử tính bất định dấu phẩy động Candle AI (SEC-06)**<br>Đảm bảo cùng digest giữa CUDA, Metal và CPU | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-cell/src/compute_candle.rs`<br>Lượng tử hóa tensor output sang Fixed-Point (`FIXED_POINT_DIGEST_SCALE = 10_000.0`) trước khi tính canonical digest và bổ sung `verify_output_tolerance`. |
| **T07** | **Đo lường thời gian PoPE hai chiều phía Verifier (SEC-07)**<br>Vô hiệu hóa việc tự khai báo `duration_micros` | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-cell/src/pope_benchmark.rs`<br>Bổ sung `verify_pope_proof_with_latency` cho phép Verifier đo thời gian RTT độc lập chống làm giả tốc độ thực thi. |
| **T08** | **Cách ly và kiểm dịch Plasmid nặc danh (SEC-08)**<br>Chống tấn công Trojan đầu độc tế bào thần kinh & da | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-agent/src/lib.rs`<br>Giới hạn tối đa 32 fountain decoders chống cạn kiệt bộ nhớ; khóa bảo vệ loại tế bào da (`Epidermal`) và nội tạng (`Visceral`) không bị plasmid sửa đổi. |
| **T09** | **Cơ chế biểu quyết cho phân tách cụm (SEC-09)**<br>Chặn đứng việc ép tách cụm tùy tiện qua `ClusterMitosis` | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-agent/src/lib.rs`<br>Quorum gate cho `ClusterMitosis`: yêu cầu tối thiểu 2 phiếu từ intra-cluster peers đã xác thực hoặc node đạt 90% quy mô trước khi tách cụm. |
| **T10** | **Khóa chặt bảo mật Dashboard API (SEC-10)**<br>Loại bỏ CORS Wildcard `*` và thêm Token xác thực API | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-dashboard/src/main.rs`<br>Chỉ cho phép origin `127.0.0.1` / `localhost`, chặn header `Sec-Fetch-Site: cross-site` đối với endpoint nhạy cảm `/api/control`. |
| **T11** | **Chống khuếch đại DDoS & Sig-Flooding UDP (SEC-11)**<br>Bảo vệ socket trước các đòn tấn công mạng | [x] | 2026-11-04 18:25:00 | 2026-11-04 18:38:00 | `crates/skynet-wire/src/lib.rs`<br>Kiểm tra `hop_ttl` (hợp lệ trong khoảng 1..=16) trước khi giải mã chữ ký Ed25519; tách mã nguồn sang module riêng biệt. |

---

## Nhật Ký Thực Hiện (Activity Log)
- `2026-11-04 18:25:00`: Khởi tạo danh mục 11 hạng mục công việc bảo mật dựa trên kết quả kiểm toán an ninh hệ thống.
- `2026-11-04 18:38:00`: Triển khai mã nguồn vá toàn bộ 11 lỗ hổng bảo mật (T01 đến T11). Tái cấu trúc modular hóa `skynet-wire` thành `fountain.rs` và `peer_table.rs`. Chạy kiểm thử toàn bộ workspace: 50/50 tests passed, cargo clippy sạch 0 cảnh báo. Đóng tất cả các task sang trạng thái hoàn thành.

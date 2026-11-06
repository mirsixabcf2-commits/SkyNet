# Requirements Quality Checklist: HWID Marketing Airdrop

## Quality Gate Verification

- [x] **No Ambiguities**: Tiêu chí xác định trùng HWID, đường cong halving, quy cách kiểm tra PoPE đều có định nghĩa kiểu và công thức cụ thể.
- [x] **Zero-Inflation Guarantee**: Nguồn cấp phát bắt buộc lấy từ quỹ dự trữ Genesis định sẵn (`GENESIS_MAX_SKN`), không mint ngoài sổ cái.
- [x] **Anti-Sybil Robustness**: Kết hợp định danh phần cứng và bài toán thực thi vi nơ-ron Candle bão hòa bộ nhớ.
- [x] **Custody Covenant Enforcement**: Sử dụng `CustodyPolicy::AgentBond` với `max_voucher_allowance = 0` ngăn chặn tẩu tán tài sản.
- [x] **Constitution & Physical Limits Alignment**: Thiết kế tách rời module `hwid_airdrop.rs` và `pope_benchmark.rs`, đảm bảo file <= 250 dòng, hàm <= 40 dòng.

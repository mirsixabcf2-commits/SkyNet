# Feature Specification: Cơ chế Marketing Airdrop theo HWID chống Gian lận & Lạm phát

## 1. Tổng quan & Mục tiêu Nghiệp vụ (Overview & Goals)
Để mở rộng nhanh mạng lưới phi tập trung SkyNet trong giai đoạn đầu (Bootstrapping phase), hệ thống cần một cơ chế phân phối phần thưởng khởi tạo (Marketing Airdrop) cho người dùng mới khi cài đặt và chạy node.

Nhằm bảo vệ hệ sinh thái khỏi các cuộc tấn công Sybil (gian lận giả lập hàng loạt máy ảo) và bảo vệ giá trị kinh tế của token SKN, cơ chế này tuân thủ các nguyên tắc:
1. **Chống lạm phát tuyệt đối (Zero-Inflation Invariant):** Toàn bộ token thưởng được trích từ quỹ dự trữ Genesis định trước (`MarketingBootstrapVault`), tuân thủ trần cứng 21 Trillion SKN (`GENESIS_MAX_SKN`).
2. **Halving lũy tiến (Progressive Halving):** Mức thưởng giảm một nửa (1 SKN -> 0.5 SKN -> 0.25 SKN...) sau mỗi cột mốc số lượng thiết bị xác thực (ví dụ: mỗi 100.000 thiết bị), tạo tính khan hiếm và khuyến khích người dùng sớm.
3. **Chống giả mạo HWID qua Bằng chứng Thực thi Vật lý (PoPE - Proof of Physical Execution):** Không dựa vào các thông số tĩnh dễ bị làm giả của hệ điều hành. Yêu cầu node thực thi một tác vụ tính toán ma trận / nơ-ron Candle bão hòa bộ nhớ (Memory-hard) với giới hạn thời gian thực nghiêm ngặt để triệt tiêu khả năng chia sẻ phần cứng vật lý qua nhiều máy ảo (VMware, QEMU, Docker).
4. **Ràng buộc Kinh tế Sinh học (Biological Vesting & Bond):** Token thưởng được khóa dưới dạng `AgentBond` phục vụ chuyển hóa năng lượng tế bào (`Cellular ATP`) và ký quỹ xử lý tác vụ vi nơ-ron, ngăn chặn hoàn toàn hành vi "Dump & Run" (xả hàng ngay lập tức).

---

## 2. Yêu cầu Chức năng (Functional Requirements)

### [FR-001] Quản trị Quỹ Marketing Genesis (`MarketingBootstrapVault`)
- Hệ thống PHẢI khởi tạo quỹ tiếp thị từ lượng dự trữ Genesis với trần cố định.
- Hệ thống PHẢI theo dõi chính xác:
  - `total_allocated`: Tổng lượng `NanoSkn` đã cấp phát.
  - `remaining_reserve`: Lượng `NanoSkn` còn lại trong quỹ.
  - `total_devices_registered`: Số thiết bị duy nhất đã được cấp phát.
  - Bảng băm danh sách các thiết bị đã nhận `claimed_devices: HashMap<[u8; 32], HwidClaimRecord>`.
- **Bảo toàn khối lượng:** Bất biến `total_allocated + remaining_reserve == initial_reserve` PHẢI luôn đúng trong mọi trường hợp.

### [FR-002] Cơ chế Halving Mức Thưởng Thiết bị
- Mức thưởng khởi đầu mặc định là `1 SKN` (`1_000_000_000 nanoSKN`).
- Sau mỗi chu kỳ `halving_step_devices` (mặc định 100.000 thiết bị), mức thưởng giảm $50\%$ (dịch bit phải `base_reward >> (total_devices / step)`).
- Khi lượng dự trữ còn lại nhỏ hơn mức thưởng hiện tại, chỉ cấp phát phần còn lại hoặc báo lỗi `ReserveExhausted`.

### [FR-003] Định danh Thiết bị & Chống Nhận Thưởng Trùng (Double-Claim Guard)
- Mỗi thiết bị được định danh bằng mã băm `HwidDigest` (`[u8; 32]`).
- Khi một thiết bị gửi yêu cầu nhận airdrop:
  - Nếu `hwid_digest` đã tồn tại trong `claimed_devices`, hệ thống PHẢI từ chối ngay lập tức với lỗi `MarketingError::HwidAlreadyClaimed`.
  - Nếu hợp lệ, ghi nhận `HwidClaimRecord` bao gồm `hwid_digest`, `claimed_epoch`, `allocated_bond`, `utxo_id`, và `proof_digest`.

### [FR-004] Bằng chứng Thực thi Vật lý (Proof of Physical Execution - PoPE)
- Node yêu cầu airdrop PHẢI nộp một `PopeProof` được tính toán thông qua động cơ tensor `skynet-cell`.
- Tác vụ PoPE bao gồm:
  - Nhận `challenge_nonce` (32 bytes) từ mạng lưới và kết hợp với `hwid_digest`.
  - Chạy chuỗi phép tính tensor nơ-ron bão hòa bộ nhớ / bộ đệm ma trận.
  - Tạo ra `result_digest` (`[u8; 32]`) cùng thời gian thực thi `duration_micros`.
- Bộ kiểm chứng (Verifier / Ledger) PHẢI kiểm tra tính chính xác của `result_digest` và đảm bảo `duration_micros <= max_allowed_micros`.

### [FR-005] Khóa Giám hộ Sinh học & Mở khóa Cọc (`Marketing AgentBond`)
- Lượng SKN thưởng PHẢI được đúc vào `TxOutput` với chính sách `CustodyPolicy::AgentBond`:
  - `human_owner`: Địa chỉ ví lạnh của người dùng.
  - `owner_pubkey`: Khóa nóng của AgentNode trên thiết bị.
  - `amount`: Mức thưởng `NanoSkn`.
  - `max_voucher_allowance = 0`: CẤM chuyển nhượng ra ngoài ví khác hoặc thanh toán voucher ngay lập tức.
- Token này có thể nạp vào `AgentMetabolicBank` để nuôi dưỡng các tế bào AI (`MicroCell`) và làm tiền cọc cho `ComputeEscrowVault`.

---

## 3. Tiêu chí Nghiệm thu (Success Criteria)

- **[SC-001] Kiểm toán Cân đối Tài chính:** Cấp phát thành công cho thiết bị hợp lệ; bất biến tổng cung của quỹ marketing bảo toàn 100% không lệch 1 nanoSKN.
- **[SC-002] Chặn đứng Gian lận Trùng HWID:** Cùng một `hwid_digest` cố gắng claim lần thứ hai PHẢI bị chặn với lỗi `HwidAlreadyClaimed`.
- **[SC-003] Kiểm thử Đường cong Halving:** Xác minh số lượng thưởng giảm đúng 50% khi vượt qua ngưỡng `halving_step_devices`.
- **[SC-004] Kiểm thử Xác thực PoPE:** Tác vụ tensor trả về digest chuẩn xác; các bài nộp giả mạo digest hoặc bị timeout đều bị từ chối với `InvalidProofOfExecution`.
- **[SC-005] Tuân thủ Kiến trúc & Giới hạn Vật lý:** Toàn bộ code mới đạt giới hạn <= 250 dòng/file, hàm <= 40 dòng, `cargo check`, `cargo clippy`, `cargo test` vượt qua 100%.

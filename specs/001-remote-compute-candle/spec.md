# Đặc Tả Tính Năng: Giao Thức Điện Toán Từ Xa & Ký Quỹ Tiền Thưởng (Remote Compute & Bounty Protocol)

**Feature Directory**: `specs/001-remote-compute-candle`  
**Created**: 2026-11-04  
**Status**: In-Review  

---

## Bối Cảnh & Mục Tiêu

Người dùng trong mạng lưới SkyNet cần gửi các tác vụ tính toán từ xa (đặc biệt là suy luận mạng nơ-ron AI, tính toán tensor trên `Candle`/CUDA) đến các Worker Node có cấu hình phần cứng phù hợp. Giao thức cần đảm bảo:
1. Đóng gói tác vụ trong giới hạn UDP MTU `<= 1200 bytes` (dùng Fountain Droplets khi dữ liệu lớn).
2. Cơ chế tiền thưởng (Bounty Escrow) khóa `NanoSkn` hai chiều chống bùng tiền và chống gian lận.
3. Node thợ giải mã an toàn, thực thi mô hình qua runtime `Candle`, trả kết quả và nhận thanh toán tự động.

---

## User Scenarios & Testing

### User Story 1 - Khởi Tạo & Thực Thi Tác Vụ Candle Micro-Neural Qua UDP (Priority: P1 — MVP)

Người dùng (Client Node) muốn gửi một tensor dữ liệu đầu vào đến Worker Node trên mạng để chạy suy luận nơ-ron vi mô bằng `candle-core` (hỗ trợ CPU/CUDA `Device`) và nhận lại kết quả tensor đã xử lý.

**Why this priority**: Đây là lát cắt MVP độc lập cốt lõi, chứng minh khả năng đóng gói, truyền tải UDP, giải mã và thực thi nơ-ron trên máy đích thành công.

**Independent Test**: Khởi tạo Client và Worker Node qua UDP socket cục bộ, Client phát lệnh tính toán tensor, Worker tính toán qua `CandleMicroExecutor` và gửi trả kết quả tensor khớp chính xác với kiểm chứng cục bộ.

**Acceptance Scenarios**:
1. **Given** Client có một vector tensor đầu vào 8 chiều và Worker có phần cứng đáp ứng `HardwareTier`, **When** Client gửi bản tin `ComputeTaskAnnouncement` và phân bổ task cho Worker, **Then** Worker thực thi qua `candle-core` và trả về kết quả tensor cùng chữ ký xác thực.
2. **Given** Worker nhận được gói tin tính toán không hợp lệ hoặc sai định dạng opcode, **When** Worker giải mã tại biên nhận, **Then** Worker từ chối xử lý và trả về mã lỗi rõ ràng mà không gây panic hay rò rỉ bộ nhớ.

---

### User Story 2 - Ký Quỹ Tiền Thưởng Khóa Kép & Quyết Toán Sổ Cái (Priority: P2)

Khách hàng muốn khóa tiền thưởng (`bounty_nano_skn`) vào một UTXO Escrow có điều kiện. Worker muốn khóa tiền bảo chứng (`worker_bond_nano_skn`) để cam kết không bỏ cuộc. Khi có kết quả hợp lệ, tiền cọc và tiền thưởng được giải phóng tự động cho Worker.

**Why this priority**: Bảo đảm an toàn kinh tế cho cả hai bên trong môi trường phi tín nhiệm (Zero-Trust), ngăn chặn việc Worker nhận task nhưng bỏ dở hoặc Client quỵt tiền thưởng.

**Independent Test**: Mô phỏng vòng đời UTXO Escrow: khóa tiền của Client + Worker -> nộp Digest kết quả hợp lệ -> giải phóng toàn bộ tiền thưởng và hoàn cọc cho Worker; thử nghiệm nhánh timeout để Client rút lại tiền khi Worker biến mất.

**Acceptance Scenarios**:
1. **Given** Client khóa 500 nanoSKN và Worker khóa 100 nanoSKN vào `ComputeEscrowUtxo`, **When** Worker nộp bằng chứng kết quả có chữ ký hợp lệ trước thời hạn `timeout_epoch`, **Then** Worker nhận 600 nanoSKN, Client nhận kết quả.
2. **Given** Worker đã nhận task nhưng quá hạn `timeout_epoch` không nộp kết quả, **When** Client yêu cầu hoàn tiền (Refund), **Then** Client nhận lại 500 nanoSKN tiền thưởng và 50% tiền cọc của Worker (50 nanoSKN), 50 nanoSKN còn lại bị thiêu hủy (slashed/burn).

---

### User Story 3 - Truyền Dữ Liệu Lớn Qua Dòng GF(2) Fountain Droplets (Priority: P3)

Người dùng có tensor đầu vào hoặc mô hình trọng số vượt quá giới hạn 1200 bytes cần truyền tải qua UDP mà không bị phân mảnh IP và không bị ảnh hưởng bởi rớt gói mạng.

**Why this priority**: Cho phép mở rộng từ các tác vụ micro-inference sang các tác vụ AI lớn hơn (ảnh, ma trận trọng số, batch tensors).

**Independent Test**: Bơm dữ liệu 4 KB - 64 KB qua bộ mã hóa `FountainEncoder`, gây rớt ngẫu nhiên 30% gói tin UDP, kiểm tra Worker tái tạo thành công 100% dữ liệu gốc qua `FountainDecoder` và thực thi Candle trơn tru.

**Acceptance Scenarios**:
1. **Given** Dữ liệu payload 8 KB được cắt thành các khối 512B, **When** Truyền qua mạng UDP với tỉ lệ rớt gói 30%, **Then** Worker thu thập đủ số giọt cần thiết, giải mã nguyên vẹn và kích hoạt tính toán.

---

## Edge Cases

- **Mất gói tin mạng UDP hoàn toàn:** Worker không nhận được bản tin giao việc; Client kích hoạt timeout để giải phóng ký quỹ.
- **Worker nộp kết quả rác / sai Digest:** Hash kết quả thực tế không khớp với chữ ký hoặc xác minh; lệnh chi trả tiền thưởng bị từ chối.
- **Tràn số lượng tiền tệ `NanoSkn`:** Sử dụng phép toán `checked_add` / `checked_sub` nghiêm ngặt của `NanoSkn`, bắt lỗi `LedgerError::Overflow`.
- **Worker cạn kiệt bộ nhớ hoặc gặp lỗi Candle OOM:** Bắt lỗi `CandleError` tường minh thành `ComputeError`, không làm sập tiến trình node chính.

---

## Requirements

### Functional Requirements

- **FR-001**: Hệ thống MUST định nghĩa cấu trúc `ComputeTaskManifest` chứa `task_id`, `client_pubkey`, `opcode`, `required_hardware`, `bounty_nano_skn`, `timeout_epoch`.
- **FR-002**: Tầng wire MUST cung cấp biến thể bản tin `ImpulseBody::Compute(...)` đảm bảo gói tin UDP mã hóa bincode không vượt quá giới hạn cứng `MAX_UDP_DATAGRAM_BYTES = 1200`.
- **FR-003**: Tầng wire MUST hỗ trợ truyền payload tính toán kích thước lớn qua `FountainDroplet` dựa trên đại số tuyến tính $\text{GF}(2)$.
- **FR-004**: Tầng ledger MUST cung cấp cấu trúc `ComputeEscrowVault` để quản lý trạng thái ký quỹ 2 chiều (`Locked`, `Completed`, `Refunded`, `Disputed`).
- **FR-005**: Tầng ledger MUST đảm bảo bất biến bảo toàn tổng lượng `NanoSkn` trong toàn bộ chu trình ký quỹ, chi trả và xử phạt slashing.
- **FR-006**: Tầng cell/compute MUST triển khai `CandleComputeExecutor` hỗ trợ thực thi suy luận nơ-ron cục bộ trên CPU và CUDA/GPU `Device`.
- **FR-007**: Executor MUST trả về `ComputeResult` gồm tensor đầu ra, `result_digest` (BLAKE3) và chữ ký số `Ed25519` của Worker.
- **FR-008**: Tầng agent MUST tích hợp `ComputeCoordinator` kết nối wire, ledger và executor thành chu trình khép kín giữa Client và Worker.
- **FR-009**: Tầng ledger MUST hỗ trợ cấu hình phân chia tiền thưởng `EscrowSplitConfig` giữa Worker chính và Hội đồng kiểm chứng (`Auditor Quorum`).
- **FR-010**: Tầng agent MUST cung cấp `AuditorCoordinator` điều phối việc đấu thầu cọc `AuditorBond`, thu thập chữ ký `Attestation` và kích hoạt thanh toán khi đạt đủ số lượng kiểm chứng viên yêu cầu.

### Key Entities

- **`ComputeTaskManifest`**: Đặc tả kỹ thuật và kinh tế của tác vụ tính toán.
- **`ComputeEscrowRecord`**: Bản ghi trạng thái ký quỹ tiền thưởng trên sổ cái.
- **`CandleComputeTask`**: Cấu trúc tensor đầu vào và thông số mô hình nơ-ron thực thi.
- **`ComputeResult`**: Kết quả đầu ra kèm digest và chữ ký xác thực.

---

## Success Criteria

### Measurable Outcomes

- **SC-001**: $100\%$ các gói tin UDP vận chuyển tác vụ tuân thủ nghiêm ngặt kích thước `<= 1200 bytes`.
- **SC-002**: Chu trình tính toán tensor `Candle` hoàn tất và trả về kết quả chính xác với sai số dấu phẩy động $< 10^{-6}$.
- **SC-003**: Quá trình giải phóng tiền thưởng hoặc hoàn cọc bảo toàn chính xác $100\%$ số lượng `NanoSkn` không thất thoát 1 đơn vị nguyên tử nào.
- **SC-004**: Mọi file mã nguồn mới đều tuân thủ Hiến pháp SkyNet: độ dài $\le 350$ dòng, hàm $\le 40$ dòng, tham số $\le 4$, độ sâu lồng nhau $\le 3$.

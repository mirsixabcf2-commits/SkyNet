# Feature Specification: Hệ Thống Tiền Xử Lý Tự Hành (Turnkey Preprocessing), Presets Danh Mục & Bộ Lệnh CLI Thực Chiến (`skynet` & `skynet-agent`)

## 1. Tổng quan & Mục tiêu Nghiệp vụ (Overview & Goals)

Hiện tại, mạng lưới SkyNet đã xây dựng hoàn chỉnh hạ tầng mạng UDP phi trạng thái, động cơ nơ-ron Candle (`MicroCell`), sổ cái UTXO-DAG và daemon bền vững (`skynet-node`). Tuy nhiên, giữa hạ tầng lõi và trải nghiệm thực tế của người dùng/AI Agent vẫn tồn tại một "khoảng cách thực tế" (Reality Gap):
1. **Thiếu Tầng Tiền Xử Lý Dữ Liệu (Turnkey Preprocessing Engine):** Động cơ tính toán phân tán chỉ nhận tensor toán học dạng số thực thô `Vec<f32>` (`CandleTaskInput`). Người dùng mới, doanh nghiệp hay AI Agent trong đời sống thực tế sở hữu dữ liệu thô (tệp văn bản hợp đồng `.txt`/`.pdf`, bảng tính bảng khảo sát `.csv`, thư mục ảnh chụp sản phẩm `.png`/`.jpg`). Họ không biết cách biến đổi ma trận phức tạp và cần một quy trình tiền xử lý tự động (khử thông tin nhạy cảm PII, chuẩn hóa số liệu, cắt lát văn bản, chuẩn hóa SĐT, mã hóa phong bì).
2. **Thiếu Bộ Cấu Hình Sẵn (Turnkey Presets):** Người mới bắt đầu không thể nhớ hơn 1.000 mã opcode hex phức tạp (`0x0215`, `0x1001`, `0x6001`). Hệ thống cần các "Preset" đóng gói sẵn (như `text-audit`, `csv-clean-normalize`, `image-batch-upscale`, `log-anomaly-detection`) để người dùng chỉ cần chọn 1 nút hoặc AI Agent chỉ cần chỉ định tên preset.
3. **Lệch Pha Giữa Tập Lệnh Tài Liệu (`guide/`) và Bộ Nhị Phân Biên Dịch:** Toàn bộ tài liệu hướng dẫn (`guide/02-user-quickstart.md`, `guide/04-worker-earning-guide.md`, `guide/user/*`, `guide/worker/*`) đều hướng dẫn người dùng gõ lệnh `skynet wallet init`, `skynet compute dispatch ...`, `skynet-agent --auto-bid ...`, nhưng Workspace hiện chỉ biên dịch `skynet-node` và `skynet-dashboard`, khiến người dùng và AI Agent thực thi gặp lỗi `command not found`.
4. **Dashboard Thiếu Bảng Điều Khiển Tiền Xử Lý (Preprocessing Console):** Giao diện web giám sát cần cung cấp màn hình trực quan cho phép người dùng thử nghiệm tải file, bật/tắt các bộ lọc tiền xử lý, xem trước kết quả đã làm sạch và phân phối tác vụ chỉ bằng một cú nhấp chuột.

Tính năng này giải quyết trọn vẹn 4 vấn đề trên, biến SkyNet thành một hệ sinh thái sẵn sàng phục vụ đời sống thực tế.

---

## 2. Yêu cầu Chức năng (Functional Requirements)

### [FR-001] Động Cơ Tiền Xử Lý Dữ Liệu Tự Hành (`Turnkey Preprocessor Engine`)
Hệ thống PHẢI cung cấp pipeline tiền xử lý mô-đun hóa, hỗ trợ các quy tắc biến đổi dữ liệu thô:
1. **Khử Thông Tin Nhạy Cảm (PII Masking / Redaction):**
   - Tự động nhận diện và ẩn danh hóa (masking thành `[REDACTED_PHONE]`, `[REDACTED_EMAIL]`, `[REDACTED_ID]`, `[REDACTED_CARD]`):
     - Số điện thoại di động (chuẩn Việt Nam & Quốc tế).
     - Địa chỉ Email RFC 5322.
     - Số Căn cước công dân (CCCD / CMND 9–12 chữ số).
     - Số thẻ thanh toán tín dụng (Luhn checksum / 16 chữ số).
     - Chuỗi khóa bí mật / API Key nhận diện qua tiền tố thông dụng (`sk_live_`, `AIzaSy`, `Bearer `).
2. **Cắt Lát Văn Bản Trượt (Sliding-Window Chunking):**
   - Chia văn bản lớn thành các đoạn nhỏ có độ dài cố định $W$ (VD: 512 ký tự/từ) với độ gối đầu $\Delta$ (overlap, VD: 64 ký tự/từ) nhằm bảo toàn ngữ cảnh trước khi đưa vào embedding/kiểm toán.
3. **Làm Sạch & Chuẩn Hóa Dữ Liệu Bảng (Tabular CSV/Log Sanitization):**
   - Tự động nhận diện dấu phân cách (`auto-delimiter`: `,`, `;`, `\t`, `|`).
   - Xử lý giá trị khuyết thiếu (Missing Value Imputation): Hỗ trợ các chiến lược `Mean`, `Median`, `Zero`, `DropRow`.
   - Chuẩn hóa số học (Numerical Scaling): Min-Max Scaler $[0.0, 1.0]$ hoặc Z-score Standardization $(x - \mu) / \sigma$.
   - Chuẩn hóa số điện thoại theo chuẩn quốc tế `E.164` (VD: `0912345678` -> `+84912345678`).
4. **Kiểm Tra & Gom Nhóm Thư Mục Ảnh (Image Batching):**
   - Quét thư mục, lọc các tệp ảnh hợp lệ (`.png`, `.jpg`, `.jpeg`, `.webp`), loại bỏ tệp hỏng hoặc tệp rác hệ thống (`.DS_Store`, `Thumbs.db`), nhóm thành các lô (batch) nhỏ sẵn sàng cho worker upscale/xử lý song song.
5. **Mã Hóa Phong Bì Bảo Mật (Envelope Encryption):**
   - Sinh khóa phiên đối xứng 256-bit tạm thời (`ChaCha20-Poly1305` hoặc stream cipher nội bộ), mã hóa dữ liệu nhạy cảm trước khi đóng gói gửi qua mạng P2P, chỉ cho phép worker có attestation hoặc node ủy quyền giải mã.
6. **Chuyển Đổi Sang Dạng Tensor & Phân Mảnh Fountain:**
   - Chuyển đổi dữ liệu số đã làm sạch thành vector số thực `Vec<f32>` tương thích với `CandleTaskInput` của `skynet-cell` và phân mảnh thành các giọt `FountainDroplet` $\le 1200$ Bytes tương thích UDP wire protocol.

### [FR-002] Sổ Bộ Cấu Hình Tiền Xử Lý Sẵn (`Turnkey Presets Registry`)
Hệ thống PHẢI đăng ký danh mục các preset đóng gói sẵn, giúp người dùng mới và AI Agent không cần thiết lập thủ công:
- **`text-audit`**: Preprocessing PII masking + sliding chunking $\to$ Opcode `0x0215` (Private Document Audit).
- **`csv-clean-normalize`**: Delimiter detection + Imputation Mean + MinMax Scaling $\to$ Opcode `0x0210` (Tabular ML / Predictive Analytics).
- **`image-batch-upscale`**: Folder scanning + batching $\to$ Opcode `0x0200` (Super-Resolution AI Upscale).
- **`log-anomaly-detection`**: Text sanitization + Token frequency vectorization $\to$ Opcode `0x0220` (Log Anomaly Detection).
- **`phone-standardize`**: E.164 phone formatting + deduplication $\to$ Opcode `0x0230` (CRM Data Cleansing).
- Mỗi preset cung cấp: `id`, `title`, `description`, `default_opcode`, `suggested_bounty_skn`, `preprocessing_pipeline`.

### [FR-003] File Nhị Phân CLI Dành Cho Người Dùng & AI Dispatcher (`skynet`)
Xây dựng binary thực thi `skynet` tương thích hoàn toàn với các hướng dẫn trong thư mục `guide/`:
1. `skynet wallet init [--path <path>]`: Khởi tạo cặp khóa Ed25519 cục bộ và lưu trữ tệp ví an toàn (`0600`).
2. `skynet wallet balance [--wallet <path>]`: Tra cứu số dư On-chain và BloodPool của ví.
3. `skynet airdrop claim [--wallet <path>]`: Kiểm tra HWID máy tính và nhận Airdrop hạt giống trực tiếp.
4. `skynet opcode list [--domain <filter>]`: Liệt kê danh mục opcode chuẩn hóa theo 12 lĩnh vực.
5. `skynet opcode search <keyword>`: Tìm kiếm opcode theo từ khóa nghiệp vụ (VD: `upscale`, `whisper`, `audit`, `blender`).
6. `skynet opcode info <hex>`: Xem chi tiết yêu cầu VRAM, RAM, Tier và giá sàn của một opcode.
7. `skynet compute presets`: Liệt kê các preset tiền xử lý dành cho người mới và AI Agent.
8. `skynet compute dispatch --input <path> --preset <name> [--opcode <hex>] [--bounty <skn>] [--encrypt]`:
   - Tự động nạp tệp/thư mục đầu vào.
   - Kích hoạt pipeline tiền xử lý tương ứng với preset.
   - Đóng gói dữ liệu và phát lệnh ủy thác tính toán tới mạng lưới SkyNet.

### [FR-004] File Nhị Phân CLI Dành Cho Thợ Đào & AI Worker (`skynet-agent`)
Xây dựng binary thực thi `skynet-agent` hỗ trợ đầy đủ các tham số được tài liệu hóa trong `guide/worker/*`:
1. `skynet-agent --init-worker [--wallet <path>]`: Khởi tạo hồ sơ thợ đào, tự động nhận diện phần cứng (CPU cores, RAM, GPU/VRAM nếu có).
2. `skynet-agent [OPTIONS]`: Chạy tiến trình thợ đào nền:
   - `--listen <port>`: Cổng UDP lắng nghe (mặc định: `8889` hoặc tự chọn).
   - `--auto-bid`: Tự động đấu thầu nhận các tác vụ tính toán phù hợp với năng lực phần cứng.
   - `--tags <tag1,tag2>`: Gắn nhãn chuyên môn (VD: `cuda`, `llm`, `whisper`, `blender`, `vram-12g`).
   - `--min-bounty-rate <rate>`: Mức giá thầu tối thiểu sẵn sàng nhận việc.
   - `--max-vram-gb <gb>`: Giới hạn VRAM sử dụng cho các mô hình AI.
   - `--battery-guard`: Tự động giảm tải hoặc tạm dừng tính toán khi thiết bị chạy bằng pin laptop.
   - `--temp-limit <celsius>`: Tự động ngắt nhận việc khi nhiệt độ phần cứng vượt ngưỡng an toàn.
   - `--schedule <cron_or_time_range>`: Giới hạn khung giờ làm việc (VD: `22:00-07:00` ban đêm).
   - `--wallet-path <path>`: Chỉ định ví thụ hưởng phần thưởng tính toán `NanoSkn`.

### [FR-005] Bảng Điều Khiển Tiền Xử Lý & Preset Console Trên Dashboard (`localhost:8888`)
Mở rộng giao diện web tại `crates/skynet-dashboard/assets/dashboard.html`:
1. **Thẻ Điều Khiển Tiền Xử Lý (Turnkey Preprocessing Studio):**
   - Danh sách chọn nhanh Presets (`text-audit`, `csv-clean-normalize`, `image-batch-upscale`, v.v.).
   - Bộ chọn hộp kiểm (Checkboxes) linh hoạt:
     - `[x] Khử thông tin định danh cá nhân (PII Redaction)`.
     - `[x] Chuẩn hóa số học (Min-Max Scaler 0..1)`.
     - `[x] Chuẩn hóa số điện thoại quốc tế E.164`.
     - `[x] Mã hóa phong bì bảo mật (Envelope Encryption)`.
     - `[x] Cắt lát văn bản ngữ cảnh (Sliding Window Chunking)`.
2. **Khung Xem Trước Trực Quan (Live Interactive Preview):**
   - Vùng nhập văn bản hoặc tải tệp xem trước.
   - So sánh trực quan 2 cột: `Dữ Liệu Thô Đầu Vào (Raw Input)` $\to$ `Dữ Liệu Sau Tiền Xử Lý (Preprocessed Output)`.
   - Hiển thị ma trận thông số kỹ thuật (Kích thước byte gốc, Kích thước sau làm sạch, Số lượng token/tensor `Vec<f32>`, Số giọt Fountain `Droplets` cần gửi).
3. **Nút Phát Lệnh Nhanh & Sao Chép Lệnh CLI (1-Click Action):**
   - Nút `Phát Lệnh Ủy Thác (Dispatch Task)` trực tiếp qua WebSocket/API.
   - Nút `Sao chép tập lệnh CLI (Copy Agent Command)` sinh sẵn câu lệnh CLI tương ứng để AI Agent hoặc terminal có thể chạy lại bất cứ lúc nào.

---

## 3. Tiêu chí Nghiệm thu (Success Criteria)

- **[SC-001] Kiểm Thử Độ Chính Xác Bộ Tiền Xử Lý (Preprocessing Accuracy):**
  - Khử PII ẩn danh 100% các mẫu số điện thoại, email, CCCD và API key chuẩn, không làm biến dạng cấu trúc ngữ nghĩa văn bản.
  - Chuẩn hóa CSV xử lý đúng các giá trị khuyết thiếu và đưa toàn bộ cột số về thang đo $[0.0, 1.0]$.
  - Chuẩn hóa E.164 chuyển đổi chính xác các định dạng số điện thoại Việt Nam (`09...`, `03...`, `84...`) về dạng `+84...`.
- **[SC-002] Tương Thích Nhị Phân CLI Hoàn Toàn Với Tài Liệu `guide/` (Zero "Command Not Found"):**
  - Lệnh `cargo run --bin skynet -- help` và `cargo run --bin skynet-agent -- help` thực thi thành công 100%, hỗ trợ đầy đủ các cờ và subcommand được ghi nhận trong tài liệu hướng dẫn.
  - Người dùng hoặc AI Agent có thể chạy tuần tự: `skynet wallet init` $\to$ `skynet wallet balance` $\to$ `skynet opcode list` $\to$ `skynet compute presets` $\to$ `skynet compute dispatch`.
- **[SC-003] Tương Tác Trực Quan Thời Gian Thực Trên Dashboard (Dashboard Live Console):**
  - Giao diện Dashboard tại cổng `8888` hiển thị đầy đủ thẻ Preset Preprocessing Studio, người dùng nhập dữ liệu mẫu và bấm "Xem Trước Tiền Xử Lý" nhận kết quả làm sạch tức thì trong $\le 50$ms.
- **[SC-004] Tuân Thủ Giới Hạn Vật Lý Của Dự Án (Constitution Compliance):**
  - Mọi file mã nguồn mới/chỉnh sửa tuân thủ nghiêm ngặt giới hạn $\le 250$ dòng (giới hạn cứng 350 dòng), hàm $\le 40$ dòng, tối đa 3-4 tham số, không comment rút gọn (`// TODO`, `// ... code cũ`).
  - Vượt qua 100% các bài kiểm tra `cargo check --workspace`, `cargo clippy --workspace --all-targets` và `cargo test --workspace`.

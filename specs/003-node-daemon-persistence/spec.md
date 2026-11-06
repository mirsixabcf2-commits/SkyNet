# Feature Specification: Đóng Gói Daemon `skynet-node` & Tầng Lưu Trữ Bền Vững (Persistence Engine)

## 1. Tổng quan & Mục tiêu Nghiệp vụ (Overview & Goals)
Hiện tại, toàn bộ các thành phần của SkyNet (`MicroCell`, `UtxoDagLedger`, `SynapticPeerTable`, `CellDna`) đang chạy trong bộ nhớ RAM (`In-Memory`). Khi tiến trình dừng lại, toàn bộ số dư tài sản `NanoSkn`, cấu trúc tiến hóa nơ-ron và danh sách peers đều bị xóa sạch.

Để chuẩn bị cho giai đoạn **Alpha Devnet** và triển khai thực tế trên máy chủ/thiết bị cá nhân (theo [MAINNET_DEPLOYMENT_PLAN.md](file:///Users/pain/DATA/Projects/SkyNet/docs/MAINNET_DEPLOYMENT_PLAN.md)), tính năng này cung cấp:
1. **Tầng Lưu trữ Bền vững (Persistence Engine):** Cơ chế lưu và nạp trạng thái toàn diện (Khóa định danh, Sổ cái UTXO-DAG, Rừng Utreexo, Cấu trúc tế bào AI `MicroCell`, Bảng định tuyến `SynapticPeerTable`) xuống ổ đĩa cục bộ, đảm bảo tính nguyên tử (Atomic commit) và không bị hỏng hóc khi mất điện đột ngột.
2. **Crate Dịch vụ Daemon Chạy Nền (`crates/skynet-node`):** File nhị phân độc lập chạy được trên Linux, macOS và Windows; hỗ trợ cấu hình linh hoạt qua cờ dòng lệnh CLI (`--port`, `--data-dir`, `--tier`, `--seed`, `--wallet`).
3. **Cơ chế Tắt Máy An Toàn (Graceful Shutdown & Crash Recovery):** Bắt các tín hiệu ngắt hệ điều hành (`SIGINT`/`Ctrl+C`), tự động đồng bộ toàn bộ bộ nhớ đệm xuống đĩa trước khi dừng socket UDP.

---

## 2. Yêu cầu Chức năng (Functional Requirements)

### [FR-001] Cấu Hình Tham Số & Giao Diện CLI (`NodeConfig`)
- `skynet-node` PHẢI hỗ trợ các tham số dòng lệnh:
  - `--port <PORT>`: Cổng UDP lắng nghe (mặc định: `8888`).
  - `--data-dir <PATH>`: Thư mục chứa dữ liệu trạng thái (mặc định: `~/.skynet/data`).
  - `--name <NAME>`: Tên định danh sinh học của Node (mặc định: tên máy chủ hoặc `Skynet-Agent`).
  - `--tier <TINY|STANDARD|COLOSSAL>`: Cấp phần cứng sinh học (mặc định: tự động nhận diện hoặc `STANDARD`).
  - `--seed <ADDR>`: Địa chỉ IP:Port của một hoặc nhiều Seed Node để ghép bầy (hỗ trợ khai báo nhiều lần).
  - `--owner <PUBKEY>`: Khóa công khai của ví con người (`HumanWallet`) để thiết lập chính sách lưu ký Soul–Body.
- Hệ thống PHẢI lưu trữ cấu hình có thể tái sử dụng tại `config.json` trong `data_dir`.

### [FR-002] Quản Lý Khóa Định Danh Tự Trị (`NodeIdentity`)
- Khi khởi động lần đầu tại `data_dir`, Node PHẢI tự động sinh một cặp khóa `Ed25519` mới và lưu an toàn tại `node_identity.key` (với quyền hạn truy cập tệp an toàn `0600` trên hệ thống Unix).
- Nếu `node_identity.key` đã tồn tại, Node PHẢI nạp lại chính xác cặp khóa này để bảo toàn định danh `pubkey` và địa chỉ nhận thưởng `NanoSkn`.

### [FR-003] Động Cơ Lưu Trữ Bền Vững Trạng Thái (`NodeStorageEngine`)
- Hệ thống PHẢI hỗ trợ tuần tự hóa (Serialize) và giải tuần tự hóa (Deserialize) toàn bộ trạng thái cốt lõi:
  1. `ledger_snapshot.dat`: Danh sách UTXO hiện hoạt, bộ tích lũy Merkle Utreexo roots, lịch sử các giao dịch đã finalize.
  2. `organism_snapshot.dat`: Tập hợp tế bào `MicroCell`, mã di truyền `CellDna`, thế hệ `generation`, năng lượng nội bào và số dư `BloodPool`.
  3. `peer_table_snapshot.dat`: Danh sách bạn bè đã biết trong bảng định tuyến `SynapticPeerTable`.
  4. `escrow_snapshot.dat`: Các hợp đồng tính toán đang active và trạng thái thợ đào.
- Mọi thao tác ghi tệp PHẢI tuân thủ nguyên tắc **Atomic Write** (ghi ra tệp tạm `.tmp` kèm fsync, sau đó thực hiện rename nguyên tử) nhằm chống hỏng dữ liệu (Data Corruption) nếu tiến trình bị dừng đột ngột giữa chừng.

### [FR-004] Vòng Đời Tiến Trình Daemon & Lắng Nghe UDP (`NodeRuntime`)
- Node khởi tạo Tokio Async Runtime, liên kết socket `UdpSocket` tại `0.0.0.0:<PORT>`.
- Vòng lặp vận hành (Event Loop) xử lý song song:
  1. Lắng nghe và điều phối các gói tin UDP `ImpulsePacket` (`GossipTx`, `SnowballPull/Reply`, `SynapticMicroPay`, `ComputeTask`, `FountainDroplet`).
  2. Nhịp sinh học định kỳ (Biological Heartbeat): Thực thi nhịp nơ-ron Candle `run_biological_tick` sau mỗi khoảng thời gian định trước.
  3. Đồng bộ trạng thái tự động (Auto-Flush): Định kỳ ghi snapshot xuống đĩa sau mỗi chu kỳ Epoch hoặc mỗi N phút.

### [FR-005] Đón Tín Hiệu & Tắt Máy An Toàn (Graceful Shutdown)
- Node PHẢI lắng nghe tín hiệu hủy tiến trình (`tokio::signal::ctrl_c()`).
- Khi nhận tín hiệu tắt máy:
  1. Dừng tiếp nhận gói tin UDP mới.
  2. Đồng bộ tức thời toàn bộ trạng thái sổ cái, năng lượng tế bào và bạn bè xuống ổ đĩa.
  3. Đóng socket UDP sạch sẽ và log thông báo xác nhận an toàn trước khi exit code 0.

### [FR-006] Lệnh Tra Cứu Trạng Thái Cục Bộ (`Status Subcommand`)
- `skynet-node status --data-dir <PATH>`: Đọc trực tiếp trạng thái từ ổ đĩa và in ra màn hình tóm tắt thông số:
  - Node Identity (Pubkey), Cấp phần cứng (Hardware Tier), Uptime/Epoch.
  - Số dư `NanoSkn` (On-Chain + BloodPool).
  - Số lượng tế bào sống (`MicroCell` count phân loại theo 5 loại mô).
  - Số lượng peer đang kết nối và kích thước RAM Utreexo.

---

## 3. Tiêu chí Nghiệm thu (Success Criteria)

- **[SC-001] Bảo Toàn Dữ Liệu Khi Khởi Động Lại (Persistence Roundtrip):**
  - Khởi động node, kích hoạt giao dịch và tiến hóa tế bào, lưu dữ liệu và tắt node.
  - Khởi động lại node từ cùng thư mục `data_dir`: Toàn bộ `pubkey`, số dư `NanoSkn`, số tế bào `MicroCell` và danh sách UTXO khớp chính xác 100%.
- **[SC-002] An Toàn Dữ Liệu Chống Mất Điện Đột Ngột (Crash Resistance):**
  - Mọi thao tác lưu trạng thái sử dụng cơ chế ghi tệp nguyên tử `.tmp -> rename`. Tệp dữ liệu không bao giờ bị rỗng hoặc lỗi cú pháp kể cả khi bị gián đoạn giữa chừng.
- **[SC-003] Kết Nối Đa Node Cục Bộ (Multi-Node Local Mesh):**
  - Hai tiến trình `skynet-node` chạy trên cùng một máy chủ tại cổng `8888` và `8889` có thể trao đổi gói tin UDP và tự động lưu trạng thái vào 2 thư mục dữ liệu riêng biệt.
- **[SC-004] Tuân Thủ Giới Hạn Vật Lý Codebase:**
  - Mọi file mới trong `crates/skynet-node` tuân thủ nghiêm ngặt giới hạn $\le 250$ dòng (cứng 350 dòng), hàm $\le 40$ dòng, không chứa comment rút gọn, pass 100% `cargo check`, `cargo clippy` và `cargo test`.

# Feature Specification: Nghi Thức Khởi Nguyên Genesis (`genesis.json`, `GENESIS_HASH`) & Đục Lỗ NAT UDP P2P Rendezvous

**Feature Directory**: `specs/005-genesis-and-p2p-rendezvous`  
**Created**: 2026-11-06  
**Status**: In-Review  
**Priority**: P1 (Mainnet Readiness Wave 1)

---

## 1. Tổng quan & Mục tiêu Nghiệp vụ (Overview & Goals)

Để đưa mạng lưới SkyNet từ môi trường mô phỏng cục bộ (`skynet-sim`) lên môi trường thử nghiệm diện rộng và Mainnet công khai theo tài liệu [MAINNET_DEPLOYMENT_PLAN.md](file:///Users/pain/DATA/Projects/SkyNet/docs/MAINNET_DEPLOYMENT_PLAN.md) và [NODE_DISCOVERY_AND_BOOTSTRAP.md](file:///Users/pain/DATA/Projects/SkyNet/docs/NODE_DISCOVERY_AND_BOOTSTRAP.md), hệ thống cần giải quyết 2 bài toán nền tảng sống còn:

1. **Khởi Nguyên Sổ Cái (The Genesis Ceremony) & Định Danh Mạng Bất Biến (`GENESIS_HASH`):**
   - Hiện tại, các hằng số tổng cung và phân bổ đang được khởi tạo ngầm định trong bộ nhớ mã nguồn. Để triển khai Mainnet công khai, toàn bộ phân bổ $21$ nghìn tỉ SKN ($2.1 \times 10^{22}$ `nanoSKN`) phải được định nghĩa minh bạch trong tệp cấu hình khởi nguyên `genesis.json`.
   - Hàm băm Blake3 của `genesis.json` tạo thành `GENESIS_HASH` (Network ID) 32-byte duy nhất.
   - Màng lọc tế bào Da (`EpidermalCell`) tại mỗi Node bắt buộc phải kiểm tra `GENESIS_HASH` trên header gói UDP để drop ngay lập tức các gói tin khác mạng, quét cổng bừa bãi hoặc tấn công giả mạo mà không tốn tài nguyên cấp phát bộ nhớ heap (*Zero-Allocation Drop*).

2. **Giao Thức Bắt Tay P2P & Đục Lỗ NAT UDP (Pioneer Rendezvous & Hole Punching):**
   - $95\%$ người dùng và AI Agent chạy node trên máy tính cá nhân hoặc thiết bị nhúng đặt sau Router Wi-Fi/NAT gia đình, không có địa chỉ IP công cộng (Public IP) và không thể mở cổng thủ công (Port Forwarding).
   - Hệ thống cần hiện thực hóa cơ chế **Điểm hẹn Đục lỗ UDP (UDP Punch Rendezvous)** với sự điều phối của Pioneer Seed Node (hoặc máy chủ có IP tĩnh):
     - Node A và Node B gửi gói `PheromoneHello` báo danh tới Seed Node.
     - Seed Node ghi nhận địa chỉ IP/Port công cộng thực tế của A và B, phát gói tin `PunchRendezvous` báo chéo thông tin cho nhau.
     - Node A và Node B bắn đồng thời các xung UDP thăm dò vào cổng ngoại vi của nhau, xuyên thủng 2 lớp NAT Wi-Fi gia đình và thiết lập kết nối P2P trực tiếp trễ siêu thấp.

---

## 2. Yêu cầu Chức năng (Functional Requirements)

### [FR-001] Mô Hình Khởi Nguyên & Tệp Cấu Hình `genesis.json` (`skynet-ledger`)
- Hệ thống PHẢI định nghĩa cấu trúc dữ liệu `GenesisConfig` và `GenesisAllocation`:
  - `network_name`: Chuỗi định danh (VD: `"skynet-mainnet-v1"` hoặc `"skynet-testnet-alpha"`).
  - `total_supply`: Bắt buộc đúng $21{,}000{,}000{,}000{,}000\text{ SKN}$ ($21 \times 10^{21}\text{ nanoSKN}$, kiểu `u128`).
  - `vault_allocation`: $85\%$ ($17.85\text{T SKN}$ / $17{,}850{,}000{,}000{,}000{,}000{,}000{,}000\text{ nanoSKN}$) khóa vào `GenesisHalvingVault` phân phối khai thác sinh học trong 20+ năm.
  - `circulating_allocation`: $15\%$ ($3.15\text{T SKN}$ / $3{,}150{,}000{,}000{,}000{,}000{,}000{,}000\text{ nanoSKN}$) cấp vốn lưu thông ban đầu cho Quỹ phát triển hệ sinh thái, Bootstrap Validators và Quỹ tiếp thị HWID.
  - `initial_utxos`: Danh sách các UTXO khởi nguyên gồm địa chỉ ví Ed25519 và lượng `NanoSkn`.
  - `annual_halving_epochs`: Số epochs mỗi chu kỳ halving (mặc định: `525_600` epochs, tương đương 1 năm với 60s/epoch).
- Bất biến bảo toàn: $\text{vault\_allocation} + \sum \text{initial\_utxos} \equiv \text{total\_supply}$. Nếu sai khác dù chỉ $1\text{ nanoSKN}$, quá trình nạp Genesis PHẢI báo lỗi và dừng tiến trình.

### [FR-002] Khóa Cứng `GENESIS_HASH` & Màng Lọc Tế Bào Da (`skynet-wire`, `skynet-node`)
- Hệ thống PHẢI tính toán mã băm chuẩn hóa 32-byte Blake3 trên toàn bộ nội dung tệp `genesis.json` đã chuẩn hóa (canonical JSON representation) để làm `GENESIS_HASH`.
- Mỗi gói tin giao thức `ImpulsePacket` trên đường truyền UDP ($\le 1200\text{ bytes}$) PHẢI chứa trường `network_id: [u8; 32]`.
- Màng lọc tế bào Da (`EpidermalCell` / `EpidermalFilter`) PHẢI kiểm tra:
  1. `network_id == GENESIS_HASH`: Nếu khác, loại bỏ gói tin ngay lập tức.
  2. `timestamp_sec`: Nằm trong dung sai $\pm 30\text{ giây}$ so với đồng hồ UTC cục bộ (chống tấn công phát lại Anti-Replay).
  3. `signature`: Chữ ký Ed25519 hợp lệ của người gửi trên dữ liệu payload.
- Quá trình kiểm tra trên màng lọc PHẢI thực hiện hoàn toàn trên bộ đệm ngăn xếp (*Stack Buffer*), không cấp phát vùng nhớ Heap (`malloc`), xử lý hoàn tất trong $< 5\mu\text{s}$.

### [FR-003] Giao Thức Khung Tin Bắt Tay `PheromoneHello` & `PunchRendezvous` (`skynet-wire`)
- Hệ thống PHẢI bổ sung hai loại payload định danh trong `WireMessage`:
  1. **`PheromoneHello`**:
     - `sender_pubkey`: Khóa công khai Ed25519 của Node (32 bytes).
     - `internal_addr`: Địa chỉ Socket nội bộ (`SocketAddr`) mà Node đang lắng nghe.
     - `node_tier`: Cấp phần cứng sinh học (`TinyPeripheral`, `StandardNode`, `ColossalOrganism`).
     - `timestamp_sec`: Dấu thời gian Unix thời gian thực (8 bytes).
     - `nonce`: Số ngẫu nhiên 64-bit chống trùng lặp.
  2. **`PunchRendezvous`**:
     - `target_peer_pubkey`: Khóa công khai của Peer cần kết nối (32 bytes).
     - `peer_public_addr`: Địa chỉ Socket công khai thực tế do Seed Node phát hiện (`SocketAddr`).
     - `rendezvous_token`: Mã định danh phiên đục lỗ 64-bit.
     - `expiry_timestamp`: Thời điểm hết hạn phiên đục lỗ.
- Cả hai thông điệp PHẢI đảm bảo kích thước sau khi đóng gói nhị phân và ký số Ed25519 không vượt quá $300\text{ bytes}$ (thỏa mãn trần cứng MTU $\le 1200\text{ bytes}$).

### [FR-004] Điểm Hẹn Đục Lỗ UDP Rendezvous Broker & Bắn Xung Đồng Thời (`skynet-agent`, `skynet-node`)
- **Vai trò Seed Node (Rendezvous Broker):**
  - Khi nhận `PheromoneHello` từ Node A, Seed Node trích xuất địa chỉ IP/Port công khai thực tế (`src_addr`) từ socket UDP.
  - Lưu cặp `(NodeA_Pubkey, NodeA_Public_Addr, Last_Seen)` vào bảng điểm hẹn tạm thời `RendezvousRegistry`.
  - Khi Node A muốn kết nối với Node B (hoặc Seed chỉ định ghép cặp Bầy đàn): Seed Node gửi gói `PunchRendezvous` chứa `B_public_addr` tới A, và gửi `PunchRendezvous` chứa `A_public_addr` tới B.
- **Vai trò NAT Client (Simultaneous UDP Punching):**
  - Ngay khi nhận được `PunchRendezvous` từ Seed Node, cả Node A và Node B lập tức gửi một loạt $3\text{--}5$ xung UDP thăm dò mang `rendezvous_token` tới địa chỉ công khai của đối phương.
  - Khi nhận được xung UDP đầu tiên từ đối phương, hai bên ghi nhận trạng thái đục lỗ thành công và chuyển sang giao thức P2P trực tiếp, lưu địa chỉ vào `SynapticPeerTable`.

### [FR-005] Bộ Lệnh CLI Genesis & Khởi Động `--seed` Cho Node Daemon (`skynet`, `skynet-node`)
- **Lệnh CLI `skynet genesis`:**
  - `skynet genesis generate --output <PATH> [--network <NAME>]`: Sinh file `genesis.json` mặc định đúng $21\text{T SKN}$, in mã băm `GENESIS_HASH`.
  - `skynet genesis verify --path <PATH>`: Kiểm tra tính toàn vẹn toán học và bảo toàn token, in `GENESIS_HASH` tương ứng.
- **Cờ tham số Daemon `skynet-node run`:**
  - Hỗ trợ thêm cờ `--seed <ADDR>` (có thể truyền nhiều lần hoặc danh sách ngăn cách bởi dấu phẩy, VD: `--seed 127.0.0.1:8888,seed2.skynet.network:8888`).
  - Hỗ trợ cờ `--genesis <PATH>`: Nạp file cấu hình Genesis để xác thực `GENESIS_HASH` khi khởi động.

---

## 3. User Scenarios & Acceptance Testing

### User Story 1 - Khởi Tạo & Kiểm Chứng Sổ Cái Genesis Chuẩn Mainnet (Priority: P1)
- **Given**: Người vận hành hoặc nhà phát triển chuẩn bị mạng lưới SkyNet mới.
- **When**: Thực thi lệnh `skynet genesis generate --output genesis.json`.
- **Then**: Hệ thống tạo tệp `genesis.json` với chính xác $21\text{T SKN}$, phân bổ $85\%$ vào `GenesisHalvingVault` và $15\%$ lưu thông ban đầu, in mã băm `GENESIS_HASH` 32-byte Blake3 hex.
- **When**: Kiểm tra lại với `skynet genesis verify --path genesis.json`.
- **Then**: Hệ thống xác nhận bảo toàn $100\%$ `nanoSKN`, tính toán mã băm trùng khớp và trả về exit code `0`.

### User Story 2 - Lọc Gói Tin Sai Mạng & Chống Tấn Công Phát Lại Tại Màng Tế Bào Da (Priority: P1)
- **Given**: Một Node SkyNet đang chạy trên socket UDP cổng `8888` với `GENESIS_HASH` đã định.
- **When**: Kẻ tấn công hoặc máy quét cổng gửi gói tin có `network_id` sai khác hoặc `timestamp_sec` lệch quá $\pm 30\text{s}$.
- **Then**: Màng tế bào Da (`EpidermalFilter`) loại bỏ gói tin trong $< 5\mu\text{s}$, không cấp phát heap, không làm suy giảm hiệu năng CPU của Node.

### User Story 3 - Đục Lỗ NAT UDP P2P Giữa 2 Node Sau Router Wi-Fi Qua Seed Rendezvous (Priority: P2)
- **Given**: Một Seed Node chạy tại cổng công khai, cùng 2 Node A và B nằm sau 2 cổng NAT khác nhau.
- **When**: Cả 2 Node gửi `PheromoneHello` tới Seed Node qua `--seed <SEED_ADDR>`.
- **Then**: Seed Node phát hiện địa chỉ công khai của 2 bên và gửi `PunchRendezvous`.
- **When**: Hai bên nhận `PunchRendezvous` và bắn xung đục lỗ đồng thời.
- **Then**: Kênh P2P UDP trực tiếp giữa A và B được thiết lập thành công mà không cần chuyển tiếp qua Seed Node.

---

## 4. Tiêu Chí Thành Công & Bất Biến Toán Học (Measurable Outcomes)

- **`[SC-001]` Bất biến Genesis:** $\text{Vault} + \text{Circulating} \equiv 21{,}000{,}000{,}000{,}000\text{ SKN}$ ($2.1 \times 10^{22}\text{ nanoSKN}$) bất biến tuyệt đối.
- **`[SC-002]` Kích thước Gói tin:** Toàn bộ bản tin `PheromoneHello` và `PunchRendezvous` khi tuần tự hóa nhị phân $\le 300\text{ bytes}$ (nằm an toàn trong giới hạn UDP MTU $\le 1200\text{ bytes}$).
- **`[SC-003]` Tốc độ Màng lọc Da:** Xác thực `network_id` và `timestamp` hoàn thành trong $< 5\mu\text{s}$ trên stack buffer với 0 cấp phát heap.
- **`[SC-004]` Giới hạn Vật lý Code:** Mọi file mới đều $\le 250\text{ dòng}$ (giới hạn cứng $350\text{ dòng}$), mọi hàm $\le 40\text{ dòng}$, độ lồng $\le 2\text{--}3$ cấp, $100\%$ pass test suite.

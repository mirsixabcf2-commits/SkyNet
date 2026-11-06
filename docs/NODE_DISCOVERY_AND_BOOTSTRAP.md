# Chi Tiết Luồng Bắt Tay, Đục Lỗ NAT UDP & Chống Giả Mạo Khi Mới Triển Khai

> **Đặc tả Kỹ thuật Chuẩn v2.1** — Cơ chế khởi tạo, tự tìm kiếm, đục lỗ tường lửa (NAT Traversal) và bảo vệ màng sinh học cho SkyNet từ giai đoạn mạng sơ khai tới quy mô phân tán toàn cầu.

---

## 1. Luồng Bắt Tay & Đục Lỗ NAT UDP Giữa 2 Máy Sau Router Gia Đình

Khi mạng lưới mới bắt đầu, phần lớn người dùng chạy Node trên máy tính cá nhân hoặc thiết bị nhúng đặt sau các Router Wi-Fi gia đình (NAT/Firewall). Các máy này không có địa chỉ IP công cộng (Public IP) và không mở cổng thủ công (Port Forwarding).

SkyNet thiết lập kết nối P2P trực tiếp thông qua **Điểm hẹn Đục lỗ UDP (UDP Punch Rendezvous)** với sự hỗ trợ của Pioneer Seed VPS:

```
[Máy A - Sau Wi-Fi Hà Nội]        [Pioneer Seed VPS]        [Máy B - Sau Wi-Fi Sài Gòn]
            |                              |                              |
   1. UDP PheromoneHello ----------------->|                              |
      (Ký Ed25519 + GenesisHash)           |<---------------- 2. UDP PheromoneHello
            |                              |      (Ký Ed25519 + GenesisHash)
            |                              |                              |
            |<-- 3. Báo IP ngoại của B ----+---- Báo IP ngoại của A ----->|
            |    (UDP PunchRendezvous)     |     (UDP PunchRendezvous)    |
            |                              |                              |
   4. Bắn xung UDP trực tiếp xuyên thủng cả 2 lớp NAT Wi-Fi gia đình!     |
            |<==================== Kết nối P2P UDP Trực tiếp ============>|
```

### Chi tiết 4 Bước Đục Lỗ:
1. **Bước 1 & 2 (PheromoneHello):** Máy A và Máy B khi khởi động cùng gửi gói tin `PheromoneHello` tới Pioneer Seed VPS qua cổng UDP `39210`. Router Wi-Fi của A và B tự động mở một cổng NAT ngoại vi tạm thời (NAT Binding Mapping) để đón gói tin phản hồi từ VPS.
2. **Bước 3 (PunchRendezvous):** Pioneer Seed VPS nhận diện địa chỉ công cộng thực tế của cả hai bên (`A_public_ip:A_port` và `B_public_ip:B_port`). VPS lập tức gửi gói tin `PunchRendezvous` báo địa chỉ của B cho A, và địa chỉ của A cho B.
3. **Bước 4 (Đục lỗ đồng thời):** Ngay khi nhận được lệnh từ VPS, cả Máy A và Máy B lập tức bắn đồng thời các gói xung UDP thăm dò vào cổng ngoại vi của nhau. Router của A thấy A vừa chủ động gửi gói đến B nên mở đường cho gói từ B đi vào, và ngược lại. Hai lớp NAT Wi-Fi gia đình bị xuyên thủng hoàn toàn, thiết lập đường truyền UDP P2P trực tiếp trễ siêu thấp mà không còn tốn băng thông của VPS.

---

## 2. Chống Kết Nối Nhầm Mạng & Loại Bỏ Node Giả Mạo Ngay Gói Đầu Tiên

Để ngăn chặn hoàn toàn các cuộc tấn công quét cổng bừa bãi, giả mạo IP (IP Spoofing) hoặc kết nối nhầm giữa các mạng thử nghiệm khác nhau, SkyNet áp dụng cơ chế xác thực ngay tại màng tế bào:

```
+-----------------------------------------------------------------------------+
|                      CẤU TRÚC GÓI IMPULSEPACKET (<= 1200B)                  |
|                                                                             |
| [Header 16B]  : Magic [0x53, 0x4B, 0x4E, 0x01] | Protocol Version (0x0201)   |
| [Network ID]  : GENESIS_HASH (32B Blake3 - Khóa cứng luật 21 nghìn tỉ SKN)  |
| [Security]    : Sender Ed25519 Pubkey (32B) | Timestamp (8B) | Nonce (8B)   |
| [Payload]     : PheromoneHello / Rendezvous / SynapticVoucher               |
| [Signature]   : Ed25519 Digital Signature (64B ký trên toàn bộ payload)     |
+-----------------------------------------------------------------------------+
```

### Cơ chế Màng lọc Tế bào Da (Epidermal Membrane Filtering)
- **Kiểm tra `GENESIS_HASH`:** Gói tin bắt buộc phải chứa chuỗi băm Blake3 của khối Genesis khởi nguyên (mã hóa luật cung bất biến 21 nghìn tỉ SKN và tham số đồng thuận). Nếu sai khác 1 bit, gói tin bị drop ngay lập tức.
- **Chống Phát lại (Anti-Replay Protection):** `timestamp_sec` bắt buộc phải nằm trong dung sai $\pm 30\text{ giây}$ so với đồng hồ địa phương. Các gói tin quá cũ hoặc trong tương lai bị từ chối ngay.
- **Xác thực Chữ ký Ed25519 Tức thời:** Mọi gói tin đều được ký bởi khóa định danh của Agent.
- **Zero-Allocation Drop:** Tế bào Da (`EpidermalCell`) giải mã và kiểm tra tính toàn vẹn trực tiếp trên bộ đệm socket stack (không cấp phát bộ nhớ heap `malloc`). Các gói tin tấn công DoS/Spam bị loại bỏ trong $< 5\mu\text{s}$, giữ cho CPU luôn mát.

---

## 3. Ba Kịch Bản Triển Khai Thực Tế: Từ Dễ Đến Tự Trị Tuyệt Đối

Hệ thống hỗ trợ 3 kịch bản vận hành linh hoạt tùy theo điều kiện hạ tầng:

```
[Kịch bản 1: Mạng LAN]      [Kịch bản 2: Genesis WAN]       [Kịch bản 3: Sinh Tồn DHT]
  Cắm chung Wi-Fi               1-2 Pioneer VPS                 Không Server / Không Domain
  UDP Multicast 100ms           DNS TXT _skynet...              BitTorrent Mainline DHT
        |                              |                                  |
        v                              v                                  v
 (Tự ghép bầy < 1s)           (Tự tra DNS & đục NAT)          (Bất khả xâm phạm)
```

### Kịch bản 1: Chạy Thử Nội Bộ (Phòng Lab / Gia Đình / Văn Phòng)
- **Cách thức:** Người dùng không cần cấu hình bất kỳ tham số nào. Chỉ cần khởi chạy binary trên 2 đến 10 máy tính cắm chung mạng LAN hoặc Wi-Fi.
- **Cơ chế:** Tầng UDP Multicast (`239.83.75.78:39210`) và Broadcast (`255.255.255.255:39210`) tự động phát xung `PheromoneBeacon`. Các máy tự phát hiện ra nhau và ghép thành cụm hoạt động trong chưa đầy **1 giây**.

### Kịch bản 2: Triển Khai Thực Tế Giai Đoạn Genesis (Khuyên Dùng)
- **Cách thức:** Cộng đồng hoặc nhóm phát triển dựng **1–2 VPS IP tĩnh siêu nhẹ** (cấu hình tối thiểu 1 vCPU, 512MB RAM) làm Pioneer Seed Node.
- **Cấu hình DNS:** Trỏ bản ghi DNS TXT tại tên miền dự án:
  ```text
  _skynet.yourdomain.org.  IN  TXT  "skn://<ed25519_pubkey>@<public_ip>:39210"
  ```
- **Trải nghiệm Người dùng:** Mọi người tải phần mềm về chỉ việc bấm khởi chạy. Agent tự động truy vấn DNS TXT để lấy địa chỉ Pioneer VPS, thực hiện bắt tay và đục lỗ NAT Wi-Fi để nối vào bầy P2P toàn cầu.

### Kịch bản 3: Chế Độ "Sinh Tồn Tuyệt Đối" (Không VPS Riêng, Không Tên Miền)
- **Đặc điểm:** Dành cho tình huống tên miền bị chặn DNS, VPS bị kiểm duyệt hoặc mất kết nối hạ tầng tập trung.
- **Cơ chế Mainline DHT:**
  - Các Node tính toán mã định danh mục tiêu:
    $$\text{target} = \text{BLAKE3}(\text{"SKYNET\_GENESIS\_V1"})$$
  - Node công bố và tìm kiếm danh sách `IP:Port` trực tiếp trên mạng **BitTorrent Mainline DHT** toàn cầu (mạng lưới sẵn có với hàng chục triệu node UDP phân tán).
  - Không một tổ chức hay quốc gia nào có thể đóng cửa hay vô hiệu hóa được cơ chế này.

---

## 4. Xử Lý 10% Mạng 4G / CGNAT Khắt Khe Không Thể Đục Lỗ

Khoảng 10–15% kết nối mạng di động 4G/5G hoặc mạng doanh nghiệp sử dụng **NAT Đối Xứng (Symmetric NAT)**. Ở loại NAT này, router thay đổi cổng ngoại vi ngẫu nhiên cho mỗi IP đích khác nhau, khiến kỹ thuật đục lỗ NAT trực tiếp giữa hai máy bị thất bại.

SkyNet xử lý bài toán này thông qua **Cơ chế Hạch Ngoại Biên (Ganglia Relay)**:

```
[Agent 4G (Symmetric NAT)]  =====(Đường hầm UDP chủ động)=====> [Ganglia Node (IP Công cộng)]
                                                                          |
                                                                          | (Chuyển tiếp nội cụm)
                                                                          v
[Agent Khác Trong Mạng]     <====(Đường truyền trực tiếp)====== [Mạng Lưới SkyNet]
```

- **Nhận diện Tự động:** Khi gửi gói tin đục lỗ tới đối tác sau 3 chu kỳ thử lại ($1.5\text{s}$) mà không nhận được `PongProof`, Node tự động xác định đường truyền bị chặn bởi Symmetric NAT.
- **Kích hoạt Đường hầm Ganglia Relay:** Node 4G chủ động duy trì 2 kết nối gửi xung ra ngoài tới 2 Node lớn có IP công cộng trong cùng `ClusterPrefix`.
- **Bảo mật Đầu cuối:** Gói tin chuyển tiếp qua Ganglia Node vẫn được mã hóa và ký Ed25519 nguyên gốc. Ganglia Node chỉ đóng vai trò chuyển tiếp xung nhị phân mà không thể đọc lén hay can thiệp vào số dư `nanoSKN` hay trạng thái mô hình AI. Nhờ đó, các Node chạy trên điện thoại hoặc modem 4G vẫn tham gia kiếm tiền bình thường mà không bị cô lập.

---

## 5. Quy Tắc Genesis Sharding Bảo Toàn Mật Độ Đồng Thuận ($N < 64$)

Để thuật toán đồng thuận xác suất Snowball hoạt động chính xác với tham số $k = 10$ và $\alpha = 0.8$:
- **Quy tắc cứng:**
  $$\text{Nếu } N_{\text{peers}} < 64 \implies \text{Cố định } \text{prefix\_len} = 0 \quad (\text{Mọi Node cùng chung Cụm } 0b*)$$
- Toàn bộ các Node trong giai đoạn Genesis đều lắng nghe và bỏ phiếu chung trong một không gian đồng thuận duy nhất, ngăn ngừa hiện tượng phân mảnh cụm khi số lượng thành viên ban đầu còn ít.

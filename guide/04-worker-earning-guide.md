# Biến Máy Tính Nhàn Rỗi Thành Nguồn Thu Nhập: Hướng Dẫn Trở Thành Worker Node

---

## 1. Bạn Đang Lãng Phí Năng Lực Phần Cứng Mỗi Ngày

Nếu bạn sở hữu:
- Một chiếc **PC Gaming** với card đồ họa NVIDIA (RTX 3060, 3070, 4070, 4080, 4090).
- Một chiếc **MacBook hoặc Mac Studio** trang bị chip Apple Silicon (M1, M2, M3, M4) với bộ nhớ hợp nhất (Unified Memory) băng thông cao.
- Một máy chủ (Server/VPS) nhàn rỗi có nhiều luồng CPU.

Thực tế là: **Hơn 70% đến 85% thời gian trong ngày**, chiếc máy tính của bạn chỉ dùng để lướt web, soạn thảo văn bản, hoặc tắt màn hình đi ngủ. Năng lực phần cứng hàng ngàn USD đang nằm im lãng phí.

**SkyNet cho phép bạn "cho thuê" từng mili-giây nhàn rỗi đó một cách an toàn để tích lũy `NanoSkn`!**

```
+----------------------------------------------------------------------------------------------------+
|                                    TẠI SAO CHỌN ĐÀO HỮU ÍCH SKYNET?                                |
+------------------------------------+---------------------------------------------------------------+
| ĐÀO COIN CŨ (PoW - Bitcoin/Kaspa)  | ĐIỆN TOÁN HỮU ÍCH SKYNET (PoUW)                               |
+------------------------------------+---------------------------------------------------------------+
| Băm hàm SHA256/kHeavyHash vô nghĩa | 100% điện năng phục vụ bài toán AI, render, khoa học thực tế |
| Tiêu hao điện khủng khiếp, hại card| Chỉ chạy khi có task thật, tự hạ xung khi nhàn rỗi            |
| Bị các trại trâu cày ASIC đè bẹp   | Máy tính cá nhân, Mac M-series cực kỳ cạnh tranh              |
| Bị dư luận chỉ trích phá hoại môi trường | Đóng góp trực tiếp cho sự phát triển của công nghệ AI nhân loại|
+------------------------------------+---------------------------------------------------------------+
```

---

## 2. Máy Tính Của Tôi Có Được Bảo Vệ An Toàn Không?

Nỗi lo lớn nhất của người dùng khi cho thuê máy tính là: **"Liệu người ta có gửi virus, mã độc hoặc đào bới tệp cá nhân trong máy tôi không?"**

SkyNet được thiết kế theo nguyên lý **Zero-Trust Security (Bảo mật Không Tin Cậy Cấp Phần Cứng)**:
1. **Cô lập Bộ nhớ (Sandbox Isolation):**
   - Động cơ nơ-ron `CandleComputeExecutor` chỉ nhận vào mảng số thực tensor (`Vec<f32>`).
   - Mã thực thi không có quyền truy cập vào hệ thống tệp tin (File System) của bạn.
   - Không có quyền quét ảnh, không đọc tài liệu, không truy cập cookie trình duyệt.
2. **Không Mở Cổng Trái Phép (No Rogue Network Sockets):**
   - Môi trường tính toán không có quyền mở kết nối TCP/UDP tùy tiện ra ngoài Internet để phát tán spam hay tấn công DDoS.
3. **Bảo Vệ Phần Cứng & Kiểm Soát Nhiệt Độ:**
   - Bộ lập lịch kiểm soát thời gian chạy (`max_ticks`). Nếu một tác vụ chạy quá thời hạn quy định, hệ thống tự động ngắt ngay lập tức để tránh làm GPU bị quá tải hoặc quá nhiệt.

---

## 3. Hướng Dẫn Bắt Đầu Chạy Worker Trong 3 Phút

### Bước 1: Tải Về & Cấu Hình Khóa Ví Thợ
Khởi tạo danh tính máy thợ và tạo ví nhận thưởng `NanoSkn`:

```bash
# Clone mã nguồn SkyNet và biên dịch bản Release tối ưu
git clone https://github.com/skynet-protocol/skynet.git
cd skynet
cargo build --release --workspace

# Khởi tạo khóa Worker Node
./target/release/skynet-agent --init-worker
```

### Bước 2: Kích Hoạt Worker Chạy Tự Động Theo Phần Cứng
Hệ thống sẽ tự động quét và lựa chọn động cơ tối ưu:
- Nếu phát hiện **NVIDIA GPU:** Tự động kích hoạt backend CUDA.
- Nếu phát hiện **Apple Silicon (Mac M1/M2/M3/M4):** Tự động kích hoạt Metal Performance Shaders.
- Nếu không có GPU: Tự động chạy chế độ CPU đa luồng tối ưu.

```bash
# Khởi chạy Worker Node kết nối vào mạng lưới UDP Mesh
./target/release/skynet-agent \
  --listen 0.0.0.0:8888 \
  --min-bounty-rate 100 \
  --max-vram-gb 8 \
  --auto-bid
```

**Các tham số thông minh:**
- `--min-bounty-rate 100`: Chỉ nhận những tác vụ trả thù lao tối thiểu từ 100 nanoSKN trở lên.
- `--max-vram-gb 8`: Chỉ cấp phát tối đa 8GB VRAM cho SkyNet, phần còn lại giữ để bạn chơi game hoặc làm việc.
- `--idle-only`: Chỉ tự động nhận việc khi bàn phím và chuột không hoạt động trong 5 phút.

---

### Bước 3: Thợ Đào Có Cần Cài Đặt Môi Trường Hay Phần Mềm Gì Khác Không?

> **HOÀN TOÀN KHÔNG!** Bạn không cần cài Python, PyTorch, CUDA Toolkit, Blender, Docker hay bất kỳ thư viện chuyên biệt nào.

Hệ thống SkyNet vận hành theo cơ chế **Zero-Touch Provisioning (Tự động hóa tuyệt đối)**:
1. **Chuẩn hóa qua Giao thức MCP (Model Context Protocol):** Toàn bộ các tác vụ tính toán được định nghĩa chuẩn hóa thành các công cụ MCP (`skynet://vision.upscale.real_esrgan`, `skynet://audio.transcribe.whisper`...).
2. **Tự động tải Capsule Runtime:** Khi máy của bạn trúng thầu một nhiệm vụ, `skynet-agent` sẽ **tự động tải ngầm** gói thực thi độc lập (**Capsule**) tương ứng qua mạng P2P Fountain UDP hoặc CDN phi tập trung.
3. **Thực thi trong Sandbox MCP:** Capsule được đóng gói sẵn mọi thư viện phụ thuộc siêu nhẹ (15MB – 80MB), chạy cô lập hoàn toàn bên trong sandbox, không chạm vào dữ liệu cá nhân của bạn, và tự động giải phóng VRAM ngay khi tính toán xong.

---

## 4. Bảng Ước Tính Lợi Nhuận & Tiêu Thụ Điện Năng

| Cấu hình thiết bị | Công suất tiêu thụ trung bình | Loại tác vụ tối ưu trên SkyNet | Ước tính thu nhập / ngày |
| :--- | :--- | :--- | :--- |
| **Nvidia RTX 4090 (24GB)** | 250W – 350W khi full tải | LoRA Fine-Tuning, Video Diffusion, 3DGS | **50,000 – 150,000 nanoSKN** |
| **Nvidia RTX 3060 (12GB)** | 110W – 160W | Micro-Neural Inference, Upscaling ảnh | **15,000 – 40,000 nanoSKN** |
| **MacBook Pro M3 Max (36GB)** | 45W – 70W (Cực kỳ tiết kiệm điện) | Large Context LLM, Whisper v3 Audio | **30,000 – 80,000 nanoSKN** |
| **CPU AMD Ryzen 9 / Intel i9** | 65W – 120W | zk-STARK Prover, Monte Carlo, Fuzzing | **10,000 – 25,000 nanoSKN** |

> 💡 **Mẹo Tối Ưu Lợi Nhuận:**
> Máy tính Apple Silicon M-series có hiệu suất năng lượng trên mỗi Watt (Perf/Watt) cao nhất thế giới hiện nay. Chạy SkyNet trên Mac M2/M3/M4 hầu như không tốn điện, máy chạy êm ru không tiếng ồn và tạo ra dòng tiền ròng cao nhất sau khi trừ tiền điện.

---

## 5. Rút Tiền & Chuyển Đổi Thu Nhập

Số `NanoSkn` kiếm được được cộng dồn trực tiếp vào ví `Sovereign Wallet` của bạn sau mỗi tác vụ hoàn tất. Bạn có thể:
1. **Dùng để thuê ngược lại:** Dùng chính số tiền đó để nhờ các máy tính khác giải quyết bài toán của bạn khi cần.
2. **Chuyển giao P2P:** Gửi `NanoSkn` cho bạn bè hoặc các Agent khác với phí chuyển mạng gần như bằng 0.
3. **Thanh khoản:** Quy đổi sang các đồng tiền kỹ thuật số khác hoặc tiền pháp định (VND/USD) thông qua các kênh thanh khoản cộng đồng phi tập trung.

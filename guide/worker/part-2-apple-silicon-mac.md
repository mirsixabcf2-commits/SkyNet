# Tuyển Tập Worker 021 – 040: Hệ Sinh Thái Apple Silicon Mac (M1/M2/M3/M4)

Tài liệu này dành riêng cho người dùng máy tính Apple Mac (MacBook Air/Pro, Mac mini, Mac Studio, Mac Pro). 

Kiến trúc **Bộ nhớ Thống nhất (Unified Memory Architecture - UMA)** của Apple Silicon mang lại lợi thế độc nhất vô nhị trên mạng lưới SkyNet: một chiếc Mac Studio nhỏ gọn có thể sở hữu tới 128GB hoặc 192GB VRAM khả dụng cho mô hình AI với mức tiêu thụ điện chỉ bằng một phần mười so với dàn PC x86 truyền thống, hoạt động hoàn toàn êm ái 24/7.

---

## Bảng Tổng Hợp Thông Số & Lệnh Worker 021 – 040

| Case | Thiết Bị Apple | Bộ Nhớ Thống Nhất | Tác Vụ Đề Xuất (Opcodes) | Lệnh Khởi Chạy Nhanh | Lợi Thế Cạnh Tranh |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **021** | Mac mini M2 Pro | 32GB Unified | `0x0104`, `0x0103` (Llama-3 14B, Whisper) | `skynet-agent --auto-bid --backend metal --tags "apple-silicon,unified-32g"` | Tiêu thụ chỉ 35W, chạy 24/7 không quạt rít |
| **022** | Mac Studio M2 Ultra| 192GB Unified | `0x0104`, `0x0106` (DeepSeek-V2, LLM 70B) | `skynet-agent --auto-bid --backend metal --tags "huge-ram,vram-192g,llm-70b"`| Chạy mô hình khổng lồ mà PC thường không kham nổi |
| **023** | MacBook Pro M3 Max | 64GB Unified | `0x0105`, `0x0201` (CogVideoX, Blender Metal)| `skynet-agent --auto-bid --backend metal --charge-only --tags "metal3"` | Sạc qua đêm tự cày tiền, sáng rút sạc tự ngắt |
| **024** | MacBook Air M2 | 16GB Unified | `0x0103`, `0x0402` (Whisper, OCR Surya) | `skynet-agent --auto-bid --fanless-mode --temp-limit 72` | 100% không tiếng ồn, không quạt tản nhiệt |
| **025** | Mac Studio M1 Max | 64GB Unified | `0x0405`, `0x0409` (HNSW Index, KG Triples) | `skynet-agent --auto-bid --tags "apple,vector-index,high-bandwidth"` | Băng thông RAM 400 GB/s cực nhanh |
| **026** | Mac Pro M2 Ultra | 128GB Unified | `0x0305`, `0x0304` (WRF Weather, Monte Carlo)| `skynet-agent --auto-bid --tags "apple,hpc,monte-carlo"` | 24 nhân CPU + 76 nhân GPU Metal |
| **027** | MacBook Pro M1 Pro | 32GB Unified | `0x0203` (Mã hóa ProRes / AV1 4K) | `skynet-agent --auto-bid --tags "apple-media-engine,video"` | Bộ giải mã phần cứng Apple Media Engine |
| **028** | Mac mini M4 mới | 24GB Unified | `0x0205` (Phân loại ảnh NPU, YOLOv10) | `skynet-agent --auto-bid --backend ane --tags "neural-engine-m4"` | Nhân NPU thế hệ mới 38 TOPS |
| **029** | MacBook Pro M2 Max | 96GB Unified | `0x040C`, `0x0403` (Reconciliation, MinHash) | `skynet-agent --auto-bid --tags "high-memory,data-pipe"` | Xử lý file lớn hoàn toàn trên RAM không swap |
| **030** | Mac mini M1 16GB cũ| 16GB Unified | `0x0001` (Auditor Node xác thực hash/chữ ký)| `skynet-agent --auditor-mode --stake 5000 --tags "auditor,low-power"`| Cụm kiểm chứng mini chỉ tốn 50k tiền điện/tháng |
| **031** | iMac M3 | 24GB Unified | `0x040D` (OCR chữ Hán Nôm, Thư viện số) | `skynet-agent --auto-bid --tags "apple,ocr,nlp"` | Màn hình tắt (Clamshell/Sleep display) máy vẫn chạy |
| **032** | Cụm 3x Mac mini M2 | 3x 16GB LAN 10G | Nhận phân tán nhiều batch cùng lúc | `skynet-agent --cluster-mode --master 192.168.1.10` | Nông trại tính toán Apple yên tĩnh trong phòng khách |
| **033** | MacBook Pro M4 Pro | 48GB Unified | `0x0201`, `0x040B` (Ray Tracing Metal 3) | `skynet-agent --auto-bid --tags "metal-rt,raytracing"` | Nhân dò tia phần cứng Metal 3 cực mạnh |
| **034** | Mac Studio M1 Ultra| 64GB Unified | `0x040A` (Demucs Tách Stem Âm Thanh) | `skynet-agent --auto-bid --tags "apple,audio-batch"` | Xử lý song song 8 luồng âm thanh liên tục |
| **035** | MacBook Pro M3 Pro | 36GB Unified | `0x050C` (Tổng hợp tin tức LLM 128k context) | `skynet-agent --auto-bid --tags "apple,long-context"` | Bộ nhớ đệm ngữ cảnh LLM cực sâu |
| **036** | Mac mini M2 8GB Base| 8GB Unified | Chuyển tiếp UDP Fountain Droplets | `skynet-agent --relay-only --bandwidth-limit 100mbps` | Trạm chuyển tiếp gói tin nhận phí hoa hồng relay |
| **037** | MacBook Air M3 | 24GB Unified | Micro-jobs khi gập máy (Clamshell mode) | `skynet-agent --auto-bid --clamshell-enable` | Tận dụng giờ nghỉ trưa văn phòng |
| **038** | Mac Pro Xeon cũ + eGPU| 96GB + Radeon | `0x0303` (CFD OpenFOAM đa luồng) | `skynet-agent --auto-bid --tags "cpu-hpc,opencl"` | Tái sinh máy trạm Mac cũ làm node CPU |
| **039** | Mac Studio M2 Max | 32GB Unified | `0x0502` (Quỹ đạo đàn drone 500 chiếc) | `skynet-agent --auto-bid --tags "apple,trajectory-gen"` | Tính toán hình học động 4D |
| **040** | MacBook Pro M1 Max | 32GB Unified | `0x050E` (Smart Contract Fuzzing) | `skynet-agent --auto-bid --idle-trigger 10m` | Tự kích hoạt khi máy không có phím chuột 10 phút |

---

## Các Tình Huống Vận Hành Đỉnh Cao Trên macOS

### Case 022: Mac Studio 192GB Unified Memory – Trùm Đón Sóng LLM Khổng Lồ
- **Tại sao Mac Studio 192GB lại là "vũ khí bí mật"?**
  Trên thị trường PC, để có 192GB VRAM, bạn cần 8 chiếc RTX 4090 (ngốn 3.500W điện và bo mạch chủ server đắt đỏ) hoặc 2 card NVIDIA A100 80GB (giá hơn 30.000 USD). Một chiếc Mac Studio M2 Ultra 192GB chỉ có giá khoảng 7.000 USD và ăn vỏn vẹn 120W điện.
- **Cấu hình khai thác:**
  ```bash
  skynet-agent \
    --listen 0.0.0.0:8888 \
    --auto-bid \
    --backend metal \
    --tags "huge-ram,vram-192g,llm-70b,deepseek" \
    --min-bounty-rate 6.0 \
    --wallet-path ~/Library/Application\ Support/SkyNet/wallet.json
  ```
- **Hiệu quả:** Tự động bắt trọn các job suy luận LLM ngữ cảnh siêu dài (128.000 tokens) và mô hình DeepSeek-V2 MoE. Nút thu về từ 80 đến 150 SKN mỗi ngày mà văn phòng làm việc vẫn hoàn toàn tĩnh lặng.

### Case 023 & 024: MacBook Khi Ngủ & Cơ Chế Fanless Mode
- **Khái niệm:** Người dùng MacBook muốn cắm máy qua đêm để kiếm tiền nhưng sợ tiếng ồn quạt làm phiền giấc ngủ hoặc sợ máy bị nóng.
- **Thiết lập:**
  ```bash
  # Tùy chọn êm dịu tuyệt đối, không bật quạt quá 2000 RPM
  skynet-agent \
    --auto-bid \
    --backend metal \
    --charge-only \
    --temp-limit 70 \
    --fan-profile silent
  ```
  Agent kích hoạt bộ lập lịch tiến trình của macOS (Grand Central Dispatch / QoS background) để điều phối tính toán vào các nhân tiết kiệm điện (Efficiency Cores) và nhân đồ họa Metal ở xung nhịp tối ưu năng lượng.

### Case 030: Biến Mac mini M1 Cũ Thành "Auditor Vĩnh Cửu"
- **Tình huống:** Người dùng nâng cấp lên máy mới, chiếc Mac mini M1 16GB cũ bị bỏ xó trong ngăn kéo.
- **Giải pháp:** Cắm một sợi dây mạng LAN và một dây nguồn, đặt máy ở góc nhà hoặc kệ sách không cần gắn màn hình (Headless mode).
- **Lệnh chạy nút Kiểm toán:**
  ```bash
  skynet-agent \
    --auditor-mode \
    --stake-bond 5000000000000 \
    --listen 0.0.0.0:8888 \
    --daemon
  ```
  Chiếc Mac mini sẽ liên tục tham gia vào Ban kiểm chứng (Auditor Quorum), tải các bằng chứng băm kết quả từ các Worker khác, chạy kiểm thử chéo và nhận 15% hoa hồng chia đều. Chi phí tiền điện mỗi tháng chỉ tương đương một ly cà phê (khoảng 35.000 – 50.000 VNĐ).

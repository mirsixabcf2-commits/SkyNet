# Tuyển Tập Worker 001 – 020: Máy Tính Chơi Game & PC Đồ Họa Cá Nhân (Consumer Gaming Rig)

Tài liệu này hướng dẫn chi tiết người dùng sở hữu máy tính chơi game cá nhân (Desktop / Laptop card rời NVIDIA GeForce RTX) cách cấu hình phần mềm `skynet-agent` để tự động biến cỗ máy giải trí thành một cỗ máy in tiền thụ động bằng cách xử lý các tác vụ AI, Render đồ họa và bóc tách dữ liệu khi máy đang nhàn rỗi.

---

## Bảng Tổng Hợp Thông Số & Lệnh Worker 001 – 020

| Case | Phần Cứng Tiêu Biểu | VRAM | Tác Vụ Chủ Lực (Opcodes) | Lệnh Khởi Chạy Nhanh | Thu Nhập Ước Tính / Ngày |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **001** | RTX 3060 12GB | 12GB GDDR6 | `0x0101` (Real-ESRGAN Upscale) | `skynet-agent --auto-bid --tags "cuda,upscale,vram-12g" --min-rate 1.0` | 8 – 15 SKN |
| **002** | RTX 4070 12GB | 12GB GDDR6X | `0x0104`, `0x0103` (Llama-3 8B, Whisper) | `skynet-agent --auto-bid --tags "cuda,llm,whisper" --power-cap 180` | 15 – 25 SKN |
| **003** | RTX 4090 24GB | 24GB GDDR6X | `0x0102`, `0x0105` (FLUX.1, SDXL, CogVideo) | `skynet-agent --auto-bid --tags "cuda,heavy-ai,flux,vram-24g"` | 45 – 80 SKN |
| **004** | Laptop RTX 4060 Mobile | 8GB GDDR6 | `0x0103`, `0x0402` (Whisper, Surya OCR) | `skynet-agent --auto-bid --battery-guard --temp-limit 75` | 6 – 12 SKN |
| **005** | RTX 3080 10GB | 10GB GDDR6X | `0x0201` (Blender Cycles Render) | `skynet-agent --auto-bid --tags "blender,cycles,optix"` | 18 – 30 SKN |
| **006** | RTX 2060 Super 8GB | 8GB GDDR6 | `0x0402`, `0x0408` (OCR, Moderation) | `skynet-agent --auto-bid --tags "cuda,ocr,fp16"` | 5 – 9 SKN |
| **007** | RTX 3070 8GB | 8GB GDDR6 | `0x040A` (Demucs v4 Audio Separation) | `skynet-agent --auto-bid --tags "cuda,audio,demucs"` | 10 – 18 SKN |
| **008** | Dual RTX 3090 (NVLink) | 48GB GDDR6X | `0x0106`, `0x0301` (LoRA Fine-tuning, AlphaFold)| `skynet-agent --auto-bid --devices "cuda:0,cuda:1"` | 70 – 120 SKN |
| **009** | RTX 4080 Super 16GB | 16GB GDDR6X | `0x0202` (Gaussian Splatting 3D) | `skynet-agent --auto-bid --tags "cuda,gaussian-splat,vram-16g"`| 25 – 45 SKN |
| **010** | RTX 3060 Ti 8GB | 8GB GDDR6 | `0x0407` (DeepFilterNet Denoise) | `skynet-agent --auto-bid --tags "cuda,audio-dsp"` | 8 – 14 SKN |
| **011** | RTX 4070 Ti Super 16GB| 16GB GDDR6X | `0x0309`, `0x0306` (AutoDock, N-Body) | `skynet-agent --auto-bid --tags "cuda,science,docking"` | 22 – 38 SKN |
| **012** | RTX 5080 (Blackwell) | 16GB GDDR7 | `0x0107`, `0x0108` (DeepSeek FP4/FP8) | `skynet-agent --auto-bid --tags "cuda-sm100,fp4,high-throughput"`| 50 – 95 SKN |
| **013** | RTX 2080 Ti 11GB | 11GB GDDR6 | `0x0207` (Medical CT/MRI 3D Seg) | `skynet-agent --auto-bid --tags "cuda,medical,seg"` | 9 – 16 SKN |
| **014** | RTX 3050 8GB | 8GB GDDR6 | `0x0409` (Knowledge Graph NLP) | `skynet-agent --auto-bid --tags "cuda,nlp-light"` | 4 – 7 SKN |
| **015** | RTX 4090 Tản Nước | 24GB GDDR6X | Chạy tải tối đa 24/7 liên tục | `skynet-agent --auto-bid --allow-sustained-load --power-cap 450` | 60 – 100 SKN |
| **016** | Laptop RTX 4050 6GB | 6GB GDDR6 | `0x0103` (Whisper Small, 45W) | `skynet-agent --auto-bid --low-power-profile` | 4 – 8 SKN |
| **017** | Dual RTX 4070 (Không cầu)| 24GB chia đôi | `0x040E` (Camera Re-ID Heatmap) | `skynet-agent --auto-bid --parallel-tasks 2` | 30 – 50 SKN |
| **018** | eGPU RTX 3080 TB4 | 10GB GDDR6X | Tác vụ ban đêm theo lịch | `skynet-agent --auto-bid --schedule "22:00-07:00"` | 12 – 20 SKN |
| **019** | RTX 3080 Ti 12GB | 12GB GDDR6X | `0x040B` (Raytracing Synthetic Data) | `skynet-agent --auto-bid --tags "cuda,raytracing"` | 20 – 35 SKN |
| **020** | PC Gaming + 4TB NVMe | 16GB RAM + GPU | `0x0404`, `0x0405` (ETL Parquet, Vector Index)| `skynet-agent --auto-bid --tags "nvme-io,data-pipeline"` | 15 – 28 SKN |

---

## Chi Tiết Các Case Điển Hình & Kỹ Thuật Vận Hành

### Case 001: Biến Chiếc RTX 3060 12GB Huyền Thoại Thành Máy Kiếm Tiền Đêm
- **Lợi thế phần cứng:** RTX 3060 sở hữu 12GB VRAM — nhiều hơn cả RTX 3070/3080 (chỉ có 8-10GB). Điều này giúp nó xử lý mượt mà mô hình `RealESRGAN_x4plus` với batch size lớn mà không bị tràn bộ nhớ (Out-Of-Memory).
- **Quy trình hoạt động:**
  1. Người dùng cắm máy tính trước khi đi ngủ lúc 23:00.
  2. Bật lệnh lắng nghe tự động với cờ giới hạn nhiệt độ an toàn:
     ```bash
     skynet-agent \
       --listen 0.0.0.0:8888 \
       --auto-bid \
       --tags "cuda,upscale,vram-12g" \
       --min-bounty-rate 1.0 \
       --temp-limit 70 \
       --wallet-path ~/.skynet/worker_wallet.json
     ```
  3. Mạng lưới SkyNet nhận diện thiết bị hỗ trợ CUDA FP16 và VRAM 12GB, tự động đẩy các gói tin Upscale ảnh (Opcode `0x0101`).
  4. Sáng hôm sau lúc 07:00, kiểm tra ví: đã xử lý 18 tác vụ, kiếm được 12.4 SKN.

### Case 003: "Cỗ Máy Quái Vật" RTX 4090 24GB – Vua Khai Thác Bounty Cao Cấp
- **Lợi thế phần cứng:** 24GB VRAM tốc độ cao, hơn 16.000 nhân CUDA và nhân Tensor thế hệ 4. Có khả năng nuốt trọn mô hình FLUX.1 Schnell (Opcode `0x0102`) trong chưa đầy 4 giây mỗi ảnh.
- **Cấu hình tối ưu công suất và tản nhiệt:**
  ```bash
  # Khóa giới hạn công suất ở mức 320W (thay vì 450W gốc) để tiết kiệm điện và mát máy
  nvidia-smi -pl 320

  skynet-agent \
    --listen 0.0.0.0:8888 \
    --auto-bid \
    --tags "cuda,vram-24g,heavy-ai,flux,lora" \
    --min-bounty-rate 3.5 \
    --max-concurrent 2 \
    --wallet-path ~/.skynet/worker_wallet.json
  ```
- **Kinh tế:** Với 8 giờ nhàn rỗi mỗi ngày, nút có thể thu về 60 SKN. Chi phí điện tiêu thụ khoảng $0.6 (15.000 VNĐ), tỷ suất sinh lời cực cao so với đào tiền ảo truyền thống.

### Case 004: Laptop Gaming RTX 4060 – Tự Động Tạm Dừng Khi Tháo Sạc Hoặc Quá Nhiệt
- **Thách thức:** Laptop gaming dễ bị quá nhiệt và chai pin nếu chạy tải nặng liên tục trong thời gian dài.
- **Giải pháp SkyNet Agent:**
  Sử dụng cờ `--battery-guard` và `--temp-limit`:
  ```bash
  skynet-agent \
    --listen 0.0.0.0:8888 \
    --auto-bid \
    --tags "cuda,whisper,ocr,lightweight" \
    --battery-guard \
    --temp-limit 75 \
    --fan-profile aggressive
  ```
  - Khi rút sạc: Agent tự động hủy đấu thầu các job mới và chuyển sang trạng thái ngủ (Sleep).
  - Khi nhiệt độ GPU chạm ngưỡng 75°C: Agent tự động hạ xung nhịp hoặc hoàn thành dứt điểm job hiện tại rồi nghỉ 5 phút làm mát.

### Case 008: Cấu Hình Dual RTX 3090 (48GB VRAM) Cho Huấn Luyện LoRA
- **Cấu hình:** Sử dụng 2 card đồ họa RTX 3090 cũ (chi phí đầu tư rẻ hơn nhiều so với 1 card A100), kết hợp cầu nối NVLink hoặc giao tiếp PCIe 4.0 x8/x8.
- **Lệnh điều phối tải kép:**
  ```bash
  skynet-agent \
    --listen 0.0.0.0:8888 \
    --auto-bid \
    --devices "cuda:0,cuda:1" \
    --tags "cuda-multi,lora-70b,protein-folding,vram-48g" \
    --min-bounty-rate 5.0
  ```
- **Hiệu quả:** Xử lý được các bài toán tinh chỉnh LoRA LLM 70B (Opcode `0x0106`) và mô phỏng AlphaFold (Opcode `0x0301`). Mức thù lao mỗi job lên tới 15 – 35 SKN.

### Case 018: Lập Lịch Khai Thác Theo Giờ (Scheduling Automation)
- **Tình huống:** Người dùng sử dụng máy tính làm đồ họa từ 08:00 sáng đến 20:00 tối. Muốn máy tự động biến thành SkyNet Worker từ 22:00 đêm đến 07:00 sáng hôm sau.
- **Thiết lập:**
  ```bash
  skynet-agent \
    --listen 0.0.0.0:8888 \
    --auto-bid \
    --schedule "22:00-07:00" \
    --auto-shutdown-on-finish false
  ```
  Agent tự động đếm ngược và chỉ nhận các job có thời gian hoàn thành ước tính trước 07:00 sáng, đảm bảo khi người dùng thức dậy máy đã hoàn toàn rảnh rỗi và mát mẻ.

# Bách Khoa Toàn Thư: 100 Ca Nhu Cầu Sử Dụng Điện Toán Thực Tế (User Compendium)

Chào mừng bạn đến với tuyển tập **100 Ca Sử Dụng Thực Chiến Dành Cho Người Dùng (Client / User)** trên mạng lưới SkyNet. 

Dù bạn là một sinh viên, nhà nghiên cứu, lập trình viên, nhà sáng tạo nội dung, doanh nghiệp hay một AI Agent tự hành — bất cứ khi nào bạn cần năng lực tính toán CPU/GPU mà máy tính của bạn không đủ sức hoặc chi phí thuê Cloud Web2 quá đắt đỏ, SkyNet là giải pháp tức thì, bảo mật và tiết kiệm hơn 80%.

---

## 🧭 Bản Đồ 100 Ca Sử Dụng Phân Theo Lĩnh Vực

```
                                 100 USER COMPUTE USE CASES
                                              │
    ┌──────────────┬──────────────┬───────────┴──┬──────────────┬──────────────┐
    ▼              ▼              ▼              ▼              ▼              ▼
[Phần 1: AI/ML] [Phần 2: Media] [Phần 3: ZK/Sec] [Phần 4: HPC]  [Phần 5: Data] [Phần 6: Edge]
Case 001 - 020  Case 021 - 040  Case 041 - 055  Case 056 - 070  Case 071 - 085 Case 086 - 100
```

| Phần tài liệu | Miền ứng dụng | Số lượng case | Nội dung điển hình |
| :--- | :--- | :--- | :--- |
| **[Phần 1: Trí Tuệ Nhân Tạo & Mô Hình Sinh](./part-1-ai-generative-intelligence.md)** | AI / ML / LLM | **Case 001 – 020** | Fine-tune LoRA, Whisper voice cloning, FLUX diffusion, model quantization, LLM streaming. |
| **[Phần 2: Đồ Họa, 3D & Đa Phương Tiện](./part-2-media-graphics-spatial.md)** | Media / 3D / Video | **Case 021 – 040** | Render Blender Cycles 4K, 3D Gaussian Splatting, NeRF, upscale ảnh cũ, nén video AV1. |
| **[Phần 3: Mật Mã Học, ZK & An Ninh Mạng](./part-3-zk-cryptography-privacy.md)** | ZK / Security / TEE | **Case 041 – 055** | Audit hợp đồng mật TEE, zk-SNARK Groth16 prover, FHE eval, distributed fuzzing AFL++. |
| **[Phần 4: Khoa Học Dữ Liệu, Sinh Tin & HPC](./part-4-science-simulation-hpc.md)** | Khoa học / HPC / Y tế | **Case 056 – 070** | Dự đoán protein AlphaFold, mô phỏng phân tử GROMACS, khí động học CFD, Monte Carlo. |
| **[Phần 5: Dữ Liệu Lớn & Thị Trường Data](./part-5-data-pipelines-marketplace.md)** | Big Data / Pipelines | **Case 071 – 085** | Thu thập web scraping phân tán chống bot, OCR PDF Surya, Polars/DuckDB Parquet queries. |
| **[Phần 6: IoT, Robot & AI Agent Tự Hành](./part-6-edge-iot-defi-agents.md)** | Edge / Web3 / Agents | **Case 086 – 100** | Lidar SLAM 3D cho drone, solver hoán đổi vòng lặp CoW-Swap, Agent tự mua GPU. |

---

## ⚡ Chuẩn Hóa Qua Giao Thức Model Context Protocol (MCP)

Toàn bộ 100 tác vụ trong cẩm nang này đều được mã hóa theo hệ thống **Mã Opcode Phân Cấp (Hierarchical 16-bit Opcode)** và chuẩn hóa tương thích 1-1 với giao thức **Model Context Protocol (MCP)**.

Người dùng và các AI Agent có thể gửi yêu cầu tính toán theo 2 phương thức tùy biến:
1. **Qua Dòng Lệnh CLI Gọn Nhẹ:**
   ```bash
   skynet compute dispatch --opcode 0x1001 --input photo.png --output photo_4k.png --bounty 1500000000
   ```
2. **Qua Chuẩn Gọi Hàm MCP JSON-RPC Trực Tiếp (Dành Cho AI Agent):**
   ```json
   {
     "method": "tools/call",
     "params": {
       "name": "skynet://vision.upscale.real_esrgan",
       "arguments": { "scale": 4, "input_uri": "skynet-p2p://task/photo.png" }
     }
   }
   ```

*Xem chi tiết tài liệu kiến trúc tại [Đặc Tả Kiến Trúc MCP Compute](../../docs/MCP_COMPUTE_SPECIFICATION.md).*

---

## 📑 Danh Sách Chi Tiết 100 Tình Huống Người Dùng

### Nhóm 1: Trí Tuệ Nhân Tạo & Mô Hình Sinh (Case 001 – 020)
- **Case 001:** [Phóng to 100 ảnh gia đình từ 720p lên 4K bằng Real-ESRGAN](./01-ai-upscale-100-images.md).
- **Case 002:** [Audit pháp lý & rà soát rủi ro hợp đồng M&A 500 trang qua TEE Enclave](./02-private-document-audit.md).
- **Case 003:** Huấn luyện bộ chuyển đổi LoRA cho mô hình ngôn ngữ Llama-3-8B với ngân sách 30,000 nanoSKN.
- **Case 004:** Tạo ảnh nghệ thuật thương mại bằng FLUX.1-schnell phân giải 2048x2048 trong 10 giây.
- **Case 005:** Chuyển đổi giọng nói thành văn bản 100 file ghi âm phỏng vấn (Whisper v3).
- **Case 006:** Nhân bản giọng nói cảm xúc (Zero-Shot Voice Cloning) đọc sách nói 20 chương.
- **Case 007:** Nén lượng tử hóa mô hình 70B FP16 thành GGUF Q4_K_M chạy trên laptop.
- **Case 008:** Trích xuất vector embedding 50,000 tài liệu phục vụ tìm kiếm ngữ nghĩa RAG.
- **Case 009:** Tạo video hoạt họa AI 5 giây (AnimateDiff / CogVideoX) từ kịch bản văn bản.
- **Case 010:** Chạy sandbox Python cô lập cho Agent giải toán biểu tượng phức tạp.
- **Case 011:** Ghép trọng số (Model Merging SLERP/Ties) giữa hai checkpoint mô hình 14B.
- **Case 012:** Tính điểm mô hình phần thưởng (Reward Model Scoring) cho 10,000 câu phản hồi.
- **Case 013:** Sinh tập dữ liệu nhân tạo đa kịch bản (Synthetic Data Generation) cho y tế.
- **Case 014:** Tách bè âm thanh tách beat / vocal từ 50 bài hát bằng Demucs v4.
- **Case 015:** Nhận diện khuôn mặt & làm mờ biển số tự động trên 10,000 ảnh camera hành trình.
- **Case 016:** Tóm tắt 50 video họp Zoom bằng mô hình đa phương thức VLM.
- **Case 017:** Sinh phụ đề song ngữ tự động khớp từng mili-giây cho kênh YouTube.
- **Case 018:** Phân tích cảm xúc & trích xuất thực thể (NER) từ 200,000 tweet Twitter/X.
- **Case 019:** Khử nhiễu âm thanh podcast bị tạp âm quạt gió / tiếng ồn đường phố.
- **Case 020:** Chạy thử nghiệm đánh giá Benchmark MMLU cho mô hình nơ-ron tùy biến.

### Nhóm 2: Đồ Họa, 3D & Đa Phương Tiện (Case 021 – 040)
- **Case 021:** [Nông trại render 3D Blender Cycles 300 khung hình 4K song song](./03-blender-3d-render-farm.md).
- **Case 022:** Huấn luyện 3D Gaussian Splatting (3DGS) từ 200 ảnh chụp Flycam khu nghỉ dưỡng.
- **Case 023:** Khớp trường bức xạ thần kinh NeRF tái lập không gian nội thất căn hộ.
- **Case 024:** Tái tạo lưới đa giác 3D Photogrammetry từ 1,000 ảnh scan tượng cổ vật.
- **Case 025:** Chuyển mã video 8K ProRes sang chuẩn nén AV1 chia thành 50 phân đoạn GOP.
- **Case 026:** Nội suy tăng tốc khung hình video từ 24fps lên 120fps siêu mượt (RIFE/SVP).
- **Case 027:** Nướng bản đồ ánh sáng tĩnh (Global Illumination Light Baking) cho màn game Unity.
- **Case 028:** Sinh bản đồ chất liệu (PBR Textures: Albedo, Normal, Height) từ ảnh chụp thực tế.
- **Case 029:** Phục chế màu phim tài liệu lịch sử đen trắng sang màu 4K.
- **Case 030:** Lọc nhiễu cảm biến LiDAR hàng triệu điểm cho mô hình khảo sát địa hình.
- **Case 031:** Mô phỏng vật lý khói lửa & chất lỏng (Fluid & Smoke Simulation) cho kỹ xảo điện ảnh.
- **Case 032:** Xóa watermark / vật thể thừa trên video 4K bằng Video Inpainting AI.
- **Case 033:** Phân giải trường ánh sáng VR/AR Light Field cho kính thực tế ảo Apple Vision Pro.
- **Case 034:** Tạo avatar 3D người thật chuyển động theo camera webcam thời gian thực.
- **Case 035:** Render kiến trúc Lumion / V-Ray cho dự án quy hoạch đô thị 50 hecta.
- **Case 036:** Tự động cắt ghép highlight trận đấu bóng đá 90 phút theo cảm xúc khán giả.
- **Case 037:** Đổi phông xanh tự động không cần phông vật lý (Background Matting AI) cho phim ngắn.
- **Case 038:** Biến đổi giọng nói diễn viên lồng tiếng sang 10 ngôn ngữ bản địa giữ nguyên ngữ điệu.
- **Case 039:** Tạo bản đồ độ sâu (Depth Map Estimation) cho 500 ảnh chụp phục vụ màn hình 3D.
- **Case 040:** Nén hàng loạt video TikTok/Reels về chuẩn H.265 tối ưu dung lượng giữ nguyên chất lượng.

### Nhóm 3: Mật Mã Học, ZK & An Ninh Mạng (Case 041 – 055)
- **Case 041:** Sinh bằng chứng không tri thức zk-SNARK Groth16 cho giao dịch bảo mật.
- **Case 042:** Nội suy ma trận vết thực thi zk-STARK qua biến đổi NTT cho Layer-2 Rollup.
- **Case 043:** Tính toán nhân vô hướng đa điểm (MSM) quy mô 2^20 điểm đường cong Elliptic.
- **Case 044:** Đánh giá mạch tính toán trên dữ liệu đã mã hóa hoàn toàn (FHE Evaluation).
- **Case 045:** Ghép mảnh bí mật ngưỡng MPC (Shamir Secret Sharing) giữa các tổ chức tài chính.
- **Case 046:** Trích xuất bằng chứng phiên web zkTLS chứng minh số dư ngân hàng không lộ mật khẩu.
- **Case 047:** Quét lỗ hổng bảo mật hợp đồng thông minh bằng thực thi ký hiệu (Symbolic Execution).
- **Case 048:** Chạy kiểm thử tự động Fuzz Testing AFL++ trên binary phần mềm tìm lỗi tràn bộ nhớ.
- **Case 049:** Dịch ngược Headless Ghidra Decompiler phân tích mã độc trong máy ảo cô lập.
- **Case 050:** Quét mã độc động trong Sandbox Detonation theo dõi hành vi ghi đĩa.
- **Case 051:** Tìm kiếm địa chỉ ví đẹp (Vanity Address) theo tiền tố mong muốn qua song song GPU.
- **Case 052:** Sinh khóa lượng tử ML-KEM/ML-DSA (Dilithium/Kyber) chống máy tính lượng tử.
- **Case 053:** Kiểm tra tính toàn vẹn chuỗi cung ứng phần mềm đối chiếu 5,000 package npm.
- **Case 054:** Đóng gói bằng chứng tích lũy mật mã (Utreexo Accumulator Proof Generator).
- **Case 055:** Bắn gói tin Fuzzing kiểm tra độ bền router phần cứng chịu tải 1 triệu packet/s.

### Nhóm 4: Khoa Học Dữ Liệu, Sinh Tin & HPC (Case 056 – 070)
- **Case 056:** Dự đoán cấu trúc gập không gian 3D của chuỗi axit amin qua AlphaFold.
- **Case 057:** Mô phỏng động học phân tử thuốc tương tác thụ thể tế bào (GROMACS).
- **Case 058:** Giải phương trình Navier-Stokes mô phỏng khí động học cánh máy bay (CFD).
- **Case 059:** Gióng hàng chuỗi gen DNA bệnh nhân ung thư vào bộ gen chuẩn (BLAST/BWA).
- **Case 060:** Mô phỏng rủi ro định giá danh mục tài chính 10 triệu kịch bản Monte Carlo.
- **Case 061:** Dự báo thời tiết tiểu vùng độ phân giải 1km dựa trên mô hình khí quyển WRF.
- **Case 062:** Tính toán tương tác hấp dẫn vũ trụ hàng tỉ hạt vật chất tối (N-Body Astrophysics).
- **Case 063:** Phân tích ứng suất kết cấu nhịp cầu dây văng chịu tải trọng gió bão (FEA).
- **Case 064:** Giả lập mạch điện toán lượng tử 35 qubit trên bộ nhớ RAM phân tán.
- **Case 065:** Giải hệ phương trình vi phân phi tuyến động học phản ứng hóa học xúc tác.
- **Case 066:** Xử lý tín hiệu địa chấn phát hiện túi dầu khí dưới lòng đất 3,000 mét.
- **Case 067:** Mô phỏng tán xạ nơ-tron trong lò phản ứng năng lượng hạt nhân thế hệ mới.
- **Case 068:** Phân tích biểu hiện gen đơn bào (Single-Cell RNA Sequencing) 100,000 tế bào.
- **Case 069:** Dự báo lan truyền ô nhiễm bụi mịn PM2.5 theo địa hình đô thị.
- **Case 070:** Tối ưu hóa chuỗi phản ứng enzyme trong sản xuất nhiên liệu sinh học.

### Nhóm 5: Dữ Liệu Lớn & Thị Trường Data (Case 071 – 085)
- **Case 071:** Thu thập thông tin giá sản phẩm thương mại điện tử từ 500 IP dân cư chống bot.
- **Case 072:** Trích xuất bảng biểu hóa đơn tài chính từ 10,000 file scan PDF qua Surya OCR.
- **Case 073:** Loại bỏ trùng lặp văn bản 500GB dữ liệu tiền huấn luyện qua MinHash / LSH.
- **Case 074:** Chạy truy vấn phân tích tổng hợp trên 100GB file Parquet bằng Polars / DuckDB.
- **Case 075:** Xây dựng đồ thị tìm kiếm láng giềng gần nhất HNSW cho 5 triệu vector 1536 chiều.
- **Case 076:** Làm sạch & gộp dữ liệu danh bạ khách hàng trùng lặp bằng thuật toán Fuzzy Matching.
- **Case 077:** Tổng hợp bản đồ nhiệt di chuyển từ 1 triệu điểm tọa độ GPS taxi.
- **Case 078:** Quét và gỡ bỏ thông tin định danh cá nhân (PII Redaction: CCCD, SĐT) trên kho văn bản.
- **Case 079:** Phát hiện dị thường thời gian thực trên chuỗi dữ liệu 10,000 cảm biến nhà máy.
- **Case 080:** Nén và đóng gói trạng thái sổ cái blockchain 500GB thành Merkle snapshot 10MB.
- **Case 081:** Chuyển đổi dữ liệu vệ tinh đa phổ thành bản đồ chỉ số thực vật NDVI theo dõi hạn hán.
- **Case 082:** Phân tích đồ thị mạng xã hội phát hiện mạng lưới tài khoản ảo gian lận (Sybil Detection).
- **Case 083:** Xây dựng bộ lọc thư rác thông minh từ tập 1 triệu email nội bộ doanh nghiệp.
- **Case 084:** Trích xuất tự động điều khoản phạt từ 2,000 hợp đồng mua sắm công.
- **Case 085:** Đồng bộ hóa & đối chiếu chéo kho dữ liệu bán hàng đa kênh trong 30 giây.

### Nhóm 6: IoT, Robot & AI Agent Tự Hành (Case 086 – 100)
- **Case 086:** Định vị & lập bản đồ 3D SLAM thời gian thực từ đám mây điểm LiDAR cho robot kho hàng.
- **Case 087:** Tính toán quỹ đạo bay tránh va chạm cho đàn máy bay không người lái 100 chiếc.
- **Case 088:** Thu gom và hợp nhất gradient học liên kết (Federated Learning) từ 10,000 smartphone.
- **Case 089:** Tối ưu hóa lộ trình giao vận 5,000 đơn hàng bưu tá trong giờ cao điểm (VRP).
- **Case 090:** Cân bằng tải tiêu thụ điện tức thời của lưới điện vi mô năng lượng mặt trời.
- **Case 091:** Định vị nguồn phát tiếng nổ / sự cố dựa trên độ lệch pha sóng âm 20 cảm biến.
- **Case 092:** Phát hiện lỗi vi mạch bán dẫn tốc độ cao trên băng chuyền sản xuất vi cơ điện tử.
- **Case 093:** Tự động điều chỉnh chu kỳ đèn tín hiệu giao thông ngã tư dựa trên camera trực tiếp.
- **Case 094:** Phân tích ảnh siêu âm phát hiện vết nứt ăn mòn đường ống dẫn dầu dưới đáy biển.
- **Case 095:** Tìm kiếm chuỗi hoán đổi vòng lặp (CoW-Swap Batch Auction Solver) cho sàn DEX.
- **Case 096:** Mô phỏng phòng chống MEV Sandwich bảo vệ lệnh giao dịch quỹ tài chính.
- **Case 097:** Tính toán ma trận độ nhạy Hy Lạp (Greeks: Delta, Gamma, Vega) cho 50,000 quyền chọn.
- **Case 098:** Dự báo nguy cơ vỡ nợ chuỗi trên đồ thị tín dụng phi tập trung DeFi.
- **Case 099:** Tối ưu hóa đường cong thanh khoản tập trung (Concentrated Liquidity) cho AMM.
- **Case 100:** AI Agent tự động thuê GPU từ mạng lưới để giải toán khi phát hiện hết VRAM cục bộ.

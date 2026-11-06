# Phần 1: 20 Tình Huống Điện Toán AI & Mô Hình Sinh (Cases 001 – 020)

---

### Case 001: Phóng To 100 Ảnh Gia Đình Từ 720p Lên 4K Bằng Real-ESRGAN
- **Bối cảnh:** 100 ảnh cũ gia đình độ phân giải thấp ($1280 \times 720$), laptop văn phòng chạy mất 8 tiếng.
- **Đầu vào / Đầu ra:** Thư mục `~/photos_720p/` (120 MB) $\to$ Thư mục `~/photos_4k/` (100 ảnh $3840 \times 2160$, 800 MB).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ~/photos_720p/ --output ~/photos_4k/ --opcode 0x0215 --model "Real-ESRGAN_x4plus" --bounty-per-item 100 --parallel 10
  ```
- **Kinh tế:** Tốn **10,000 nanoSKN** (~3,000 VNĐ), hoàn tất trong 2 phút 14 giây (so với $20/tháng trên SaaS).
- *Chi tiết xem tại:* [01-ai-upscale-100-images.md](./01-ai-upscale-100-images.md).

---

### Case 002: Kiểm Toán Pháp Lý Hợp Đồng M&A 500 Trang Bảo Mật Qua TEE Enclave
- **Bối cảnh:** Tài liệu sáp nhập doanh nghiệp tối mật dài 500 trang, tuyệt đối không được tải lên máy chủ OpenAI/Google.
- **Đầu vào / Đầu ra:** Tệp `contract_audit.pdf` (18 MB) $\to$ Báo cáo phân tích rủi ro `audit_report.md`.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ~/confidential/contract_audit.pdf --output ~/confidential/audit_report.md --opcode 0x0109 --model "Llama-3.3-70B-Instruct-Q4" --require-tee --bounty 15000
  ```
- **Kinh tế:** Tốn **15,000 nanoSKN**, dữ liệu mã hóa phong bì trong vùng nhớ phần cứng TEE, không ai đọc trộm được.
- *Chi tiết xem tại:* [02-private-document-audit.md](./02-private-document-audit.md).

---

### Case 003: Huấn Luyện Bộ Chuyển Đổi LoRA Cho Llama-3-8B Với Ngân Sách Hạn Hẹp
- **Bối cảnh:** Sinh viên cần fine-tune mô hình Llama-3 trên tập 5,000 câu đối thoại y tế bằng LoRA rank 16.
- **Đầu vào / Đầu ra:** Tệp dữ liệu `dataset.jsonl` (12 MB) $\to$ Thư mục trọng số adapter `adapter_weights.safetensors` (48 MB).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./medical_qa.jsonl --output ./lora_adapter/ --opcode 0x0102 --model "meta-llama/Llama-3-8b" --max-ticks 100000 --bounty 35000 --timeout 1800
  ```
- **Kinh tế:** Tốn **35,000 nanoSKN** (~10,000 VNĐ), chạy trên Worker có RTX 4090 trong 18 phút (tiết kiệm 85% so với thuê RunPod A100).

---

### Case 004: Tạo Ảnh Nghệ Thuật 2048x2048 Bằng FLUX.1-schnell Trong 10 Giây
- **Bối cảnh:** Nhà thiết kế quảng cáo cần sinh 10 ảnh concept độ phân giải cao phục vụ thiết kế banner.
- **Đầu vào / Đầu ra:** File mô tả `prompts.txt` (10 câu prompt) $\to$ 10 file ảnh PNG chất lượng studio 2K.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input prompts.txt --output ./banners/ --opcode 0x0105 --model "FLUX.1-schnell" --bounty-per-item 80 --parallel 5
  ```
- **Kinh tế:** Tốn **800 nanoSKN** (~200 VNĐ) cho 10 ảnh, rẻ gấp 20 lần so với mua gói Midjourney $30/tháng.

---

### Case 005: Bóc Băng Tự Động 100 Tệp Ghi Âm Phỏng Vấn Dài Bằng Whisper v3
- **Bối cảnh:** Nhà báo có 100 file âm thanh ghi âm phỏng vấn (.m4a, tổng 20 tiếng), cần chuyển thành văn bản có dấu thời gian.
- **Đầu vào / Đầu ra:** Thư mục `~/recordings/` $\to$ Thư mục `~/transcripts/` (.srt & .txt).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ~/recordings/ --output ~/transcripts/ --opcode 0x0107 --model "openai/whisper-large-v3" --bounty-per-item 120 --parallel 8
  ```
- **Kinh tế:** Xong toàn bộ 20 tiếng ghi âm trong **12 phút** nhờ 8 máy thợ cùng gõ song song; chi phí **12,000 nanoSKN**.

---

### Case 006: Nhân Bản Giọng Nói Đọc Sách Nói 20 Chương (Zero-Shot Voice Cloning)
- **Bối cảnh:** Tác giả muốn dùng chính giọng đọc mẫu 30 giây của mình để chuyển thể 20 chương sách chữ thành tệp audiobook mp3.
- **Đầu vào / Đầu ra:** Mẫu giọng `voice_sample.wav` (30s) + 20 file `chapter_*.txt` $\to$ 20 file `audiobook_*.mp3`.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./book_text/ --ref-audio ./voice_sample.wav --output ./audiobook/ --opcode 0x0108 --model "CosyVoice-300M" --bounty 18000
  ```
- **Kinh tế:** Tốn **18,000 nanoSKN**, không mất phí thuê studio hay phí đăng ký ElevenLabs $22/tháng.

---

### Case 007: Nén Lượng Tử Hóa Mô Hình 70B FP16 Xuống GGUF Q4_K_M
- **Bối cảnh:** Cần nén một mô hình mã nguồn mở 70B dung lượng 140GB xuống còn 40GB để chạy vừa máy cá nhân.
- **Đầu vào / Đầu ra:** Checkpoint HuggingFace $\to$ File nhị phân `Llama-3-70B-Q4_K_M.gguf`.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --model-repo "meta-llama/Llama-3-70B" --output ./gguf/ --opcode 0x0104 --quant-type "Q4_K_M" --bounty 25000 --timeout 2400
  ```
- **Kinh tế:** Máy thợ có 128GB RAM chạy nén trong 30 phút, chi phí **25,000 nanoSKN**, người dùng không cần mua thêm RAM đắt đỏ.

---

### Case 008: Trích Xuất Vector Embedding 50,000 Tài Liệu Cho Hệ Thống RAG
- **Bối cảnh:** Xây dựng cơ sở tri thức hỏi đáp doanh nghiệp RAG từ 50,000 đoạn văn bản pháp lý.
- **Đầu vào / Đầu ra:** File `corpus.jsonl` $\to$ File ma trận embedding nhị phân `embeddings.npy` (50,000 x 1024).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./corpus.jsonl --output ./vectors/ --opcode 0x0103 --model "BAAI/bge-large-en-v1.5" --bounty 8000 --parallel 4
  ```
- **Kinh tế:** Tốn **8,000 nanoSKN**, hoàn tất trong 45 giây qua Tensor Cores của 4 máy thợ.

---

### Case 009: Sinh Video Hoạt Họa AI 5 Giây (AnimateDiff) Từ Kịch Bản Quảng Cáo
- **Bối cảnh:** YouTuber cần sinh 5 cảnh video ngắn hoạt họa anime phục vụ video âm nhạc.
- **Đầu vào / Đầu ra:** File mô tả phân cảnh `scenes.json` $\to$ 5 video ngắn `scene_*.mp4` (720p 24fps).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./scenes.json --output ./video_scenes/ --opcode 0x0106 --model "AnimateDiff-Lightning" --bounty-per-item 400 --parallel 5
  ```
- **Kinh tế:** Tốn **2,000 nanoSKN** cho cả 5 video clip, hoàn thành trong 1 phút 30 giây.

---

### Case 010: Khởi Chạy Sandbox Python Cô Lập Cho AI Agent Giải Toán Đại Số
- **Bối cảnh:** Agent cần chạy mã Python tính ma trận nghịch đảo và vẽ đồ thị hàm số mà không làm treo tiến trình chính.
- **Đầu vào / Đầu ra:** File kịch bản `calc_matrix.py` $\to$ File kết quả JSON & hình ảnh `plot.png`.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --script ./calc_matrix.py --opcode 0x010A --sandbox wasmtime --max-ticks 50000 --bounty 50
  ```
- **Kinh tế:** Tốn **50 nanoSKN** (< 0.01 xu), thực thi an toàn trong Wasmtime MicroCell Sandbox.

---

### Case 011: Ghép Trọng Số Mô Hình (Model Merging SLERP) Giữa 2 Checkpoint 14B
- **Bối cảnh:** Kỹ sư AI muốn lai ghép khả năng toán học của Model A và khả năng lập trình của Model B bằng thuật toán SLERP.
- **Đầu vào / Đầu ra:** Hai mô hình HuggingFace $\to$ Trọng số lai ghép `ModelMerged-14B`.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --merge-config merge_slerp.yaml --opcode 0x010D --bounty 12000
  ```
- **Kinh tế:** Tốn **12,000 nanoSKN**, tiết kiệm 4 tiếng thiết lập máy chủ trên Cloud.

---

### Case 012: Chấm Điểm Mô Hình Phần Thưởng (RLHF Reward Scoring) 10,000 Câu Trả Lời
- **Bối cảnh:** Đội ngũ AI cần chọn lọc câu trả lời tốt nhất trong tập dữ liệu huấn luyện căn chỉnh con người.
- **Đầu vào / Đầu ra:** Tệp `pairs.jsonl` $\to$ Tệp kết quả điểm số `scored_rewards.csv`.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./pairs.jsonl --output ./scores/ --opcode 0x010B --model "ArmoRM-Llama3-8B" --bounty 6000 --parallel 6
  ```
- **Kinh tế:** Tốn **6,000 nanoSKN**, xử lý 10,000 lượt chấm điểm trong 1 phút.

---

### Case 013: Sinh Tập Dữ Liệu Y Tế Nhân Tạo Đa Kịch Bản (Synthetic Data)
- **Bối cảnh:** Bệnh viện cần tập 20,000 ca bệnh nhân tạo để nghiên cứu mà không vi phạm đạo đức y tế và bảo mật danh tính bệnh nhân thật.
- **Đầu vào / Đầu ra:** Kịch bản sinh `schema_medical.json` $\to$ File dữ liệu `synthetic_patients.parquet`.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --config schema_medical.json --count 20000 --opcode 0x010C --bounty 15000
  ```
- **Kinh tế:** Tốn **15,000 nanoSKN**, chất lượng dữ liệu tương đương thuê chuyên viên y tế soạn tay hàng tháng trời.

---

### Case 014: Tách Bè Nhạc Cụ & Lời Hát Cho 50 Bài Hát Bằng Demucs v4
- **Bối cảnh:** DJ / Producer cần tách Vocal, Drum, Bass để remix 50 ca khúc chất lượng lossless FLAC.
- **Đầu vào / Đầu ra:** Thư mục 50 bài hát `.flac` $\to$ Các thư mục `stems/` chứa riêng từng track nhạc cụ.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./songs/ --output ./stems/ --opcode 0x010E --model "htdemucs_ft" --bounty-per-item 150 --parallel 8
  ```
- **Kinh tế:** Xong 50 bài hát chỉ trong **8 phút**, chi phí **7,500 nanoSKN** (Web2 tính phí $0.50/bài = $25).

---

### Case 015: Làm Mờ Biển Số Xe & Khuôn Mặt Tự Động Trên 10,000 Ảnh Đường Phố
- **Bối cảnh:** Công ty xe tự hành cần xử lý quyền riêng tư cho tập ảnh camera trước khi công bố ra công chúng.
- **Đầu vào / Đầu ra:** Thư mục 10,000 ảnh JPEG $\to$ Thư mục ảnh đã được làm mờ PII bằng mô hình YOLOv10-Face.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./dashcam_raw/ --output ./dashcam_clean/ --opcode 0x010F --bounty-per-item 2 --parallel 16
  ```
- **Kinh tế:** Tốn **20,000 nanoSKN**, 16 máy thợ xử lý 10,000 ảnh trong vỏn vẹn **3 phút 20 giây**.

---

### Case 016: Tóm Tắt 50 Video Họp Zoom Dài Bằng Mô Hình Đa Phương Thức VLM
- **Bối cảnh:** Doanh nghiệp có 50 buổi họp ghi hình dài 1 tiếng, cần tóm tắt biên bản quyết định và giao việc.
- **Đầu vào / Đầu ra:** File video cuộc họp $\to$ Các tài liệu Markdown biên bản cuộc họp.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./zoom_meetings/ --output ./meeting_notes/ --opcode 0x0110 --model "Qwen2-VL-7B" --bounty-per-item 300
  ```
- **Kinh tế:** Tốn **15,000 nanoSKN**, tiết kiệm hàng trăm giờ nghe lại băng của nhân viên.

---

### Case 017: Sinh Phụ Đề Song Ngữ Tự Động Khớp Từng Mili-giây Cho Video YouTube
- **Bối cảnh:** Nhà sáng tạo nội dung cần tạo file phụ đề tiếng Anh và tiếng Việt chuẩn xác cho 30 video 4K.
- **Đầu vào / Đầu ra:** 30 file âm thanh MP3 $\to$ 30 file `.srt` chuẩn xác thời gian từng câu nói.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./audio_tracks/ --output ./subtitles/ --opcode 0x0111 --bounty-per-item 80 --parallel 5
  ```
- **Kinh tế:** Tốn **2,400 nanoSKN**, hoàn thành trong 3 phút.

---

### Case 018: Phân Tích Cảm Xúc & Phân Loại 200,000 Bình Luận Thương Mại Điện Tử
- **Bối cảnh:** Nhãn hàng cần phân tích phản hồi của người tiêu dùng để phát hiện sản phẩm lỗi và khiếu nại.
- **Đầu vào / Đầu ra:** File `reviews.csv` (200,000 dòng) $\to$ File phân loại sentiment & category.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input reviews.csv --output ./sentiment_results.csv --opcode 0x0112 --bounty 5000 --parallel 10
  ```
- **Kinh tế:** Tốn **5,000 nanoSKN**, phân rã thành 10 sub-batches chạy trong 40 giây.

---

### Case 019: Khử Nhiễu Âm Thanh Podcast Bằng DeepFilterNet
- **Bối cảnh:** Tập podcast thu âm ngoài trời bị lẫn tiếng mưa rơi và còi xe, cần phục hồi giọng nói studio trong trẻo.
- **Đầu vào / Đầu ra:** File `podcast_noisy.wav` (45 phút) $\to$ File `podcast_clean.wav`.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input podcast_noisy.wav --output podcast_clean.wav --opcode 0x0113 --model "DeepFilterNet3" --bounty 500
  ```
- **Kinh tế:** Tốn **500 nanoSKN**, hoàn tất trong 15 giây.

---

### Case 020: Chạy Đánh Giá Benchmark MMLU Cho Mô Hình Ngôn Ngữ Tự Huấn Luyện
- **Bối cảnh:** Phòng thí nghiệm AI cần kiểm tra chỉ số thông minh của mô hình vừa tinh chỉnh trên 57 môn học.
- **Đầu vào / Đầu ra:** Checkpoint mô hình $\to$ Bảng điểm số accuracy (%) chi tiết từng bộ môn.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --model ./my_model_ckpt/ --benchmark "MMLU" --opcode 0x0114 --bounty 10000 --timeout 1200
  ```
- **Kinh tế:** Tốn **10,000 nanoSKN**, máy thợ chạy trọn bộ 14,000 câu trắc nghiệm trong 8 phút.

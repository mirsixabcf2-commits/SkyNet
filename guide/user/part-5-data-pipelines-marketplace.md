# Tuyển tập Case 071 – 085: Đường ống Dữ liệu Lớn & Sàn Tri thức (Data Pipelines & Marketplace)

Tài liệu này hướng dẫn chi tiết cách Người dùng (Data Engineer, Data Scientist, Doanh nghiệp) điều phối các tác vụ xử lý dữ liệu lớn (ETL/ELT), chỉ mục hóa vector, trích xuất dữ liệu đa phương tiện và xử lý văn bản quy mô hàng triệu bản ghi trên mạng lưới phân tán SkyNet.

---

## Case 071: Cào bóc dữ liệu và làm sạch 1 triệu trang web thương mại điện tử (Distributed Scrapy)
- **Tình huống:** Công ty nghiên cứu thị trường cần thu thập và bóc tách cấu trúc giá của 1 triệu trang sản phẩm e-commerce trong 3 giờ.
- **Dữ liệu đầu vào:** File danh sách URLs và quy tắc selector JSON `crawl_manifest.json` (45 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0401` (`DistributedWebScrapeClean`), Mạng băng thông cao, CPU nhiều nhân.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0401 \
    --input ./crawling/crawl_manifest.json \
    --output ./datasets/ecommerce_cleaned_1m.jsonl.zst \
    --bounty 7500000000 \
    --auditor-quorum 3 \
    --timeout 10800 \
    --tags "data,scraping,network,high-bandwidth"
  ```
- **Xác thực & Kết quả:** Auditor ngẫu nhiên kiểm tra chéo 200 URL trên trang web nguồn để đối chiếu tính trung thực của giá và tồn kho. Kết quả xuất dạng file nén zstandard `jsonl`.

---

## Case 072: Trích xuất bảng biểu hóa đơn phức tạp đa ngôn ngữ (Surya OCR / Donut)
- **Tình huống:** Doanh nghiệp kế toán dịch vụ cần nhận diện và trích xuất bảng biểu chi tiết từ 15.000 file scan hóa đơn PDF bị nghiêng, mờ.
- **Dữ liệu đầu vào:** File nén ảnh hóa đơn `invoices_batch_15k.zip` (3.8 GB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0402` (`TableExtractionOcrSurya`), GPU VRAM $\ge 12$GB.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0402 \
    --input ./invoices/invoices_batch_15k.zip \
    --output ./invoices/extracted_tables.json \
    --bounty 5000000000 \
    --auditor-quorum 3 \
    --timeout 7200 \
    --tags "ocr,surya,table-transformer,gpu"
  ```
- **Xác thực & Kết quả:** Kiểm toán viên tính tổng tiền từng dòng hàng (Line items) đối chiếu tổng tiền thanh toán hóa đơn.

---

## Case 073: Loại bỏ trùng lặp ngữ nghĩa tập dữ liệu văn bản 500GB (MinHash LSH Deduplication)
- **Tình huống:** Nhóm tiền xử lý dữ liệu huấn luyện LLM cần lọc bỏ tài liệu trùng lặp hoàn toàn hoặc gần giống (Near-duplicate) trên tập corpus 500GB.
- **Dữ liệu đầu vào:** Danh mục các tệp Parquet văn bản thô `text_corpus_500g.manifest` (80 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0403` (`MinHashLshDeduplication`), RAM $\ge 64$GB, SSD NVMe tốc độ cao.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0403 \
    --input ./corpus/text_corpus_500g.manifest \
    --output ./corpus/dedup_filtered_index.bin \
    --bounty 12000000000 \
    --auditor-quorum 2 \
    --timeout 18000 \
    --tags "nlp,dedup,minhash,high-ram"
  ```
- **Xác thực & Kết quả:** Kiểm tra tỷ lệ Jaccard similarity trên các cặp tài liệu bị gắn cờ trùng lặp. User nhận danh sách ID tài liệu duy nhất và chỉ mục loại trừ.

---

## Case 074: Chuyển đổi kho dữ liệu lớn CSV $\to$ Snappy Parquet (Polars / DuckDB)
- **Tình huống:** Chuyển đổi 200GB dữ liệu giao dịch tài chính từ các file `.csv` thô sang định dạng cột `.parquet` nén Snappy với dictionary encoding.
- **Dữ liệu đầu vào:** Danh mục tệp CSV `financial_csv_list.tar` (1.5 GB file nén danh mục và dữ liệu mẫu).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0404` (`EtlCsvToParquetPolars`), Đa luồng CPU SIMD, NVMe I/O cao.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0404 \
    --input ./data/financial_csv_list.tar \
    --output ./parquet_lake/transactions_snappy.parquet \
    --bounty 3200000000 \
    --auditor-quorum 2 \
    --timeout 3600 \
    --tags "data,etl,polars,duckdb"
  ```
- **Xác thực & Kết quả:** Ban kiểm chứng kiểm tra tổng checksum số lượng dòng bản ghi và tổng giá trị cột số liệu đối chiếu giữa hai định dạng.

---

## Case 075: Xây dựng chỉ mục tìm kiếm vector 10 triệu embeddings (HNSW Vector Index Build)
- **Tình huống:** Hệ thống RAG doanh nghiệp cần dựng đồ thị vector HNSW (Hierarchical Navigable Small World) từ 10 triệu vectors 1536 chiều để tìm kiếm thời gian thực.
- **Dữ liệu đầu vào:** File ma trận vector `embeddings_10m_1536d.bin` (61.4 GB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0405` (`HnswIndexBuildFaiss`), RAM $\ge 96$GB, đa nhân CPU xung cao.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0405 \
    --input ./vectors/embeddings_10m_1536d.bin \
    --output ./index/knowledge_base_hnsw.index \
    --bounty 14000000000 \
    --auditor-quorum 3 \
    --timeout 14400 \
    --tags "vector,hnsw,faiss,high-ram"
  ```
- **Xác thực & Kết quả:** Kiểm tra Recall@10 trên 1.000 truy vấn thử nghiệm ngẫu nhiên đạt $> 98\%$.

---

## Case 076: Ẩn danh hóa và che dấu dữ liệu nhạy cảm PII 100.000 hồ sơ bệnh án (PII Redaction)
- **Tình huống:** Bệnh viện cần công bố tập dữ liệu y khoa phục vụ nghiên cứu nhưng phải loại bỏ 100% tên, số CMND/CCCD, địa chỉ, số điện thoại của bệnh nhân.
- **Dữ liệu đầu vào:** File JSONL chứa 100.000 hồ sơ `ehr_records_raw.jsonl` (420 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0406` (`MedicalPiiRedaction`), TEE Hardware Enclave (Intel SGX / AMD SEV) bắt buộc để đảm bảo bí mật tuyệt đối.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0406 \
    --input ./medical/ehr_records_raw.jsonl \
    --output ./medical/ehr_anonymized.jsonl \
    --bounty 6500000000 \
    --auditor-quorum 3 \
    --tee-required \
    --timeout 7200 \
    --tags "privacy,pii,tee,hipaa"
  ```
- **Xác thực & Kết quả:** Xác thực chữ ký phần cứng TEE Quote và quét regex đối soát mẫu ngẫu nhiên không còn sót token PII nào.

---

## Case 077: Xử lý và chuẩn hóa tín hiệu cảm biến rung động IoT công nghiệp 1 năm (Timeseries DSP)
- **Tình huống:** Nhà máy nhiệt điện phân tích tín hiệu gia tốc rung động tần số 10kHz thu thập từ vòng bi tuabin trong 365 ngày để phát hiện sớm hư hỏng.
- **Dữ liệu đầu vào:** Tệp nén nhị phân cảm biến `vibration_1year_10khz.tar.zst` (18 GB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0407` (`IotDspFastFourierTransform`), GPU / AVX-512 CPU.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0407 \
    --input ./iot/vibration_1year_10khz.tar.zst \
    --output ./iot/spectral_kurtosis_anomalies.parquet \
    --bounty 5800000000 \
    --auditor-quorum 2 \
    --timeout 7200 \
    --tags "dsp,fft,iot,timeseries"
  ```
- **Xác thực & Kết quả:** Kiểm tra tính tuần hoàn Fourier và phổ tần đặc trưng của lỗi rãnh lăn ngoài (BPFO).

---

## Case 078: Lọc bỏ bình luận độc hại, thù địch và spam 10 triệu bài viết (Bilingual Moderation)
- **Tình huống:** Nền tảng mạng xã hội địa phương cần kiểm duyệt và dán nhãn phân loại độc hại cho 10 triệu bình luận tiếng Việt và tiếng Anh.
- **Dữ liệu đầu vào:** File nén văn bản `comments_stream_10m.csv.zst` (1.2 GB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0408` (`ContentModerationDeBERTa`), GPU RTX 3080/4080 hoặc cụm đa GPU.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0408 \
    --input ./moderation/comments_stream_10m.csv.zst \
    --output ./moderation/moderated_labels.parquet \
    --bounty 8500000000 \
    --auditor-quorum 3 \
    --timeout 10800 \
    --tags "nlp,moderation,gpu-batch"
  ```
- **Xác thực & Kết quả:** Kiểm chứng đối chiếu ngẫu nhiên F1-Score trên 500 bài viết phân loại nhạy cảm.

---

## Case 079: Chuyển đổi đồ thị tri thức Wikipedia tiếng Việt sang RDF triples (KG Extraction)
- **Tình huống:** Dự án AI mã nguồn mở trích xuất thực thể, thuộc tính và quan hệ (Subject-Predicate-Object) từ toàn bộ bài viết Wikipedia tiếng Việt.
- **Dữ liệu đầu vào:** Bản dump XML Wikipedia `viwiki-latest-pages-articles.xml.bz2` (1.6 GB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0409` (`KnowledgeGraphExtraction`), CPU 32 nhân hoặc GPU NLP NER.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0409 \
    --input ./wiki/viwiki-latest-pages-articles.xml.bz2 \
    --output ./wiki/vietnamese_kg_triples.nt.zst \
    --bounty 7200000000 \
    --auditor-quorum 3 \
    --timeout 14400 \
    --tags "kg,rdf,wikidata,nlp"
  ```
- **Xác thực & Kết quả:** Kiểm tra tính hợp lệ cú pháp N-Triples và độ chính xác của các thực thể liên kết WikiData QID.

---

## Case 080: Tách âm và bóc tách ca từ 1.000 bài hát sang stems không lời (Demucs v4 Audio Separation)
- **Tình huống:** Kênh âm nhạc trực tuyến cần bóc tách giọng hát (Vocals), trống (Drums), bass và nhạc cụ khác từ 1.000 bài hát WAV chất lượng cao để làm nhạc nền karaoke.
- **Dữ liệu đầu vào:** Thư mục 1.000 bài hát nén `songs_wav_1000.tar` (45 GB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x040A` (`DemucsV4StemSeparation`), GPU CUDA VRAM $\ge 12$GB.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x040A \
    --input ./audio/songs_wav_1000.tar \
    --output ./audio/karaoke_stems_1000.tar \
    --bounty 9000000000 \
    --auditor-quorum 2 \
    --timeout 14400 \
    --tags "audio,demucs,stem-separation,gpu"
  ```
- **Xác thực & Kết quả:** Kiểm tra SDR (Signal-to-Distortion Ratio) và bảo toàn phổ năng lượng giữa tổng các stems và bản gốc.

---

## Case 081: Tổng hợp dữ liệu huấn luyện thị giác bằng Ray-tracing (Synthetic Visual Data)
- **Tình huống:** Đội ngũ phát triển xe tự hành cần sinh 20.000 hình ảnh photorealistic cảnh mưa ban đêm kèm bounding box 3D hoàn hảo không tốn công gán nhãn thủ công.
- **Dữ liệu đầu vào:** Kịch bản cấu hình mô phỏng môi trường Blender/Unreal `sim_scene_config.json` (35 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x040B` (`SyntheticDataGenRaytracing`), GPU RTX có RT Cores phần cứng.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x040B \
    --input ./sim/sim_scene_config.json \
    --output ./synthetic_dataset/night_rain_20k.tar \
    --bounty 11500000000 \
    --auditor-quorum 3 \
    --timeout 18000 \
    --tags "synthetic,raytracing,autonomous-vehicle"
  ```
- **Xác thực & Kết quả:** Kiểm toán viên đối chiếu ground-truth 3D bounding box và mask segmentation tương ứng với camera matrix.

---

## Case 082: Đối soát và phân tích đối chiếu sao kê tài chính 10 ngân hàng (Financial Reconciliation)
- **Tình huống:** Cổng thanh toán thương mại điện tử cần đối chiếu 5 triệu giao dịch trong tháng từ 10 ngân hàng đối tác với cơ sở dữ liệu nội bộ để tìm các sai lệch tiền tệ.
- **Dữ liệu đầu vào:** File nén sao kê các ngân hàng `bank_statements_nov2026.tar.gz` (850 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x040C` (`FinancialReconciliation`), TEE Enclave hoặc CPU tính toán an toàn bộ nhớ.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x040C \
    --input ./finance/bank_statements_nov2026.tar.gz \
    --output ./finance/reconciliation_discrepancies.xlsx \
    --bounty 4500000000 \
    --auditor-quorum 3 \
    --tee-required \
    --timeout 3600 \
    --tags "fintech,reconciliation,tee,high-integrity"
  ```
- **Xác thực & Kết quả:** Kiểm tra tính cân đối kép (Double-entry balance) và xuất danh sách các đơn hàng chênh lệch lẻ từng đồng.

---

## Case 083: Chuyển hóa kho sách cổ chữ Hán Nôm sang bản số hóa chuẩn Unicode (Ancient OCR Engine)
- **Tình huống:** Thư viện quốc gia cần số hóa 5.000 trang mộc bản tài liệu Hán Nôm cổ thế kỷ 18 sang chữ Quốc ngữ và Unicode chuẩn.
- **Dữ liệu đầu vào:** Thư mục ảnh chụp độ phân giải 600 DPI `han_nom_pages_5k.tar` (12 GB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x040D` (`AncientHanNomOcr`), GPU Vision Transformer.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x040D \
    --input ./heritage/han_nom_pages_5k.tar \
    --output ./heritage/han_nom_transcribed.json \
    --bounty 6200000000 \
    --auditor-quorum 3 \
    --timeout 7200 \
    --tags "heritage,ocr,han-nom,vision"
  ```
- **Xác thực & Kết quả:** Quorum auditor đối chiếu ký tự chữ tượng hình với từ điển chuẩn Hán Nôm di sản.

---

## Case 084: Xây dựng bản đồ nhiệt phân tích hành vi khách hàng từ 500 camera cửa hàng (Multi-Camera Re-ID)
- **Tình huống:** Chuỗi bán lẻ phân tích luồng di chuyển và mật độ khách ghé thăm các kệ hàng qua 500 luồng video an ninh trích xuất theo giờ.
- **Dữ liệu đầu vào:** File tổng hợp metadata vector nhận diện người `pedestrian_features_500cam.bin` (2.4 GB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x040E` (`MultiCameraReIdHeatmap`), GPU CUDA và RAM $\ge 32$GB.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x040E \
    --input ./retail/pedestrian_features_500cam.bin \
    --output ./retail/store_traffic_heatmaps.json \
    --bounty 5200000000 \
    --auditor-quorum 2 \
    --timeout 5400 \
    --tags "retail,reid,heatmap,vision-tracking"
  ```
- **Xác thực & Kết quả:** Kiểm tra tính liên tục của quỹ đạo người đi bộ không bị phân mảnh hay nhầm lẫn giữa các góc quay.

---

## Case 085: Tối ưu lộ trình logistics giao hàng chặng cuối 50.000 đơn hàng (VRP OR-Tools)
- **Tình huống:** Công ty chuyển phát nhanh cần lập lịch di chuyển tối ưu cho 1.200 tài xế giao 50.000 bưu kiện trong khung giờ cam kết (Time windows) tiết kiệm xăng nhất.
- **Dữ liệu đầu vào:** Ma trận khoảng cách thời gian thực và danh sách đơn `vrp_orders_50k.json` (65 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x040F` (`VrpRouteOptimization`), CPU Đa luồng xung nhịp cao (High Single-Thread/Multi-Thread).
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x040F \
    --input ./logistics/vrp_orders_50k.json \
    --output ./logistics/optimized_driver_routes.json \
    --bounty 4800000000 \
    --auditor-quorum 3 \
    --timeout 3600 \
    --tags "logistics,vrp,or-tools,optimization"
  ```
- **Xác thực & Kết quả:** Kiểm toán viên kiểm tra tính khả thi của mọi lộ trình (không vi phạm tải trọng xe và khung giờ khách yêu cầu). Tiết kiệm trung bình 18% tổng quãng đường di chuyển.

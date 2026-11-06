# Phân Khu 06: Dữ Liệu Lớn & Tìm Kiếm Vector (80 Opcodes)

Phân khu này định nghĩa 80 mã tác vụ chuyên biệt phục vụ các Data Engineer, Nhà khoa học dữ liệu, Hệ thống tìm kiếm RAG và Doanh nghiệp xử lý dữ liệu lớn: từ Chuyển đổi kho dữ liệu dạng cột siêu tốc (Polars / DuckDB Parquet), Khử trùng lặp ngữ nghĩa (MinHash LSH Deduplication), Xây dựng chỉ mục vector quy mô hàng chục triệu chiều (HNSW Faiss / Annoy) cho đến Xử lý tín hiệu chuỗi thời gian (Timeseries DSP & Wavelet).

---

## Danh Mục Chi Tiết 80 Opcodes Dữ Liệu Lớn & Vector Search (`0x6001` – `0x6050`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x6001` | `skynet://data.etl.polars_csv_to_parquet` | Chuyển đổi 100GB CSV sang Snappy Parquet tối ưu dictionary encoding | 16 GB RAM | `capsule-polars-arrow` |
| `0x6002` | `skynet://data.etl.duckdb_sql_aggregation` | Chạy câu truy vấn phân tích OLAP tổng hợp trên 500 triệu bản ghi | 32 GB RAM | `capsule-duckdb-olap` |
| `0x6003` | `skynet://data.dedup.minhash_lsh_500gb` | Khử trùng lặp văn bản 500GB bằng thuật toán MinHash LSH song song | 64 GB RAM | `capsule-minhash-lsh` |
| `0x6004` | `skynet://data.dedup.simhash_near_dup` | Phát hiện văn bản gần giống nhau bằng dấu vân tay 64-bit SimHash | 16 GB RAM | `capsule-simhash-fast` |
| `0x6005` | `skynet://data.vector.hnsw_index_build_10m`| Xây dựng đồ thị tìm kiếm láng giềng HNSW từ 10 triệu vector 1536d | 96 GB RAM | `capsule-hnswlib-fast` |
| `0x6006` | `skynet://data.vector.faiss_ivf_pq_train` | Huấn luyện lượng tử hóa tích phân Faiss IVF-PQ nén 8x bộ nhớ vector | 32 GB RAM | `capsule-faiss-gpu` |
| `0x6007` | `skynet://data.vector.scann_anisotropic` | Xây dựng chỉ mục Google ScaNN tối ưu tích vô hướng cực đại MIPS | 32 GB RAM | `capsule-scann-index` |
| `0x6008` | `skynet://data.vector.annoy_tree_build` | Xây dựng rừng cây nhị phân xẻ không gian Annoy tìm kiếm xấp xỉ | 16 GB RAM | `capsule-annoy-build` |
| `0x6009` | `skynet://data.graph.pagerank_distributed`| Tính toán thứ hạng nút PageRank trên đồ thị mạng xã hội 1 tỷ cạnh | 64 GB RAM | `capsule-graph-pagerank`|
| `0x600A` | `skynet://data.graph.louvain_community` | Phát hiện cộng đồng cụm liên kết Louvain trên đồ thị giao dịch ngân hàng| 32 GB RAM | `capsule-graph-louvain` |
| `0x600B` | `skynet://data.dsp.fft_spectral_100k` | Biến đổi Fourier nhanh (FFT) phân tích phổ tín hiệu rung động 100kHz | 8 GB RAM | `capsule-fftw-simd` |
| `0x600C` | `skynet://data.dsp.wavelet_denoise_ecg` | Lọc nhiễu tín hiệu bằng biến đổi sóng con Wavelet Daubechies | 4 GB RAM | `capsule-wavelet-dsp` |
| `0x600D` | `skynet://data.time.kalman_filter_smoother`| Ước lượng trạng thái thực của phương tiện bằng bộ lọc Kalman mở rộng | 4 GB RAM | `capsule-kalman-filter` |
| `0x600E` | `skynet://data.time.prophet_timeseries_forecast`| Dự báo doanh thu bán lẻ 365 ngày kèm tính chu kỳ ngày trong tuần | 8 GB RAM | `capsule-prophet-ts` |
| `0x600F` | `skynet://data.scrape.distributed_crawler` | Cào bóc dữ liệu và làm sạch 100.000 trang web chống bot phân tán | 16 GB RAM | `capsule-crawler-net` |
| `0x6010` | `skynet://data.scrape.headless_render_pool`| Render JavaScript trang SPA qua cụm Chromium headless nhẹ | 16 GB RAM | `capsule-playwright` |
| `0x6011` | `skynet://data.compress.zstd_level19_pack` | Nén tệp sao lưu dữ liệu lớn bằng thuật toán Zstandard mức 19 | 16 GB RAM | `capsule-zstd-high` |
| `0x6012` | `skynet://data.compress.snappy_fast_stream`| Nén luồng dữ liệu thời gian thực cực nhanh với Snappy cho Kafka | 4 GB RAM | `capsule-snappy-codec` |
| `0x6013` | `skynet://data.schema.json_schema_validate`| Kiểm thực cú pháp và ràng buộc giá trị của 10 triệu bản ghi JSON | 8 GB RAM | `capsule-jsonschema` |
| `0x6014` | `skynet://data.clean.outlier_isolation_forest`| Phát hiện bản ghi ngoại lai bất thường trong dữ liệu cảm biến | 16 GB RAM | `capsule-iso-forest` |
| `0x6015` | `skynet://data.clean.knn_missing_impute` | Điền bù giá trị thiếu (Missing values) bằng thuật toán KNN trọng số | 16 GB RAM | `capsule-knn-imputer` |
| `0x6016` | `skynet://data.clean.fuzzy_string_matching`| Khớp nối tên khách hàng bị sai lỗi gõ phím theo khoảng cách Levenshtein| 8 GB RAM | `capsule-fuzzy-wuzzy` |
| `0x6017` | `skynet://data.clean.phonetic_double_metaphone`| Nhóm các danh từ riêng phát âm giống nhau qua mã Double Metaphone | 4 GB RAM | `capsule-metaphone` |
| `0x6018` | `skynet://data.geo.h3_spatial_indexing` | Gán chỉ mục lục giác không gian Uber H3 cho 50 triệu tọa độ GPS | 16 GB RAM | `capsule-uber-h3` |
| `0x6019` | `skynet://data.geo.s2_geometry_hierarchy` | Phân vùng không gian phân cấp hình cầu Google S2 Geometry | 16 GB RAM | `capsule-google-s2` |
| `0x601A` | `skynet://data.geo.spatial_kdtree_nearest` | Tìm kiếm 10 quán ăn gần nhất cho 1 triệu khách hàng qua KD-Tree | 16 GB RAM | `capsule-kdtree-geo` |
| `0x601B` | `skynet://data.text.tfidf_matrix_sparse` | Xây dựng ma trận thưa TF-IDF từ kho ngữ liệu 1 triệu bài báo | 32 GB RAM | `capsule-tfidf-sparse` |
| `0x601C` | `skynet://data.text.bm25_index_builder` | Lập chỉ mục xếp hạng tài liệu tìm kiếm BM25 chuẩn Okapi | 16 GB RAM | `capsule-bm25-index` |
| `0x601D` | `skynet://data.text.bloom_filter_check` | Kiểm tra sự tồn tại của URL trong danh sách 1 tỷ phần tử (Bloom Filter)| 8 GB RAM | `capsule-bloom-filter` |
| `0x601E` | `skynet://data.text.cuckoo_filter_delete` | Bộ lọc Cuckoo Filter hỗ trợ xóa phần tử với xác suất sai cực nhỏ | 8 GB RAM | `capsule-cuckoo-filter`|
| `0x601F` | `skynet://data.text.hyperloglog_cardinality`| Đếm xấp xỉ số lượng người dùng duy nhất (UV) từ luồng log 100 tỷ event| 4 GB RAM | `capsule-hyperloglog` |
| `0x6020` | `skynet://data.text.count_min_sketch_freq` | Ước lượng tần suất xuất hiện của từ khóa bằng Count-Min Sketch | 4 GB RAM | `capsule-count-min` |
| `0x6021` | `skynet://data.kg.wikidata_sparql_extract` | Trích xuất đồ thị phân tầng WikiData thành bảng quan hệ RDF | 32 GB RAM | `capsule-sparql-kg` |
| `0x6022` | `skynet://data.kg.graph_embedding_node2vec`| Huấn luyện vector biểu diễn các đỉnh đồ thị bằng Node2Vec | 32 GB RAM | `capsule-node2vec` |
| `0x6023` | `skynet://data.kg.trans_e_knowledge_embed`| Học nhúng tri thức dịch chuyển TransE cho bài toán suy diễn quan hệ | 16 GB RAM | `capsule-transe-kg` |
| `0x6024` | `skynet://data.stream.sliding_window_stats`| Tính trung bình trượt và phương sai trên cửa sổ trôi 10 phút | 4 GB RAM | `capsule-sliding-win` |
| `0x6025` | `skynet://data.stream.ewma_trend_detect` | Tính giá trị trung bình động hàm mũ EWMA phát hiện sớm xu hướng | 2 GB RAM | `capsule-ewma-stream` |
| `0x6026` | `skynet://data.cluster.dbscan_density` | Gom cụm mật độ DBSCAN phát hiện các điểm tập trung người bất thường | 16 GB RAM | `capsule-dbscan-fast` |
| `0x6027` | `skynet://data.cluster.kmeans_minibatch` | Phân cụm nhanh 10 triệu khách hàng theo thói quen mua sắm (MiniBatch)| 16 GB RAM | `capsule-kmeans-fast` |
| `0x6028` | `skynet://data.cluster.hdbscan_hierarchical`| Gom cụm phân cấp HDBSCAN tự động tìm số lượng nhóm tối ưu | 32 GB RAM | `capsule-hdbscan` |
| `0x6029` | `skynet://data.dim.pca_svd_dimension_reduce`| Giảm chiều dữ liệu từ 5.000 biến số xuống 50 thành phần chính PCA | 16 GB RAM | `capsule-pca-svd` |
| `0x602A` | `skynet://data.dim.tsne_cuda_visualize` | Chiếu không gian t-SNE 2D để trực quan hóa cụm trên giao diện web | 8 GB VRAM | `capsule-tsne-cuda` |
| `0x602B` | `skynet://data.dim.umap_learn_manifold` | Giữ cấu trúc toàn cục dữ liệu khi chiếu phẳng UMAP 3D | 16 GB RAM | `capsule-umap-fast` |
| `0x602C` | `skynet://data.sql.partition_pruning_reorg`| Sắp xếp lại dữ liệu Parquet theo partition ngày/tháng tăng tốc truy vấn| 32 GB RAM | `capsule-partition-sort`|
| `0x602D` | `skynet://data.sql.delta_lake_compact` | Gom các file nhỏ (Small files problem) trong Delta Lake thành file lớn| 32 GB RAM | `capsule-delta-compact`|
| `0x602E` | `skynet://data.sql.iceberg_metadata_expire`| Xóa bỏ snapshot cũ và dọn dẹp manifest thừa trong bảng Apache Iceberg | 16 GB RAM | `capsule-iceberg-maint`|
| `0x602F` | `skynet://data.recsys.als_matrix_factorize`| Phân rã ma trận người dùng - sản phẩm ALS gợi ý phim cá nhân hóa | 32 GB RAM | `capsule-als-recsys` |
| `0x6030` | `skynet://data.recsys.two_tower_candidate`| Trích xuất vector sở thích người dùng từ mô hình Hai tháp (Two-Tower)| 16 GB RAM | `capsule-twotower-rec` |
| `0x6031` | `skynet://data.recsys.lightgbm_ranker` | Huấn luyện cây quyết định LightGBM xếp hạng kết quả tìm kiếm | 16 GB RAM | `capsule-lightgbm` |
| `0x6032` | `skynet://data.convert.avro_to_parquet` | Chuyển đổi định dạng log nhị phân Apache Avro sang cột Parquet | 8 GB RAM | `capsule-avro-parquet` |
| `0x6033` | `skynet://data.convert.xml_to_jsonl_stream`| Bóc tách file XML bách khoa toàn thư dung lượng 50GB sang file JSONL | 8 GB RAM | `capsule-xml-stream` |
| `0x6034` | `skynet://data.convert.protobuf_decode_batch`| Giải mã hàng triệu gói tin nhị phân Google Protocol Buffers sang CSV | 8 GB RAM | `capsule-proto-codec` |
| `0x6035` | `skynet://data.audit.checksum_sha256_tree`| Tính toán mã băm cây Merkle xác thực tính toàn vẹn của kho dữ liệu | 8 GB RAM | `capsule-merkle-data` |
| `0x6036` | `skynet://data.audit.differential_privacy_laplace`| Thêm nhiễu ngẫu nhiên Laplace bảo đảm an toàn vi sai cho báo cáo thống kê| 4 GB RAM | `capsule-diff-privacy` |
| `0x6037` | `skynet://data.audit.k_anonymity_generalize`| Ẩn danh hóa tập dữ liệu thỏa mãn tiêu chí K-Anonymity (k=5) | 8 GB RAM | `capsule-k-anonymity` |
| `0x6038` | `skynet://data.nlp.vowpal_wabbit_online` | Học trực tuyến tốc độ cực cao với Vowpal Wabbit trên luồng nhấp chuột | 8 GB RAM | `capsule-vowpal-wabbit`|
| `0x6039` | `skynet://data.nlp.fasttext_word_vectors` | Huấn luyện mô hình từ vựng n-gram FastText cho ngôn ngữ mới | 16 GB RAM | `capsule-fasttext` |
| `0x603A` | `skynet://data.nlp.gensim_lda_topic_model`| Khám phá 50 chủ đề ẩn ngầm LDA từ tập tài liệu 100.000 bài viết | 16 GB RAM | `capsule-gensim-lda` |
| `0x603B` | `skynet://data.graph.dijkstra_all_pairs` | Tìm đường đi ngắn nhất giữa tất cả các cặp đỉnh trong đồ thị giao thông| 32 GB RAM | `capsule-all-pairs-sp` |
| `0x603C` | `skynet://data.graph.betweenness_centrality`| Đo độ trung tâm trung gian xác định trạm trung chuyển huyết mạch | 32 GB RAM | `capsule-centrality` |
| `0x603D` | `skynet://data.graph.triangle_counting` | Đếm số lượng tam giác liên kết đo hệ số cụm của mạng xã hội | 16 GB RAM | `capsule-triangle-count`|
| `0x603E` | `skynet://data.time.dtw_sequence_align` | Đo độ tương đồng giữa hai chuỗi thời gian bằng Dynamic Time Warping | 8 GB RAM | `capsule-dtw-align` |
| `0x603F` | `skynet://data.time.stl_decomposition` | Tách chuỗi thời gian thành 3 thành phần: Xu hướng, Mùa vụ, Nhiễu dư | 4 GB RAM | `capsule-stl-decompose`|
| `0x6040` | `skynet://data.time.matrix_profile_motif` | Tìm kiếm các chuỗi mẫu lặp lại (Motifs) và điểm dị thường thời gian | 16 GB RAM | `capsule-matrix-profile`|
| `0x6041` | `skynet://data.spatial.delaunay_triangulate`| Tam giác hóa Delaunay tạo lưới mạng đa giác phẳng từ tập điểm | 8 GB RAM | `capsule-delaunay-2d` |
| `0x6042` | `skynet://data.spatial.convex_hull_monotone`| Tìm bao lồi không gian nhỏ nhất chứa tập điểm tọa độ | 4 GB RAM | `capsule-convex-hull` |
| `0x6043` | `skynet://data.spatial.r_tree_spatial_index`| Lập chỉ mục cây R-Tree phục vụ truy vấn vùng giao cắt hình chữ nhật | 8 GB RAM | `capsule-rtree-spatial`|
| `0x6044` | `skynet://data.sort.tim_sort_parallel` | Sắp xếp mảng hỗn hợp 100 triệu phần tử siêu nhanh bằng thuật toán Timsort| 16 GB RAM | `capsule-parallel-sort`|
| `0x6045` | `skynet://data.sort.radix_sort_integers` | Sắp xếp số nguyên 64-bit theo cơ số không dùng so sánh Radix Sort | 8 GB RAM | `capsule-radix-sort` |
| `0x6046` | `skynet://data.hash.murmurhash3_stream` | Băm 64-bit tốc độ cao không mật mã cho bảng băm phân tán Distributed DHT| 2 GB RAM | `capsule-murmur3` |
| `0x6047` | `skynet://data.hash.xxhash64_fast_digest`| Tính mã băm cực nhanh 10 GB/giây để kiểm tra đối soát file trùng | 2 GB RAM | `capsule-xxhash` |
| `0x6048` | `skynet://data.io.direct_io_nvme_benchmark`| Đo băng thông đọc ghi thực tế I/O của dàn ổ đĩa NVMe U.2 | 4 GB RAM | `capsule-fio-bench` |
| `0x6049` | `skynet://data.sql.ch_benchmark_olap` | Đánh giá năng lực xử lý phân tích dữ liệu dạng cột OLAP | 16 GB RAM | `capsule-olap-bench` |
| `0x604A` | `skynet://data.log.grok_pattern_parse` | Bóc tách log máy chủ Nginx/Apache theo biểu thức chính quy Grok | 8 GB RAM | `capsule-grok-parser` |
| `0x604B` | `skynet://data.log.vector_log_router` | Phân luồng và biến đổi luồng log thời gian thực chuyển về kho lưu trữ| 4 GB RAM | `capsule-vector-log` |
| `0x604C` | `skynet://data.format.orc_stripe_optimize` | Tối ưu hóa kích thước Stripe trong file Apache ORC tăng tốc truy vấn| 16 GB RAM | `capsule-orc-optimize` |
| `0x604D` | `skynet://data.format.feather_ipc_stream` | Giao tiếp chia sẻ bộ nhớ RAM giữa Python và Rust không cần copy | 8 GB RAM | `capsule-arrow-ipc` |
| `0x604E` | `skynet://data.fuzzy.bk_tree_spell_check` | Xây dựng cây Burkhard-Keller tìm kiếm từ vựng sai chính tả trong từ điển | 4 GB RAM | `capsule-bk-tree` |
| `0x604F` | `skynet://data.clean.dedup_phone_standardize`| Chuẩn hóa định dạng số điện thoại quốc tế E.164 và xóa trùng lặp danh bạ| 2 GB RAM | `capsule-phone-clean` |
| `0x6050` | `skynet://data.vector.quantization_sq8` | Nén vector số thực FP32 sang số nguyên INT8 tiết kiệm 75% RAM | 16 GB RAM | `capsule-vector-sq8` |

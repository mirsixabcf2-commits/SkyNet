# Phân Khu 07: Mật Mã, Zero-Knowledge & Điện Toán Riêng Tư (80 Opcodes)

Phân khu này định nghĩa 80 mã tác vụ chuyên biệt phục vụ các Kỹ sư mật mã học, Giao thức Web3, Kiểm toán an ninh mạng và Doanh nghiệp bảo mật dữ liệu: từ Sinh bằng chứng không tiết lộ tri thức (ZK-SNARK / ZK-STARK Provers), Mật mã đồng cấu toàn phần (Fully Homomorphic Encryption - FHE), Tính toán đa bên an toàn (MPC) cho đến Môi trường thực thi tin cậy phần cứng (Hardware TEE Enclaves) và Kiểm thử đột biến bảo mật (Fuzzing).

---

## Danh Mục Chi Tiết 80 Opcodes Mật Mã & ZK/Privacy (`0x7001` – `0x7050`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x7001` | `skynet://crypto.zk.groth16_prover` | Sinh bằng chứng mật mã ZK-SNARK Groth16 trên đường cong BN254 | 32 GB RAM | `capsule-groth16-prover`|
| `0x7002` | `skynet://crypto.zk.plonk_ultra_prover` | Sinh bằng chứng Plonk với Custom Gates và Plookup | 64 GB RAM | `capsule-plonk-prover` |
| `0x7003` | `skynet://crypto.zk.plonky2_fri_stark` | Sinh bằng chứng ZK-STARK đệ quy siêu tốc trong 0.2 giây (Plonky2)| 32 Core CPU | `capsule-plonky2-stark`|
| `0x7004` | `skynet://crypto.zk.halo2_ipa_kzg` | Sinh bằng chứng Halo2 không cần Trusted Setup qua cam kết KZG | 64 GB RAM | `capsule-halo2-kzg` |
| `0x7005` | `skynet://crypto.zk.cairo_vm_trace_prover`| Sinh bằng chứng tính toán hợp lệ của chương trình Cairo VM STARK | 128 GB RAM | `capsule-cairo-prover` |
| `0x7006` | `skynet://crypto.zk.risc0_zkvm_receipt` | Sinh biên lai chứng minh thực thi mã nguồn Rust chuẩn trên RISC Zero| 64 GB RAM | `capsule-risc0-zkvm` |
| `0x7007` | `skynet://crypto.zk.sp1_zkvm_prover` | Tạo bằng chứng thực thi máy ảo Succinct SP1 ZK-VM | 64 GB RAM | `capsule-sp1-zkvm` |
| `0x7008` | `skynet://crypto.zk.merkle_tree_batch_proof`| Tạo bằng chứng cây Merkle 32 tầng cho hàng triệu tài khoản Rollup | 16 GB RAM | `capsule-merkle-prover`|
| `0x7009` | `skynet://crypto.fhe.ckks_vector_multiply`| Tính toán nhân ma trận mã hóa số thực xấp xỉ bằng mật mã đồng cấu CKKS| 32 GB RAM | `capsule-seal-ckks` |
| `0x700A` | `skynet://crypto.fhe.bfv_integer_arithmetic`| Tính toán cộng trừ nhân trên số nguyên mã hóa chính xác tuyệt đối BFV| 32 GB RAM | `capsule-seal-bfv` |
| `0x700B` | `skynet://crypto.fhe.tfhe_boolean_gate` | Đánh giá cổng logic NAND/XOR trên bit mã hóa với tính năng Bootstrapping| 16 GB RAM | `capsule-tfhe-rs` |
| `0x700C` | `skynet://crypto.mpc.yao_garbled_circuit`| Sinh và đánh giá mạch xáo trộn Yao Garbled Circuit cho 2 bên | 16 GB RAM | `capsule-yao-mpc` |
| `0x700D` | `skynet://crypto.mpc.spdz_secret_sharing`| Giao thức chia sẻ bí mật SPDZ tính toán đa bên an toàn chống gian lận | 32 GB RAM | `capsule-spdz-mpc` |
| `0x700E` | `skynet://crypto.tss.frost_threshold_sign`| Ký ngưỡng phân tán FROST Schnorr Ed25519 (t-out-of-n) không lộ khóa | 4 GB RAM | `capsule-frost-tss` |
| `0x700F` | `skynet://crypto.tss.gg20_ecdsa_sign` | Ký ngưỡng đa bên phân tán ECDSA secp256k1 cho ví lạnh tổ chức | 8 GB RAM | `capsule-gg20-tss` |
| `0x7010` | `skynet://crypto.tee.intel_sgx_attestation`| Tạo và thẩm định chứng thực phần cứng Intel SGX Enclave Quote | 16 GB TEE | `capsule-sgx-attest` |
| `0x7011` | `skynet://crypto.tee.amd_sev_snp_validate`| Xác thực tính nguyên bản máy ảo bí mật AMD SEV-SNP Confidential VM | 32 GB TEE | `capsule-sev-snp` |
| `0x7012` | `skynet://crypto.tee.aws_nitro_enclave_run`| Chạy mã nguồn nhạy cảm bên trong môi trường cô lập AWS Nitro Enclave| 16 GB TEE | `capsule-nitro-enclave`|
| `0x7013` | `skynet://crypto.fuzz.aflplusplus_mutator` | Kiểm thử đột biến phần mềm tìm lỗi tràn bộ nhớ bằng AFL++ | 16 Core CPU | `capsule-aflpp-fuzz` |
| `0x7014` | `skynet://crypto.fuzz.echidna_solidity` | Fuzzing đột biến sâu 100 triệu biến thể tìm lỗi Smart Contract Solidity| 32 GB RAM | `capsule-echidna-fuzz` |
| `0x7015` | `skynet://crypto.fuzz.honggfuzz_hardware`| Fuzzing dựa trên phản hồi bộ đếm phần cứng CPU Intel PT | 16 Core CPU | `capsule-honggfuzz` |
| `0x7016` | `skynet://crypto.audit.slither_solidity_ast`| Quét tĩnh phát hiện 80 lỗ hổng logic tái nhập (Reentrancy) hợp đồng | 8 GB RAM | `capsule-slither-ast` |
| `0x7017` | `skynet://crypto.audit.mythril_symbolic` | Phân tích thực thi biểu tượng tìm đường dẫn tấn công đánh cắp tiền | 16 GB RAM | `capsule-mythril-sym` |
| `0x7018` | `skynet://crypto.audit.manticore_binary` | Giải ràng buộc toán học tìm lỗi phân nhánh trong mã nhị phân x86/ARM | 32 GB RAM | `capsule-manticore` |
| `0x7019` | `skynet://crypto.postquantum.kyber_kem_key`| Sinh cặp khóa và đóng gói khóa mật mã hậu lượng tử ML-KEM (Kyber) | 2 GB RAM | `capsule-pqc-kyber` |
| `0x701A` | `skynet://crypto.postquantum.dilithium_sign`| Tạo chữ ký số chống máy tính lượng tử ML-DSA (Dilithium) | 2 GB RAM | `capsule-pqc-dilithium`|
| `0x701B` | `skynet://crypto.postquantum.sphincs_hash`| Chữ ký số dựa trên hàm băm SLH-DSA (SPHINCS+) kích thước nhỏ | 4 GB RAM | `capsule-pqc-sphincs` |
| `0x701C` | `skynet://crypto.hash.poseidon_zk_friendly`| Băm mã hóa thân thiện với mạch ZK Poseidon trên trường hữu hạn Fr | 8 GB RAM | `capsule-poseidon-hash`|
| `0x701D` | `skynet://crypto.hash.rescue_prime_zk` | Hàm băm đại số Rescue-Prime tối ưu số lượng ràng buộc Plonk | 8 GB RAM | `capsule-rescue-hash` |
| `0x701E` | `skynet://crypto.hash.blake3_avx512_digest`| Tính mã băm BLAKE3 với tốc độ tối đa băng thông bộ nhớ AVX-512 | 2 GB RAM | `capsule-blake3-simd` |
| `0x701F` | `skynet://crypto.hash.argon2id_password_kdf`| Hàm dẫn xuất khóa kháng GPU/ASIC Argon2id bảo vệ mật khẩu người dùng | 8 GB RAM | `capsule-argon2id` |
| `0x7020` | `skynet://crypto.crack.hashcat_benchmark` | Đánh giá độ mạnh của mật khẩu doanh nghiệp qua kiểm thử từ điển | 16 GB VRAM | `capsule-hashcat-eval` |
| `0x7021` | `skynet://crypto.curve.bls12_381_pairings` | Tính toán cặp song tuyến tính Bilinear Pairing trên đường cong BLS12-381| 8 GB RAM | `capsule-bls12-pairing`|
| `0x7022` | `skynet://crypto.curve.ed25519_batch_verify`| Xác thực theo lô 100.000 chữ ký Ed25519 trong chưa đầy 1 giây | 8 GB RAM | `capsule-ed25519-batch`|
| `0x7023` | `skynet://crypto.curve.secp256k1_ecdsa_mult`| Nhân vô hướng điểm trên đường cong elip Bitcoin/Ethereum siêu tốc | 4 GB RAM | `capsule-secp256k1` |
| `0x7024` | `skynet://crypto.vdf.wesolowski_verifiable`| Tạo và kiểm tra hàm trễ có thể kiểm chứng Verifiable Delay Function | 16 GB RAM | `capsule-vdf-wesolowski`|
| `0x7025` | `skynet://crypto.vdf.pietrzak_fast_verify` | Xác thực bằng chứng hàm trễ thời gian Pietrzak chỉ tốn vài miligiây | 4 GB RAM | `capsule-vdf-pietrzak` |
| `0x7026` | `skynet://crypto.vrf.ecvrf_random_beacon` | Sinh số ngẫu nhiên công bằng có thể kiểm chứng được (Verifiable Random)| 2 GB RAM | `capsule-ecvrf-beacon` |
| `0x7027` | `skynet://crypto.onion.tor_v3_vanity_addr` | Đào địa chỉ vanity ẩn danh .onion 56 ký tự cho dịch vụ bảo mật | 16 GB VRAM | `capsule-onion-vanity` |
| `0x7028` | `skynet://crypto.tls.certificate_transparency`| Quét cây Merkle kiểm tra chứng chỉ SSL/TLS bị cấp phát gian lận | 8 GB RAM | `capsule-cert-trans` |
| `0x7029` | `skynet://crypto.blind.rsa_blind_signature`| Ký mù RSA bảo vệ tính riêng tư trong hệ thống bỏ phiếu điện tử | 4 GB RAM | `capsule-rsa-blind` |
| `0x702A` | `skynet://crypto.ring.ring_confidential_tx` | Ký vòng RingCT ẩn danh số tiền và người gửi (công nghệ Monero) | 8 GB RAM | `capsule-ringct-tx` |
| `0x702B` | `skynet://crypto.mix.coinjoin_whirlpool` | Điều phối phiên gom giao dịch CoinJoin cắt đứt liên kết dấu vết on-chain| 8 GB RAM | `capsule-coinjoin-mix` |
| `0x702C` | `skynet://crypto.commit.pedersen_vector` | Cam kết ẩn số Pedersen Vector Commitment bảo toàn tính cộng đồng cấu | 8 GB RAM | `capsule-pedersen-comm`|
| `0x702D` | `skynet://crypto.commit.kzg_polynomial` | Mở cam kết đa thức KZG trong công nghệ EIP-4844 Proto-Danksharding | 16 GB RAM | `capsule-kzg-poly` |
| `0x702E` | `skynet://crypto.commit.ipa_inner_product` | Cam kết tích trong IPA (Inner Product Argument) không cần trusted setup| 16 GB RAM | `capsule-ipa-commit` |
| `0x702F` | `skynet://crypto.zk.snark_setup_powers_of_tau`| Đóng góp ngẫu nhiên vào nghi thức Ceremony Powers-of-Tau cho ZK | 32 GB RAM | `capsule-powers-of-tau`|
| `0x7030` | `skynet://crypto.zk.circom_r1cs_compile` | Biên dịch mạch logic Circom sang hệ thống ràng buộc bậc nhất R1CS | 16 GB RAM | `capsule-circom-r1cs` |
| `0x7031` | `skynet://crypto.zk.witness_generate_cpp`| Tính toán nhân chứng (Witness Generation) từ tham số đầu vào mạch | 16 GB RAM | `capsule-witness-calc` |
| `0x7032` | `skynet://crypto.zk.novas_folding_scheme` | Gấp các bằng chứng đệ quy liên tục bằng lược đồ Nova Folding | 32 GB RAM | `capsule-nova-folding` |
| `0x7033` | `skynet://crypto.zk.zk_regex_email_verify`| Chứng minh nội dung email DKIM chính xác mà không tiết lộ thông tin cá nhân| 16 GB RAM | `capsule-zk-regex` |
| `0x7034` | `skynet://crypto.zk.zk_passport_icao9303` | Xác thực hộ chiếu điện tử gắn chip NFC bằng ZK chứng minh trên 18 tuổi | 16 GB RAM | `capsule-zk-passport` |
| `0x7035` | `skynet://crypto.zk.dark_forest_fog_of_war`| Tính toán tọa độ hành tinh trong game ẩn thông tin Dark Forest | 8 GB RAM | `capsule-zk-game` |
| `0x7036` | `skynet://crypto.tee.confidential_sql_db` | Thực thi truy vấn SQL trên cơ sở dữ liệu mã hóa bên trong Enclave | 32 GB TEE | `capsule-conf-sql` |
| `0x7037` | `skynet://crypto.tee.key_vault_sealing` | Niêm phong khóa bí mật gắn chặt với cấu hình phần cứng CPU (Sealing) | 8 GB TEE | `capsule-tee-seal` |
| `0x7038` | `skynet://crypto.audit.decompiler_ghidra_headless`| Dịch ngược mã máy nhị phân sang mã giả C bằng Ghidra phân tích mã độc | 16 GB RAM | `capsule-ghidra-head` |
| `0x7039` | `skynet://crypto.audit.yara_rule_scanner` | Quét tập tin tìm kiếm chữ ký mã độc và virus máy tính theo luật YARA | 8 GB RAM | `capsule-yara-scan` |
| `0x703A` | `skynet://crypto.audit.semgrep_sast_rules`| Phân tích tĩnh mã nguồn SAST phát hiện lỗ hổng OWASP Top 10 | 8 GB RAM | `capsule-semgrep-sast` |
| `0x703B` | `skynet://crypto.audit.dependency_trivy_vuln`| Quét lỗ hổng bảo mật CVE trong các thư viện phụ thuộc package npm/cargo | 4 GB RAM | `capsule-trivy-audit` |
| `0x703C` | `skynet://crypto.audit.container_grype_sbom`| Phân tích hóa đơn nguyên vật liệu phần mềm SBOM của container image | 4 GB RAM | `capsule-grype-sbom` |
| `0x703D` | `skynet://crypto.network.suricata_ids_eval`| Phân tích gói tin PCAP phát hiện dấu hiệu xâm nhập mạng Suricata IDS | 16 GB RAM | `capsule-suricata-pcap`|
| `0x703E` | `skynet://crypto.network.zeek_bro_protocol_extract`| Bóc tách siêu dữ liệu giao thức HTTP/DNS/TLS từ lưu lượng mạng Zeek | 16 GB RAM | `capsule-zeek-parser` |
| `0x703F` | `skynet://crypto.network.nmap_stealth_syn_parse`| Phân tích kết quả quét cổng mạng phát hiện dịch vụ mở nguy hiểm | 4 GB RAM | `capsule-nmap-parser` |
| `0x7040` | `skynet://crypto.forensic.volatility_mem_dump`| Trích xuất danh sách tiến trình ngầm từ tệp dump bộ nhớ RAM hệ điều hành | 16 GB RAM | `capsule-volatility-ram`|
| `0x7041` | `skynet://crypto.forensic.autopsy_disk_carve`| Khôi phục tệp tin đã bị xóa trên phân vùng ổ đĩa nghi phạm | 16 GB RAM | `capsule-autopsy-carve`|
| `0x7042` | `skynet://crypto.forensic.pcap_voip_call_reconstruct`| Tái tạo cuộc gọi thoại VoIP RTP từ gói tin mạng bắt được | 8 GB RAM | `capsule-voip-pcap` |
| `0x7043` | `skynet://crypto.honey.honeypot_cowrie_log`| Tổng hợp các lệnh kẻ tấn công gõ vào máy chủ SSH ảo bẫy (Cowrie) | 4 GB RAM | `capsule-cowrie-logs` |
| `0x7044` | `skynet://crypto.threat.misp_ioc_correlation`| Tương quan các dấu hiệu thỏa hiệp IoC với cơ sở dữ liệu tình báo MISP | 16 GB RAM | `capsule-misp-ioc` |
| `0x7045` | `skynet://crypto.secret.vault_shamir_unseal`| Khôi phục khóa gốc kho bảo mật từ 3 trong 5 mảnh khóa Shamir | 2 GB RAM | `capsule-shamir-vault` |
| `0x7046` | `skynet://crypto.secret.tpm2_nvram_read` | Giao tiếp với chip phần cứng TPM 2.0 đo đạc trạng thái khởi động PCR | 2 GB RAM | `capsule-tpm2-tools` |
| `0x7047` | `skynet://crypto.stego.steghide_extract_hidden`| Phát hiện và bóc tách thông điệp mật giấu trong ảnh JPEG/WAV | 4 GB RAM | `capsule-steghide-tool`|
| `0x7048` | `skynet://crypto.stego.aletheia_steganalysis`| Phân tích thống kê xác định bức ảnh có chứa dữ liệu ẩn giấu hay không| 8 GB RAM | `capsule-aletheia-steg`|
| `0x7049` | `skynet://crypto.ransom.crypto_extension_alert`| Giám sát tốc độ mã hóa file bất thường cảnh báo tấn công mã độc tống tiền| 4 GB RAM | `capsule-ransom-detect`|
| `0x704A` | `skynet://crypto.proxy.reencryption_umbral`| Tái mã hóa dữ liệu ủy quyền chuyển tiếp Umbral không để lộ khóa bí mật| 8 GB RAM | `capsule-umbral-pre` |
| `0x704B` | `skynet://crypto.fhe.openfhe_context_gen` | Khởi tạo không gian tham số đa thức và khóa đánh giá mật mã OpenFHE | 32 GB RAM | `capsule-openfhe-core` |
| `0x704C` | `skynet://crypto.zk.snark_batch_verifier` | Xác minh hợp lệ cùng lúc 10.000 bằng chứng Groth16 trong vài giây | 16 GB RAM | `capsule-batch-zk-verify`|
| `0x704D` | `skynet://crypto.tee.secure_shuffle_deck` | Xáo bài ngẫu nhiên hoàn toàn bảo mật và công bằng cho sòng bạc online | 4 GB TEE | `capsule-tee-shuffle` |
| `0x704E` | `skynet://crypto.audit.formal_verification_dafny`| Chứng minh toán học tính đúng đắn tuyệt đối của mã nguồn bằng Dafny | 16 GB RAM | `capsule-dafny-verify` |
| `0x704F` | `skynet://crypto.audit.certora_prover_solidity`| Kiểm tra vi phạm bất biến bảo mật tài chính bằng Certora Prover | 32 GB RAM | `capsule-certora-prover`|
| `0x7050` | `skynet://crypto.zk.lookup_table_generator` | Sinh bảng tra cứu trước Plookup cho hàm băm Keccak256 trong zkEVM | 32 GB RAM | `capsule-zk-lookups` |

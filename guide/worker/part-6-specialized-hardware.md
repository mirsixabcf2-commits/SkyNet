# Tuyển Tập Worker 089 – 100: Phần Cứng Chuyên Dụng, TEE & Nút Kiểm Toán Cấp Cao (Specialized Hardware, TEE & Master Auditors)

Trong kiến trúc của SkyNet, các tác vụ liên quan đến bí mật y tế bệnh viện, đối soát ngân hàng, bằng chứng mật mã không tiết lộ tri thức (ZK-Proof) và trọng tài phân xử tranh chấp đòi hỏi những tiêu chuẩn phần cứng khắt khe nhất. 

Các nút sở hữu **Môi trường Thực thi Tin cậy (Trusted Execution Environment - TEE)**, **Thẻ gia tốc phần cứng FPGA/ASIC** hoặc **Ký quỹ cổ phần lớn (Staked Master Auditors)** được hưởng các đặc quyền nhận những gói bounty giá trị cao nhất trên toàn mạng lưới.

---

## Bảng Tổng Hợp Thông Số & Lệnh Worker 089 – 100

| Case | Loại Phần Cứng Đặc Biệt | Công Nghệ Bảo Mật Cốt Lõi | Tác Vụ Độc Quyền (Opcodes) | Lệnh Khởi Chạy Nút | Mức Thù Lao Kỷ Lục |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **089** | Intel Xeon Scalable (Ice/Sapphire Lake)| Intel SGX Enclave | `0x0406` (Khử PII Y Tế Bệnh Viện HIPAA) | `skynet-agent --tee-type intel-sgx --enclave-mem 64G` | 50 – 90 SKN / Job |
| **090** | AMD EPYC Genoa 9004 | AMD SEV-SNP | `0x0503` (Secure Federated Learning Ngân Hàng)| `skynet-agent --tee-type amd-sev-snp --verify-attestation`| 60 – 110 SKN / Job |
| **091** | Nút Staking Master Auditor | Ký quỹ $\ge 50.000$ SKN | Thẩm định & Phê duyệt mọi giải ngân Escrow | `skynet-agent --master-auditor --stake-bond 50000000000000` | 15% Chia đều toàn mạng |
| **092** | Thẻ FPGA Xilinx Alveo U50 | Mạch logic nạp bitstream ZK | `0x0209` (Gia tốc Merkle & ZK-STARK Prover) | `skynet-agent --fpga-device /dev/xfpga0 --tags "zk-fpga"`| 40 – 75 SKN / Job |
| **093** | YubiHSM2 / Nitrokey HSM | Phần cứng bảo vệ khóa Ed25519| Ký xác thực khối Ledger & Giao dịch Soul-Body | `skynet-agent --hsm-pkcs11 /usr/lib/libykcs11.so` | Bảo mật tuyệt đối |
| **094** | Nút Kiểm Toán UTXO-DAG | RAM ECC + NVMe RAID 10 | Quét liên tục chống Double-spend và rửa tiền | `skynet-agent --dag-auditor --verify-utxo-invariants` | Nhận phí giao dịch mạng |
| **095** | Tenstorrent Wormhole n150 | Kiến trúc Tensix AI Processor| Tính toán ma trận thưa và suy luận LLM mở | `skynet-agent --backend tt-metal --tags "tenstorrent"` | 35 – 60 SKN / Job |
| **096** | Nút Kiểm Toán AI Safety | Sandbox kiểm tra mã độc | Quét phát hiện Prompt Injection / Malicious Code| `skynet-agent --safety-auditor --sandbox-isolate` | 20 – 40 SKN / Job |
| **097** | AWS Nitro Enclaves / GCP CVM | Điện toán đám mây bảo mật cao | Cầu nối thanh toán giữa Web2 và SkyNet DAG | `skynet-agent --nitro-enclave-mode` | Phí tích hợp doanh nghiệp |
| **098** | Trạm Kiểm Toán Sinh Học Phân Tử| Đối chiếu RMSD và pLDDT | Kiểm toán tính khả thi cấu trúc AlphaFold3 | `skynet-agent --bio-auditor --pdb-validator` | 30 – 50 SKN / Job |
| **099** | Nút Thẩm Tra DeFi Batch Solvers | Sandbox mô phỏng EVM/Solana | Chống giải thuật ép giá và trục lợi MEV Swap | `skynet-agent --defi-auditor --simulate-mempool` | 25 – 45 SKN / Job |
| **100** | Genesis Quorum Validator Node | Đồng thuận Snowball Hạt Nhân | Dẫn dắt tạo khối và neo nhánh DAG toàn cầu | `skynet-agent --genesis-validator --snowball-k 20` | Phần thưởng đúc tiền Halving |

---

## Chi Tiết Kỹ Thuật Các Nút Thượng Tầng & Môi Trường Tin Cậy (TEE)

### Case 089 & 090: Môi Trường Thực Thi Tin Cậy TEE (Intel SGX & AMD SEV-SNP)
- **Bối cảnh:** Khi bệnh viện gửi 100.000 hồ sơ bệnh án để xử lý hoặc ngân hàng tổng hợp mô hình tín dụng, dữ liệu chứa bí mật thương mại và thông tin cá nhân tối mật. Nếu chạy trên máy tính thông thường, người quản trị máy (root admin) có thể dump bộ nhớ RAM để đánh cắp dữ liệu.
- **Cơ chế bảo vệ TEE:**
  1. Dữ liệu khi gửi vào được mã hóa trực tiếp bằng khóa công khai của Enclave phần cứng bên trong chip CPU.
  2. Hệ điều hành máy chủ và Admin không thể đọc được vùng nhớ này.
  3. Chip CPU tạo một chứng thực phần cứng (**Attestation Quote**) kèm chữ ký gốc của Intel/AMD để chứng minh mã nguồn đang chạy trong Enclave là nguyên bản chưa bị sửa đổi.
- **Lệnh điều phối cho thợ đào TEE:**
  ```bash
  skynet-agent \
    --listen 0.0.0.0:8888 \
    --auto-bid \
    --tee-type intel-sgx \
    --enclave-mem 64G \
    --require-attestation \
    --tags "tee,hipaa,medical,confidential" \
    --min-bounty-rate 12.0 \
    --wallet-path /etc/skynet/secure_wallet.json
  ```
- **Kinh tế:** Do số lượng máy chủ TEE chuẩn công nghiệp trên mạng lưới ít hơn GPU tiêu dùng thông thường trong khi nhu cầu của các tập đoàn y tế và tài chính rất lớn, mức bounty chi trả cho các nút TEE cao gấp 3 đến 5 lần các tác vụ tính toán thông thường.

### Case 091: Trở Thành "Master Auditor" (Kiểm Toán Viên Trưởng)
- **Điều kiện:**
  - Nút phải khóa một khoản tiền bảo chứng (**Staked Bond**) tối thiểu `50.000 SKN` vào hợp đồng lưu ký `Soul-Body Custody`.
  - Nếu Auditor ký duyệt sai hoặc cấu kết gian lận với Worker để chiếm đoạt tiền Escrow, số tiền ký quỹ sẽ bị tịch thu (Slashing) ngay lập tức theo cơ chế phạt của giao thức.
- **Quyền lợi:**
  - Tự động nhận **15%** tổng bounty của tất cả các nhiệm vụ mà nút được chọn vào Ban thẩm tra (Auditor Quorum).
  - Thu nhập đều đặn, ổn định và hầu như không phụ thuộc vào chu kỳ card đồ họa hay biến động giá điện.
- **Lệnh vận hành:**
  ```bash
  skynet-agent \
    --master-auditor \
    --stake-bond 50000000000000 \
    --listen 0.0.0.0:8888 \
    --quorum-concurrency 16 \
    --auto-sign-valid
  ```

### Case 100: Genesis Quorum Validator – Trụ Cột Duy Trì Sự Tồn Tại Mạng Lưới
- **Vai trò:**
  Nút tham gia trực tiếp vào tầng đồng thuận xác suất **Snowball** (trong crate `skynet-agent`), định tuyến các xung lượng thần kinh (`ImpulseBody`), lắng nghe các giao dịch chuyển tiền `NanoSkn`, duy trì đồ thị phi chu trình có hướng UTXO-DAG và phân phối phần thưởng đúc tiền theo chu kỳ **Halving** (trong crate `skynet-ledger`).
- **Khởi chạy nút:**
  ```bash
  skynet-agent \
    --genesis-validator \
    --listen 0.0.0.0:8888 \
    --snowball-k 20 \
    --snowball-alpha 0.8 \
    --snowball-beta 15 \
    --peer-table-capacity 2048 \
    --log-metrics-ws 0.0.0.0:9999
  ```
- **Giá trị cốt lõi:** Nút được cộng đồng tôn vinh, nhận phí giao dịch mạng lưới và định hình tương lai tự chủ kinh tế của mạng lưới đa bào AI SkyNet.

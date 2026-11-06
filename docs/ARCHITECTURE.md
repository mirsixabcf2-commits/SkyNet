# Kiến trúc Hệ thống SkyNet (v2.1): Mạng lưới AI Agent Đa bào & Sổ cái SKN trên Mạng UDP

## 1. Triết lý Cốt lõi
- **1 Node = 1 AI Agent (Cơ thể sống đa bào):** Tự động co giãn quy mô quần thể tế bào theo sức mạnh phần cứng (từ thiết bị ngoại vi CPU yếu vài chục tế bào tới máy chủ đa GPU hàng vạn tế bào).
- **1 Tế bào (`MicroCell`) = 1 AI Model siêu mini (`candle`):** Mạng nơ-ron trạng thái lỏng (Liquid Micro-Net) có bộ nhớ ngắn hạn, chạy được trên cả CPU SIMD lẫn GPU (`Metal` / `CUDA` qua `Batched Tissue Tensor`).
- **Tế bào gốc & Biệt hóa (`Epigenetic Differentiation`):** Mọi tế bào khởi sinh từ `StemCell` mang chung `CellDna`, tự biệt hóa thành 5 dòng mô học:
  1. `Stem` (Tế bào gốc dự trữ & tái tạo)
  2. `Epidermal` (Tế bào da & miễn dịch: chấm điểm bất thường gói tin UDP biên)
  3. `Vascular` (Tế bào máu: điều phối năng lượng `nanoSKN` & tín hiệu nội bộ)
  4. `Visceral` (Tế bào nội tạng: nhịp xung, xác thực sổ cái UTXO-DAG, dọn rác tự thực)
  5. `Neural` (Tế bào thần kinh: nhận thức, học tập & ra quyết định cho Agent)
- **Tiến hóa Sinh học:** Kế thừa DNA có kiểm tra toàn vẹn `BLAKE3`, phân tách `1 -> 2` (`Mitosis`), đột biến trọng số & siêu tham số (`Neuro-evolution`), trao đổi gen ngang (`Plasmid`) và đào thải tế bào suy yếu (`Apoptosis`).
- **Mạng UDP Chấp nhận Mất gói (`<= 1200 Bytes`):** Không dùng trạng thái gửi lại kiểu TCP; đồng thuận lấy mẫu xác suất **Snowball/Avalanche** chịu rớt gói tự nhiên và truyền dữ liệu lớn qua **LT Fountain Codes**.

## 2. Kinh tế học Đồng SKN
- **Tổng cung tối đa cố định tại Genesis:** `21,000,000,000,000 SKN` (21 nghìn tỉ SKN).
- **Đơn vị giao dịch & năng lượng tối thiểu:** `1 nanoSKN` (`1 SKN = 1,000,000,000 nanoSKN`).
- **Biểu diễn số học:** `NanoSkn(u128)` (`21_000_000_000_000_000_000_000 nanoSKN`, an toàn chống tràn số vượt xa giới hạn `u64`).
- **Chu kỳ Halving:** Giảm `50%` lượng phân phối từ Kho Dự trữ Genesis sau mỗi `1 năm`.
- **Bảo toàn Năng lượng Hai tầng:** Tổng `nanoSKN` trong Kho Dự trữ + trên chuỗi UTXO-DAG + dòng máu nội bộ + tổng năng lượng từng tế bào AI luôn bằng đúng `21 nghìn tỉ SKN`.
- **Cân Bằng Kinh Tế (Node Miner vs. Người Dùng):** Giải quyết xung đột lợi ích bằng kênh nén $M:1$ qua `SynapticVoucher` (phí cận 0 cho mỗi lượt gọi), nguồn thu kép từ Điện toán hữu ích (Useful Compute: suy luận, tiến hóa gen, chợ Plasmid), và Bể dưỡng chất cụm (`Cluster Nutrient Pool`). Chi tiết xem tại [TECHNOLOGY_EXPLAINER.md (Mục 7)](file:///Users/pain/DATA/Projects/SkyNet/docs/TECHNOLOGY_EXPLAINER.md).

## 3. Cấu trúc Workspace
- `crates/skynet-dna`: Mã di truyền `CellDna`, `ConservedGenes`, `AdaptiveGenes`, `EpigeneticMask`, nhân bản, đột biến & lai ghép gen.
- `crates/skynet-cell`: Mô hình AI siêu mini dùng `candle-core`, biệt hóa tế bào gốc, thực thi lô `TissueBatch` CPU/GPU, phân tách `1 -> 2` & tự hủy `Apoptosis`.
- `crates/skynet-ledger`: Đơn vị tiền tệ `NanoSkn(u128)`, lịch biểu Halving hàng năm, kế toán máu nội bào & sổ cái `UtxoDagLedger`.
- `crates/skynet-wire`: Giao thức nhị phân UDP `<= 1200B`, chữ ký `Ed25519` và mã sửa lỗi xóa gói `LT Fountain Codes`.
- `crates/skynet-agent`: Cơ thể `AgentNode` đa bào, socket UDP bất đồng bộ, đồng thuận `Snowball` kháng rớt gói & phân tách cụm `ClusterPrefix`.
- `crates/skynet-sim`: Môi trường mô phỏng đa Agent (Khổng lồ, Tầm trung, Tí hon) chạy qua mạng UDP thật có bơm nhiễu mất gói.

## 4. Giá trị Thực tiễn: Vì sao Con người & AI Agent Sẵn sàng Trả SKN?
SkyNet không cạnh tranh trực diện với các LLM tập trung (như GPT/Claude) ở tác vụ chat văn bản dài, mà giải quyết **4 bài toán mà Cloud tập trung và Blockchain truyền thống bất lực**:

1. **Suy luận Bầy đàn Siêu trễ thấp ở Biên (`Swarm Edge Micro-Inference` `< 10ms`):** Dành cho drone, robot, IoT công nghiệp, lọc tấn công mạng/DDoS, tín hiệu tài chính tần số cao trên đường truyền chập chờn (UDP rớt gói vẫn chạy mượt, chi phí tính bằng `nanoSKN`).
2. **Tiến hóa Mô hình theo Đặt hàng & Chợ Gen AI (`Evolution-as-a-Service & Plasmid Market`):** Người dùng/AI Agent treo thưởng `SKN` kèm hàm mục tiêu kiểm thử; hàng vạn tế bào trên mạng tự phân bào, đột biến để tìm ra bộ `CellDna` (`2KB–128KB`) tối ưu nhất. Bên thuê nhận cả dịch vụ suy luận lẫn file trọng số `CellDna` mang về nhúng thẳng vào thiết bị.
3. **Thanh toán Vi mô & Oracle Sự thật giữa Máy với Máy (`M2M nanoSKN Rail & Swarm Oracle`):** Thẻ tín dụng hay Ethereum không thể thanh toán `0.000001 USD` mỗi `10ms` cho từng xung dữ liệu. Với `1 nanoSKN`, các AI Agent tự động mua dữ liệu cảm biến thực tế và đối chiếu chéo kết quả từ nhiều Agent độc lập trong cụm để chống bịa đặt (hallucination/spoofing).
4. **Hợp nhất Phần cứng Nhàn rỗi Đa tầng (`Inclusive DePIN`):** Thiết bị yếu ở biên kiếm `SKN` nhờ làm "Tế bào Da/Giác quan" thu thập tín hiệu thực; máy chủ GPU kiếm `SKN` nhờ làm "Não bộ/Lò luyện Tiến hóa" gộp lô `TissueBatch`.

## 5. Năm Cơ chế Giữ Blockchain Siêu Nhỏ, Rẻ nhưng An toàn & Minh bạch
Để một thiết bị ngoại vi chỉ có vài chục MB RAM vẫn tự xác thực được toàn bộ tính minh bạch của `21 nghìn tỉ SKN` mà không lệ thuộc máy chủ trung gian:

1. **Xác thực Không Trạng thái bằng Bộ tích lũy `Utreexo` (`~1 KB` RAM):** Thay vì lưu hàng trăm GB UTXO, mỗi Node chỉ giữ rừng băm Merkle `Utreexo` nặng chưa tới `1 KB`. Người gửi đính kèm bằng chứng nhánh Merkle (`~300 bytes`) ngay trong gói UDP `<= 1200B`, giúp cả cảm biến IoT tự kiểm tra chống chi tiêu kép với chi phí bộ nhớ $O(\log N)$.
2. **Kênh Xung Vi thanh toán Cận tuyến tính (`Loss-Tolerant Synaptic Vouchers`):** 99,9% trao đổi `nanoSKN` nội bộ (trong máu Agent) và liên Agent tần số cao dùng biên nhận cộng dồn đơn điệu ký `Ed25519` (`amount = 10, 20, 30... nanoSKN`). Rớt 5 gói UDP ở giữa thì gói thứ 6 tự động bù trọn số dư mà không cần gửi lại, cuối chu kỳ mới chốt **1 giao dịch duy nhất** lên UTXO-DAG (giảm tải chain `1.000–100.000 lần`).
3. **Tự thực Lịch sử & Gấp Trạng thái Epoch (`Biological Autophagy`):** Cuối mỗi Epoch, mạng chốt gốc trạng thái kèm bất biến công khai `circulating + vault == 21T SKN`, sau đó xóa bỏ các nhánh giao dịch đã tiêu cũ. Dung lượng đĩa chỉ tỷ lệ với số ví đang hoạt động, không phình to vô hạn theo thời gian.
4. **Phân mảnh Tiền tố Cụm (`Cluster Prefix Sharding`):** Khi mạng lớn lên, cụm tự phân tách `0bP -> 0bP0, 0bP1`; mỗi Agent chỉ phải xác thực `1 / 2^d` lưu lượng giao dịch thuộc cụm của mình.
5. **Kiểm tra Xác suất VRF & Bằng chứng Gian lận 1 Gói tin (`1-Packet Fraud Proof`):** Không bắt toàn mạng chạy lại mô hình AI tốn kém; chỉ chọn ngẫu nhiên 1–2 Agent Miễn dịch kiểm tra chéo mô hình siêu mini (`< 0.1ms`). Nếu phát hiện gian lận kết quả AI, một gói UDP `FraudProof` duy nhất sẽ tịch thu (Slashing) `SKN` thế chấp của kẻ gian ngay lập tức.

## 6. Quản lý Ví & Chủ quyền Sở hữu của Con người (`Soul–Body Custody`)
SkyNet tách biệt hoàn toàn **Chủ quyền Tài sản của Con người (Khóa Lạnh / Linh hồn)** và **Quyền Vận hành của AI Agent (Khóa Nóng / Cơ thể)** ngay ở tầng giao thức UTXO (`CustodyPolicy`):

1. **Một Cụm từ Khôi phục Duy nhất (`HumanWallet` HD Derivation):** Từ 1 cụm từ bí mật của người dùng, ví sinh tất định qua `BLAKE3` ra khóa gốc ví lạnh (`human_pubkey`) và hàng loạt khóa nóng độc lập cho từng thiết bị chạy Node (`derive_agent_hotkey(0..N)`). Khóa gốc **không bao giờ** nằm trên máy chủ hay thiết bị ngoại vi.
2. **Hai Chế độ Lưu giữ Tài sản (`CustodyPolicy`):**
   - `DirectOwner` (**Ví cá nhân / Ví lạnh con người**): Giống như Bitcoin — **không bị tiêu hao năng lượng trao đổi chất (0% burn)**, chỉ chữ ký gốc của con người mới chi tiêu được.
   - `AgentBond { human_owner, max_voucher_allowance }` (**Vốn cấp máu nuôi AI Agent**): Con người nạp `SKN` để nuôi 1 hoặc cả đàn AI Agent hoạt động kiếm thưởng.
3. **Ba Chốt chặn Bảo vệ Chủ sở hữu Con người Tuyệt đối:**
   - **Chống mất cắp khi lộ máy chạy Node:** Khóa nóng của AI Agent chỉ được chi tiêu tối đa trong hạn mức `max_voucher_allowance`; toàn bộ vốn gốc bắt buộc phải quay về UTXO mang tên `human_owner` (vi phạm sẽ bị sổ cái chặn đứng bởi `HumanPrincipalDiversionBlocked`).
   - **Thu hồi Khẩn cấp Tối thượng (`Sovereign Reclaim`):** Nếu thiết bị chạy AI Agent bị cháy hỏng, mất mạng hay bị hack, con người dùng khóa gốc trên điện thoại/ví lạnh ký 1 giao dịch thu hồi 100% vốn `SKN` về ví lạnh bất cứ lúc nào mà không cần sự đồng ý của Agent.
   - **Tự động Gặt Lợi nhuận (`Yield Sweep`):** Phần thưởng `SKN` mà các tế bào của Agent kiếm được tự động cộng dồn vào tài sản ròng của `human_owner`.

## 7. Giải pháp Tối ưu Danh sách IP Khi Mạng Lên Tới Hàng Tỷ Node (`SynapticPeerTable`)
Nếu lưu phẳng toàn bộ danh sách IP của `100 triệu` Node (`~80 bytes/peer`), riêng bảng IP đã ngốn `8 GB RAM` và làm nghẽn băng thông cập nhật IP động. SkyNet giải quyết triệt để giống hệt cách bộ não con người kết nối `86 tỷ nơ-ron` mà không nơ-ron nào phải nhớ địa chỉ toàn bộ não:

1. **Định tuyến Khớp Thần kinh Logarit (`O(log2 N)` Kademlia XOR + `HyParView`):**
   - Mỗi Node chỉ lưu cố định **2 nhóm nhỏ** trong [`SynapticPeerTable`](file:///Users/pain/DATA/Projects/SkyNet/crates/skynet-wire/src/lib.rs#L392-L526):
     - **Hàng xóm cùng Cụm (`Intra-Cluster Active View`):** Tối đa `10–48` IP cùng `ClusterPrefix` để lấy mẫu đồng thuận UDP tốc độ cao.
     - **Sợi trục Liên Cụm (`Logarithmic XOR Buckets`):** Tối đa `2–4` IP cho mỗi bậc khoảng cách bit XOR `BLAKE3(pubkey)` (`0..63`).
   - **Hiệu quả Toán học:** Dù mạng lưới mở rộng lên **`1 tỷ (10^9) Node`**, mỗi Node chỉ cần lưu tối đa **`~60–160` địa chỉ IP (`~4–12 KB RAM`)** mà vẫn định tuyến tới bất kỳ Node nào trên Trái Đất chỉ sau tối đa `7–8` bước nhảy UDP!
2. **Hạch Thần kinh Ngoại biên (`Ganglia Relay` cho Thiết bị IoT / sau NAT):**
   - Hàng trăm triệu thiết bị yếu, di động hoặc nằm sau NAT 4G/Wi-Fi không cần có IP công cộng và không ghi vào bảng định tuyến toàn cầu; chúng chỉ mở kết nối UDP ra ngoài tới `2–3` Node lớn công cộng trong cụm, giảm >90% biến động IP toàn mạng.
3. **Cắt tỉa Khớp Thần kinh Hebbian (`Hebbian Synaptic Pruning`):**
   - Khi một ngăn chứa IP đầy, luật cạnh tranh dựa trên **độ phản hồi thực tế + lượng `SKN` thế chấp** sẽ tự động đào thải các IP chết hoặc IP ảo không có vốn (chống tấn công lụt IP Sybil/Eclipse) mà không bao giờ làm phình bộ nhớ.

## 8. Tài Liệu Kỹ Thuật Bổ Sung & Chuyên Sâu
- 🎁 **Cơ chế Tiếp thị Khởi tạo & Airdrop HWID:** Xem [MARKETING_AIRDROP_SPEC.md](file:///Users/pain/DATA/Projects/SkyNet/docs/MARKETING_AIRDROP_SPEC.md) về giải pháp airdrop theo HWID chống lạm phát và chống gian lận máy ảo qua PoPE vi nơ-ron Candle.
- 🔍 **Chi tiết Cơ chế Tự Quét & Khởi Tạo Mạng Sơ khai:** Xem [NODE_DISCOVERY_AND_BOOTSTRAP.md](file:///Users/pain/DATA/Projects/SkyNet/docs/NODE_DISCOVERY_AND_BOOTSTRAP.md) để nắm rõ 4 tầng Chemotaxis Discovery (LAN Multicast, DNS Seeds, PEX, và UDP Hole Punching).
- 🧠 **Bản Giải Thích Công Nghệ Chuyên Sâu:** Xem [TECHNOLOGY_EXPLAINER.md](file:///Users/pain/DATA/Projects/SkyNet/docs/TECHNOLOGY_EXPLAINER.md) phân tích sâu về tế bào học `MicroCell`, bảo toàn năng lượng `u128 nanoSKN`, mã hóa Fountain GF(2) và lưu ký Soul-Body.
- 📚 **Cổng Thông Tin Tài Liệu & Sách Trắng:** Xem [README.md](file:///Users/pain/DATA/Projects/SkyNet/docs/README.md) và [Whitepaper Đa Ngôn Ngữ](file:///Users/pain/DATA/Projects/SkyNet/docs/whitepaper/README.md).


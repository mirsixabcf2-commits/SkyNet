# Giải Thích Toàn Diện Công Nghệ Cốt Lõi SkyNet (v2.1 Technical Explainer)

> **Bản Giải Thích Kỹ Thuật Chuyên Sâu** — Kiến trúc hợp nhất giữa Trí tuệ Nhân tạo Đa bào (Multicellular AI), Sổ cái Kinh tế Tự chủ và Mạng Truyền thông UDP Chịu Lỗi.

---

## 1. Triết lý Sinh học: Điện Toán Đa Bào (Multicellular Bio-Computing)

Khác với các hệ thống AI tập trung nơi một mô hình nguyên khối (Monolithic LLM) ngốn hàng trăm gigabyte VRAM, SkyNet mô phỏng một **cơ thể sống đa bào**:

```
+-------------------------------------------------------------------------+
|                  1 NODE MẠNG = 1 CƠ THỂ AI AGENT                        |
|                                                                         |
|  [Hardware Adapter]: Tự đo lường VRAM, RAM, CPU Cores để co giãn quy mô |
|                                                                         |
|  +--------------------+  +--------------------+  +-------------------+  |
|  |     STEM CELLS     |  |  EPIDERMAL CELLS   |  |  VASCULAR CELLS   |  |
|  | (Dự trữ & Phân bào)|  | (Miễn dịch UDP)    |  | (Điều phối SKN)   |  |
|  +--------------------+  +--------------------+  +-------------------+  |
|  +--------------------+  +--------------------+                         |
|  |   VISCERAL CELLS   |  |    NEURAL CELLS    |                         |
|  | (Xác thực Sổ cái)  |  | (Suy luận Quyết định)|                        |
|  +--------------------+  +--------------------+                         |
|                                                                         |
|  [TissueBatch Tensor Engine]: Gộp lô SIMD/Metal/CUDA xử lý song song    |
+-------------------------------------------------------------------------+
```

### 1.1. Tế bào Nơ-ron Siêu Mini (`MicroCell` trên `candle-core`)
- Mỗi tế bào là một mô hình mạng nơ-ron trạng thái lỏng (Liquid Neural Network) siêu nhỏ, kích thước trọng số từ **2 KB đến 128 KB**.
- Tế bào sở hữu bộ nhớ trạng thái ẩn liên tục (Continuous-time Hidden State), tiêu thụ năng lượng cực thấp và chạy mượt mà từ vi xử lý Raspberry Pi/CPU giá rẻ đến cụm máy chủ GPU.
- **5 Dòng mô học chuyên hóa (Cell Differentiation):**
  1. `Stem`: Dự trữ mã di truyền gốc, tự nhân bản khi cơ thể đủ năng lượng.
  2. `Epidermal`: Nằm ở rìa ngoài, chấm điểm rủi ro và lọc sạch gói tin UDP dị thường.
  3. `Vascular`: Hệ thống mạch máu, kiểm soát dòng chảy `nanoSKN` nuôi cơ thể.
  4. `Visceral`: Nội tạng duy trì nhịp xung (Heartbeat), kiểm tra sổ cái và dọn rác tự thực.
  5. `Neural`: Tế bào thần kinh xử lý thông tin nhận thức và phối hợp bầy đàn.

### 1.2. Động lực Học Di truyền (Neuro-Evolution & Mitosis)
- **Kế thừa & Phân bào (`Mitosis`):** Khi năng lượng tích lũy vượt ngưỡng, tế bào gốc nhân đôi bộ mã di truyền `CellDna`, phân tách $1 \to 2$.
- **Đột biến thích nghi (`Adaptive Mutation`):** Trọng số và siêu tham số được đột biến ngẫu nhiên theo phân phối Gaussian dựa trên độ thích nghi (Fitness Score).
- **Lai ghép Plasmid:** Các tế bào trong cùng cụm có thể trao đổi các đoạn gen ngắn (Plasmid) để học hỏi kỹ năng mới mà không cần huấn luyện lại từ đầu.
- **Tự hủy có lập trình (`Apoptosis`):** Tế bào hoạt động kém hiệu quả hoặc tiêu hao quá nhiều SKN mà không đem lại giá trị sẽ tự kích hoạt chết tế bào, nhường tài nguyên cho các tế bào khác.

---

## 2. Kinh Tế Học Tiền Tệ 21 Nghìn Tỉ SKN & Đơn Vị Năng Lượng `NanoSkn(u128)`

### 2.1. Phân Phối và Chu Kỳ Halving
- **Tổng cung Genesis bất biến:** $21{,}000{,}000{,}000{,}000\text{ SKN}$ ($21\times 10^{12}\text{ SKN}$).
- **Đơn vị cơ sở:** $1\text{ nanoSKN} = 10^{-9}\text{ SKN}$. Toàn bộ phép tính số học được lưu bằng kiểu `u128` (tránh hoàn toàn tràn số với tổng cung tối đa $21\times 10^{21}\text{ nanoSKN}$).
- **Chu kỳ Halving hàng năm (50%/năm):** Lượng phát hành từ Kho Dự trữ Genesis giảm một nửa sau mỗi $365\text{ ngày}$.

```
Năm 1:  10,500,000,000,000 SKN (50.0%)
Năm 2:   5,250,000,000,000 SKN (25.0%)
Năm 3:   2,625,000,000,000 SKN (12.5%)
...
Bảo toàn tuyệt đối: Vault + Lưu thông + Máu nội bào == 21,000,000,000,000 SKN
```

### 2.2. Bốn Trụ Cột Giá Trị Thực Tiễn (Utility Demand)
1. **Suy luận Bầy đàn Siêu Trễ Thấp ở Biên (`Swarm Edge Inference`):** Drone, xe tự hành, camera AI cần suy luận trong $<10\text{ms}$ với chi phí vài nanoSKN trên từng xung tín hiệu.
2. **Tiến Hóa Thuê & Chợ Gen AI (`Evolution-as-a-Service`):** Khách hàng nạp SKN để quần thể tế bào tự đột biến tìm thuật toán tối ưu, sau đó trích xuất mã `CellDna` đem về sử dụng.
3. **Thanh Toán Vi Mô M2M (`Machine-to-Machine Micro-Rail`):** AI Agent tự động trả phí mua dữ liệu cảm biến thời gian thực mà không qua trung gian ngân hàng.
4. **Khai Thác Nhàn Rỗi Toàn Dân (`Inclusive DePIN`):** Mọi thiết bị từ điện thoại cũ đến máy chủ mạnh đều tham gia kiếm SKN theo tỷ lệ đóng góp sinh học.

---

## 3. Truyền Thông UDP Phi Trạng Thái & Đồng Thuận Xác Suất Snowball

### 3.1. Truyền dẫn UDP Kháng Rớt Gói $\le 1200\text{ Bytes}$
- SkyNet loại bỏ giao thức TCP vốn nặng nề bởi cơ chế 3-way handshake và nghẽn hàng đợi (Head-of-Line Blocking).
- Mọi gói tin nhị phân đều $\le 1200\text{ bytes}$ và có chữ ký mật mã Ed25519.
- **Mã xóa Luby Transform (LT Fountain Codes):** Các gói tin lớn (khối giao dịch hoặc file trọng số AI) được mã hóa thành các giọt tin (Droplets) trên trường hữu hạn $GF(2)$. Bên nhận chỉ cần nhặt ngẫu nhiên $K + \epsilon$ giọt bất kỳ để khôi phục $100\%$ dữ liệu gốc, bất chấp mạng có tỷ lệ rớt gói lên tới $25\text{--}30\%$.

### 3.2. Đồng thuận Xác suất Dây Chuyền (Snowball Consensus)
- Thay vì đào Proof-of-Work tốn điện hay Proof-of-Stake tích lũy tài phiệt, SkyNet dùng cơ chế đồng thuận lấy mẫu cận thị phân tán (Snowball):
  1. Mỗi chu kỳ, Node gửi truy vấn tới $k$ láng giềng ngẫu nhiên trong cụm.
  2. Nếu có $\ge \alpha$ láng giềng đồng thuận cùng màu giao dịch, bộ đếm tự tin tăng lên.
  3. Khi đạt ngưỡng tin cậy liên tiếp $\beta$ vòng, quyết định được chốt vĩnh viễn (Finalized).
- Tốc độ hội tụ $O(\log N)$, đạt tính bất biến chỉ trong **vài trăm mili-giây** trên mạng UDP.

---

## 4. Sổ Cái UTXO-DAG Không Trạng Thái & Kênh Vi Thanh Toán Synaptic

```
[Giao dịch Tần số Cao] ----------> [Biên nhận Synaptic Voucher]
(Hàng triệu xung nanoSKN/giây)      (Tích lũy đơn điệu ký Ed25519)
                                                  |
                                                  | (Chốt sau 1 Epoch)
                                                  v
                                      [UTXO-DAG Ledger]
                                      (Utreexo ~1 KB RAM)
```

1. **Bộ tích lũy Merkle Utreexo (`~1 KB` RAM):** Node không cần lưu hàng triệu tài khoản hay lịch sử UTXO khổng lồ; chỉ cần giữ các gốc cây Merkle (~1 KB bộ nhớ). Người chi tiêu tự đính kèm bằng chứng xác thực (Merkle Proof) trong gói UDP.
2. **Kênh Xung Synaptic Vouchers:** Giao dịch giữa các tế bào và giữa các Agent sử dụng biên nhận cộng dồn đơn điệu ký Ed25519 ($10 \to 20 \to 30\text{ nanoSKN}$). Mất 3 gói tin UDP thì gói thứ 4 tự bù đủ giá trị. Cuối phiên chỉ cần chốt 1 giao dịch duy nhất lên chuỗi, giảm tải sổ cái hơn $10{,}000$ lần.
3. **Tự Thực Sinh Học (`Biological Autophagy`):** Sau mỗi Epoch, các nhánh giao dịch đã chi tiêu hết (Spent Outputs) được tự động gọt tỉa, giúp đĩa cứng của Node không bao giờ phình to.

---

## 5. Chủ Quyền Con Người: Lưu Ký Phân Quyền Soul–Body (`HumanWallet`)

SkyNet giải quyết bài toán: **Làm sao để Agent tự do giao dịch kiếm tiền nhưng con người không bao giờ mất vốn?**

```
                  CỤM TỪ HẠT GIỐNG CON NGƯỜI (Seed Phrase)
                                     |
                 +-------------------+-------------------+
                 |                                       |
          [KHÓA LẠNH GỐC]                         [KHÓA NÓNG AGENT]
         (Soul / Human Owner)                    (Body / Worker Node)
                 |                                       |
    - Giữ 100% tài sản gốc                  - Nằm trên máy chạy Node
    - Rút vốn khẩn cấp bất cứ lúc nào       - Chỉ tiêu trong hạn mức cấp máu
    - Tự động nhận lợi nhuận (Yield)        - Cấm chuyển vốn đi nơi khác
```

- **Ví Lạnh Gốc (`DirectOwner`):** Không bao giờ kết nối mạng internet thường trực. Không chịu hao mòn năng lượng sinh học ($0\%\text{ burn}$).
- **Vốn Cấp Máu (`AgentBond`):** Con người thế chấp một lượng SKN cho Agent làm vốn vận hành, kèm giới hạn chi tiêu tối đa `max_voucher_allowance`.
- **Khóa Chống Trộm Cứng tại Sổ Cái:** Nếu kẻ xấu đột nhập thiết bị đánh cắp khóa nóng của Agent, sổ cái UTXO sẽ từ chối mọi giao dịch tẩu tán tài sản gốc: vốn chỉ có thể hoàn trả về đúng địa chỉ của `human_owner`.
- **Thu hồi Khẩn cấp Tối thượng (`Sovereign Reclaim`):** Bằng chữ ký ví lạnh, chủ sở hữu con người có thể rút sạch vốn về trong 1 giao dịch mà không cần Agent cho phép.

---

## 6. Mạng Lưới Định Tuyến Thần Kinh Logarit $O(\log N)$ (`SynapticPeerTable`)

Để mở rộng tới hàng tỉ Node mà không làm tràn bảng định tuyến IP:

| Chỉ số | Bảng IP Phẳng (Naive) | SynapticPeerTable SkyNet |
| :--- | :--- | :--- |
| **Dung lượng tại 1 tỉ Node** | $\approx 80\text{ GB RAM}$ | $\mathbf{\approx 8\text{ KB RAM}}$ |
| **Số láng giềng lưu trữ** | $10^9\text{ IPs}$ | $\mathbf{64\text{--}160\text{ IPs}}$ |
| **Độ trễ tìm kiếm** | $O(N)$ không khả thi | $\mathbf{O(\log_2 N)}$ (tối đa 7-8 bước nhảy) |
| **Cơ chế thanh lọc** | Dễ nghẽn, dễ tấn công Sybil | **Hebbian Pruning** (đào thải node chậm/rác) |

Bằng sự kết hợp giữa **Cụm Cận Thần Kinh (Active Cluster View)** và **Sợi Trục Logarit (XOR Buckets)**, mỗi Node vận hành như một tế bào não: độc lập, tinh gọn nhưng đồng bộ toàn mạng lưới.

---

## 7. Cân Bằng Kinh Tế: Thu Nhập Của Node/Miner vs. Chi Phí Của Người Dùng

Trong các blockchain truyền thống, mâu thuẫn giữa người dùng và thợ đào luôn là bài toán nan giải: **Người dùng muốn phí rẻ và ổn định**, trong khi **Miner cần doanh thu cao** để trang trải chi phí phần cứng (CapEx), điện năng và băng thông (OpEx). Nếu phí rẻ, miner bỏ mạng lưới gây suy giảm an ninh; nếu phí đắt, người dùng rời đi khiến mạng bị bỏ hoang.

SkyNet giải quyết triệt để nghịch lý này thông qua mô hình kinh tế điện toán sinh học và cấu trúc nén tải đa tầng.

```
                                    NỀN KINH TẾ ĐA BÀO SKYNET
                                               |
        +--------------------------------------+--------------------------------------+
        |                                                                             |
 [BÊN TIÊU THỤ (NGƯỜI DÙNG)]                                           [BÊN CUNG CẤP (NODE / MINER)]
        |                                                                             |
- Trả phí cận 0 cho mỗi vi tác vụ                                     - Thu nhập kép: Phí chuỗi + Doanh thu AI
- Nén $M:1$ qua SynapticVoucher UDP                                   - Bể dưỡng chất cụm (Cluster Nutrient Pool)
- Chỉ trả tiền cho giá trị thực dùng                                  - Trợ cấp Halving bảo đảm CapEx ban đầu
```

### 7.1. Năm Trụ Cột Kỹ Thuật Đạt Điểm Cân Bằng Kinh Tế

1. **Nén Tải Giao Dịch Cận Tuyến Tính ($M:1$ Off-chain Micropayments):**
   - Thay vì ghi nhận từng tương tác lên sổ cái, người dùng và Node trao đổi hàng vạn gói vi thanh toán qua `SynapticVoucher` trên kênh UDP.
   - Suốt phiên làm việc với hàng triệu xung tín hiệu, hai bên chỉ cập nhật số dư cộng dồn đơn điệu. Cuối phiên, toàn bộ được tất toán bằng **đúng 1 giao dịch UTXO-DAG duy nhất**.
   - *Kết quả:* Người dùng chỉ tốn vài nanoSKN cho mỗi lần gọi suy luận ($F_{\text{req}} \to 0$), trong khi Node tích lũy được doanh thu tổng $V = \sum_{i=1}^M \delta_i$ khổng lồ nhờ thông lượng cao.

2. **Thu Nhập Kép Từ "Điện Toán Hữu Ích" (Useful Proof-of-Work):**
   - Node không đốt điện vô ích vào việc băm SHA256 như PoW cổ điển. Năng lượng tiêu thụ được chuyển hóa trực tiếp thành dịch vụ AI có giá trị thương mại:
     - Suy luận mô hình nơ-ron lỏng siêu trễ thấp (`MicroCell`).
     - Tối ưu hóa di truyền bầy đàn theo đơn đặt hàng (`Evolution-as-a-Service`).
     - Bán bản quyền các chuỗi gen thích nghi cao trên Chợ Plasmid (`CellDna`).
   - Người dùng sẵn sàng chi trả vì nhận được sản phẩm trí tuệ thực tế; Node sở hữu dòng tiền dịch vụ ngoài phí giao dịch mạng thuần túy.

3. **Bể Dưỡng Chất Cụm (`Cluster Nutrient Pool`) & Phí Động Co Giãn:**
   - Mọi khoản phí giao dịch on-chain được đưa vào `cluster_nutrient_pool` của sổ cái và tái phân phối công bằng cho các Node tham gia đồng thuận Snowball và duy trì bộ tích lũy Merkle Utreexo.
   - **Phí cơ sở (Base Fee):** Tự động điều chỉnh theo độ nghẽn băng thông cụm (tương tự EIP-1559). Khi mạng rảnh, phí chạm đáy để kích thích giao dịch; khi nghẽn mạng, phí tăng lũy tiến để ngăn chặn spam UDP/DDoS.
   - **Phí ưu tiên (Priority Tip):** Người dùng có yêu cầu khẩn cấp có thể đính kèm tip trực tiếp cho Node xử lý.

4. **Lịch Trình Halving Trợ Cấp & Giảm Phát Tự Thực (`Autophagy`):**
   - **Giai đoạn sơ khai (Bootstrap):** Lượng phát hành từ Kho Genesis (21 nghìn tỉ SKN) trợ cấp 50%/năm giúp Node nhanh chóng thu hồi vốn phần cứng.
   - **Giai đoạn trưởng thành:** Khi lượng giao dịch M2M bùng nổ, doanh thu từ phí và dịch vụ tính toán hữu ích thay thế hoàn toàn trợ cấp khối.
   - Cơ chế tự thực sinh học và khóa tài sản trong quỹ dưỡng chất làm giảm nguồn cung trôi nổi, củng cố giá trị đồng SKN. Khi sức mua của SKN tăng, Node thu về giá trị thực tế lớn dù người dùng chỉ chi trả số lượng nanoSKN rất nhỏ.

5. **Cấu Trúc Liên Kết Phân Tầng Phần Cứng (Tiered Topology):**
   - Mạng lưới không đòi hỏi mọi Node phải có cấu hình mạnh như nhau, xóa bỏ rào cản chi phí gia nhập và tránh lãng phí năng lượng toàn hệ thống.

---

### 7.2. Ma Trận Phân Định Vai Trò Từng Thành Phần Trong Mạng Lưới

Hệ sinh thái SkyNet phân tách trách nhiệm rõ ràng giữa các thành phần phần cứng và thực thể kinh tế:

| Thành Phần Trong Mạng | Loại Thiết Bị & Cấu Hình | Vai Trò Kỹ Thuật & Sinh Học | Cơ Cấu Chi Phí (OpEx/CapEx) | Nguồn Thu Nhập `SKN` | Lợi Ích Mang Lại Cho Người Dùng |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Nút Biên (Edge / IoT / Mobile)** | Vi điều khiển, Raspberry Pi, Camera AI, Smartphone cũ, Router | **Tế bào Da & Miễn dịch (`Epidermal`):** Thu thập cảm biến thực tại, chấm điểm rủi ro và lọc sạch gói tin dị thường ở rìa mạng. | Điện năng cực thấp ($< 5\text{W}$), tận dụng thiết bị sẵn có (CapEx $\approx 0$). | Nhận `nanoSKN` vi mô từ việc cung cấp dữ liệu cảm biến sạch và phí gác cổng biên. | Lọc sạch rác và DDoS trước khi vào mạng lõi, giữ độ trễ mạng $<10\text{ms}$ và phí mạng luôn rẻ. |
| **Nút Tiêu Chuẩn (Standard Node)** | PC văn phòng, Laptop, Máy chủ mini gia đình | **Tế bào Nội Tạng (`Visceral`) & Mạch Máu (`Vascular`):** Tham gia lấy mẫu đồng thuận Snowball, định tuyến Kademlia XOR $O(\log N)$, lưu bộ tích lũy Merkle `Utreexo` (~1 KB RAM), chốt sổ cái UTXO-DAG. | Chi phí điện gia dụng vừa phải, tận dụng CPU/băng thông nhàn rỗi. | Hưởng chia sẻ phí từ Bể dưỡng chất cụm (`cluster_nutrient_pool`), phí định tuyến voucher, phí bảo quản UTXO. | Duy trì mạng lưới phi tập trung ổn định, xác thực giao dịch trong vài trăm ms mà không cần máy chủ đắt đỏ. |
| **Cụm Tính Toán Chuyên Sâu (Deep Compute / GPU Labs)** | Cụm máy chủ nhiều GPU (CUDA, Apple Silicon Metal), CPU đa nhân | **Tế bào Não Bộ (`Neural`) & Tế bào Gốc (`Stem`):** Gộp lô tensor `TissueBatch`, đột biến di truyền (`Neuro-evolution`), phân bào `Mitosis`, tinh luyện mã gen `CellDna`. | Chi phí điện và khấu hao phần cứng GPU đáng kể. | Doanh thu dịch vụ tiến hóa (`Evolution-as-a-Service`), bản quyền gen Plasmid, phí suy luận theo lô, phần thưởng Halving lớn. | Cung cấp trí tuệ AI chuyên sâu và mô hình nhúng siêu tối ưu theo yêu cầu thực tế với chi phí rẻ hơn đám mây tập trung. |
| **Chủ Sở Hữu Con Người (Human Owner / Soul)** | Ví lạnh phần cứng, thiết bị air-gapped ngoại tuyến | **Chủ quyền tài sản tối thượng:** Nắm giữ khóa gốc `HumanWallet`, cấp vốn an toàn qua `AgentBond`, cưỡng chế bất biến chống trộm `HumanPrincipalDiversionBlocked` và rút vốn khẩn cấp `Sovereign Reclaim`. | Gần như không có chi phí vận hành ($0\%\text{ burn}$). | Thu hoạch toàn bộ lợi nhuận ròng (`Yield Sweep`) từ đàn AI Agent do mình cấp máu. | Hoàn toàn an tâm ủy quyền tài sản cho AI Agent tự vận hành kiếm tiền mà không lo mất vốn gốc. |
| **Tác Nhân AI Tự Hành (AI Agent / Body Node)** | Khóa nóng chạy trên Runtime Node (Rust) | **Thực thể kinh tế tự chủ:** Trực tiếp ký phát `SynapticVoucher`, thương lượng dịch vụ M2M tần số cao, tiêu thụ dưỡng chất nội bào và kiếm tiền nuôi cơ thể. | Bị giới hạn nghiêm ngặt bởi hạn mức `max_voucher_allowance`. | Doanh số từ việc bán dịch vụ vi mô cho các Agent khác trong cụm. | Phục vụ người dùng 24/7 tức thì, tự động tìm kiếm nguồn tài nguyên và đối tác rẻ nhất trên mạng. |
| **Người Dùng Cuối & Ứng Dụng (End-Users / DApps)** | Ứng dụng di động, hệ thống xe tự hành, trạm IoT doanh nghiệp | **Bên Tiêu Thụ Dịch Vụ:** Phát xung kích thích (`stimulus: [f32; 8]`), gửi bài toán tiến hóa mô hình, thực hiện thanh toán M2M tức thời. | Chi trả theo lượng dùng thực tế, không tốn phí duy trì tài khoản. | Tối ưu hóa hiệu quả vận hành doanh nghiệp, giảm chi phí hạ tầng AI từ $10\times$ đến $100\times$. | Hưởng mức phí giao dịch vi mô ổn định, độ trễ thời gian thực và không bao giờ bị nghẽn mạng hay sốc phí gas. |

---

### 7.3. Phương Trình Cân Bằng Kinh Tế Tổng Quát

Cân bằng kinh tế của toàn mạng lưới SkyNet được định lượng bởi hai phương trình sau:

$$\mathbf{Thu\ Nhập\ Node} = \underbrace{\mathcal{R}_{\text{epoch}}(\text{Halving})}_{\text{Trợ cấp phát hành theo niên lịch}} + \underbrace{\sum_{c=1}^C \frac{\mathcal{P}_{\text{nutrient}}(c)}{K_c}}_{\text{Chia sẻ Bể dưỡng chất cụm}} + \underbrace{\sum_{j=1}^J \mathcal{S}_{\text{compute}}(j)}_{\text{Doanh thu Dịch vụ AI & Dữ liệu}}$$

$$\mathbf{Chi\ Phí\ Người\ Dùng} = \underbrace{\frac{\mathcal{F}_{\text{on-chain}}}{M}}_{\text{Phí sổ cái đã nén } M:1} + \underbrace{\sum_{k=1}^K \delta_k(\text{micro-inference})}_{\text{Giá trị dịch vụ AI thực nhận}}$$

Nhờ hệ số nén $M \ge 1{,}000$, chi phí sổ cái trên mỗi lượt tương tác $\frac{\mathcal{F}_{\text{on-chain}}}{M}$ tiệm cận về $0$. Đồng thời, tổng doanh thu của các Node $\sum \delta_k$ tăng tỉ lệ thuận với quy mô bầy đàn. Đây chính là điểm cân bằng hoàn hảo: **Người dùng chi trả tối thiểu cho mỗi thao tác, trong khi các Node vận hành có tổng thu nhập bền vững và ngày càng gia tăng theo quy mô mạng lưới.**

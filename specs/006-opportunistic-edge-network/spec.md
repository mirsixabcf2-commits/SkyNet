# Feature Specification: Mạng Lưới AI Cận Biên Tận Dụng GPU/CPU Nhàn Rỗi, Huấn Luyện Vi Mô Phân Tán & Kinh Tế P2P Thời Gian Thực

**Feature Directory**: `specs/006-opportunistic-edge-network`  
**Created**: 2026-11-12  
**Status**: Implemented  
**Priority**: P1 (Mainnet Production Readiness & Opportunistic Edge Expansion)

---

## 1. Tổng Quan & Định Vị Chiến Lược ("Nhanh Hơn & Rộng Hơn")

Lấy cảm hứng từ hướng tiếp cận *Opportunistic Micro-training on Edge Devices Using Spare GPU Utilization* (tận dụng năng lực GPU dư thừa trên máy tính cá nhân và máy chơi game khi nhàn rỗi để chạy các lát huấn luyện vi mô có kiểm soát nhiệt/điện năng), đặc tả này nâng cấp SkyNet thành một **mạng lưới AI cận biên phi tập trung hoàn chỉnh cấp độ Production**, vượt trội ở cả hai chiều **Nhanh hơn (Faster)** và **Rộng hơn (Broader)** đồng thời khép kín toàn bộ khoảng trống kết nối đa thiết bị thực tế từ giai đoạn `005`:

1. **Nhanh hơn ở 4 tầng cốt lõi:**
   - **Nhường máy cho Game thủ / Người dùng trong `< 100ms` SLA (mục tiêu `< 25ms`):** Kết hợp phát hiện độ trễ lát cắt tính toán nội tại với giám sát hệ điều hành thích nghi, giải phóng tức thì GPU/CPU và lưu vi điểm dừng (`Micro-Checkpoint` `< 5ms`) không mất công sức tính toán dở dang.
   - **Truyền tải UDP phi trạng thái `<= 1200 Bytes` & Giọt nước Mã xóa:** Loại bỏ nghẽn hàng đợi TCP bằng luồng phân mảnh mã xóa không cần gói xác nhận lại, chịu được mức rớt gói mạng gia đình/4G lên tới `40%`.
   - **Xác thực Công việc Hữu ích bằng Bẫy Ngẫu nhiên `O(1)` (`Canary Spot-Check`):** Thay vì nhân bản tính toán 3 lần gây lãng phí 66% phần cứng, hệ thống trộn ngầm `10%` lát kiểm tra tất định để phát hiện và cắt cọc tức thì các nút gian lận hoặc lỗi phần cứng.
   - **Thanh toán Vi mô Từng Lát (`Chunk-Streaming Settlement`) & Tự động Quét Lãi (`Yield Sweep`):** Giải ngân `NanoSkn` ngay sau từng phân mảnh hoàn thành và tự động quét phần lãi ròng về Ví Lạnh của chủ sở hữu thay vì giam vốn bằng điểm thưởng nội bộ.

2. **Rộng hơn ở 4 chiều mở rộng:**
   - **Đa nền tảng phần cứng không cần quyền quản trị (`root`/Admin):** Tương thích đồng thời máy trạm GPU rời, máy tính Apple Silicon Unified Memory tiết kiệm điện, máy Linux/Windows và thiết bị biên nhẹ.
   - **Đa tác vụ 4-trong-1 theo 3 nấc nhàn rỗi:** Điều phối linh hoạt giữa Huấn luyện vi mô (`Micro-Training`), Tiến hóa di truyền (`Plasmid` Neuro-Evolution), Suy luận trễ thấp (`< 10ms` Micro-Inference) và Tiền xử lý dữ liệu chuẩn hóa (`1,000+` mã lệnh).
   - **Kinh tế song phương cho cả Con người và Tác tử AI tự hành (M2M):** Bất kỳ Game thủ, chủ máy cá nhân hay AI Agent tự trị nào cũng có thể mua hoặc bán năng lực tính toán trực tiếp không cần trung gian tài chính tập trung, bảo vệ bởi cơ chế lưu ký phân quyền **Soul–Body Custody**.
   - **Kết nối P2P Xuyên NAT Gia Đình Thực Chiến:** Hoàn thiện nghi thức khởi nguyên `genesis.json` (`GENESIS_HASH`), màng lọc biên `< 5µs`, tự động tìm nút nội mạng LAN (`< 1s`), đục lỗ NAT UDP qua điểm hẹn (`PunchRendezvous`) và chuyển tiếp dự phòng cho mạng di động 4G/Symmetric NAT.

---

## 2. User Scenarios & Testing *(mandatory)*

### User Story 1 - Khởi Nguyên Sổ Cái `GENESIS_HASH` & Kết Nối P2P Xuyên NAT Đa Thiết Bị (Priority: P1 — MVP)

Người vận hành mạng lưới và chủ nút khởi tạo mạng SkyNet từ cùng một tệp cấu hình khởi nguyên `genesis.json` bất biến ($21\text{ nghìn tỉ SKN}$), lọc bỏ ngay tại biên mọi gói tin giả mạo/sai mạng, và tự động thiết lập đường truyền UDP P2P trực tiếp giữa các máy tính cá nhân đặt sau bộ định tuyến Wi-Fi gia đình hoặc mạng di động.

**Why this priority**: Đây là nền tảng hạ tầng bắt buộc để các máy tính thực tế ngoài đời thực nhìn thấy nhau trên cùng một sổ cái gốc và trao đổi gói tin an toàn mà không cần mở cổng thủ công trên Router.

**Independent Test**: Khởi tạo tệp `genesis.json` bằng công cụ dòng lệnh, kiểm chứng mã băm định danh mạng `GENESIS_HASH`, chạy 1 nút điểm hẹn (Seed) cùng 2 nút Worker mô phỏng sau NAT; xác nhận 2 nút đục lỗ UDP thành công, từ chối gói tin khác mạng trong `< 5µs` và đồng bộ trạng thái sổ cái gốc.

**Acceptance Scenarios**:

1. **Given** người vận hành chạy lệnh khởi tạo Genesis mới, **When** kiểm định lại tệp `genesis.json` bằng lệnh xác thực, **Then** hệ thống xác nhận tổng phân bổ `85%` Kho Halving (`17.85T SKN`) cộng `15%` Lưu thông ban đầu (`3.15T SKN`) bằng chính xác `21,000,000,000,000 SKN` (`2.1 × 10^22 nanoSKN`) không lệch dù chỉ `1 nanoSKN` và xuất mã băm `GENESIS_HASH` 32-byte nhất quán.
2. **Given** một nút SkyNet đang lắng nghe trên cổng UDP với `GENESIS_HASH` xác định, **When** nhận được gói tin có `network_id` khác `GENESIS_HASH` hoặc dấu thời gian lệch quá `±30 giây`, **Then** màng lọc biên loại bỏ gói tin trong `< 5µs` mà không cấp phát bộ nhớ động và không ảnh hưởng tới các luồng xử lý hợp lệ.
3. **Given** hai nút A và B nằm sau hai lớp NAT khác nhau cùng cấu hình trỏ về nút điểm hẹn Seed, **When** cả hai gửi bản tin báo danh `PheromoneHello` và nhận lệnh ghép cặp `PunchRendezvous`, **Then** hai nút bắn đồng thời loạt xung UDP thăm dò để thiết lập kết nối P2P trực tiếp trong `< 3 giây` (hoặc tự động chuyển sang kênh chuyển tiếp `GangliaRelay` nếu NAT đối xứng chặn đục lỗ trực tiếp).

---

### User Story 2 - Bộ Điều Tốc Cơ Hội Không Độ Trễ Cho Game Thủ & Chủ Máy (Priority: P1 — MVP)

Game thủ hoặc người làm việc sáng tạo bật chế độ kiếm thu nhập thụ động trên máy cá nhân; hệ thống tự động nhận diện lúc máy rảnh để tính toán AI và lập tức trả lại 100% tài nguyên GPU/CPU trong tích tắc ngay khi người dùng mở Game, rút sạc pin hoặc khi máy nóng lên, đảm bảo không sụt giảm tốc độ khung hình (0 FPS drop) và không gây hú quạt khó chịu.

**Why this priority**: Trải nghiệm "không gây phiền nhiễu tuyệt đối" và "không lỗ tiền điện" là yếu tố quyết định để hàng triệu chủ sở hữu GPU cá nhân tự nguyện tham gia mạng lưới.

**Independent Test**: Chạy tiến trình Worker ở chế độ `--opportunistic` đang xử lý dở một lát huấn luyện, kích hoạt tín hiệu mở tiến trình Game ưu tiên / rút nguồn sạc AC / vượt ngưỡng nhiệt `78°C`; đo thời gian dừng thực tế `< 100ms`, kiểm chứng tệp `Micro-Checkpoint` được ghi nguyên tử trên đĩa và tự động nối lại đúng bước dở dang sau khi máy rảnh và nguội lại.

**Acceptance Scenarios**:

1. **Given** Worker đang thực thi các lát cắt tính toán ở trạng thái nhàn rỗi sâu (`DeepIdle`), **When** xuất hiện tiến trình nằm trong danh sách Game/ứng dụng nặng hoặc độ trễ lát cắt phần cứng tăng vọt vượt ngưỡng tranh chấp (`> 1.8x` mức trung bình trượt), **Then** bộ điều tốc ngắt vòng lặp ngay sau lát cắt hiện tại trong `< 100ms` (mục tiêu `< 25ms`), giải phóng bộ nhớ tính toán và lưu nguyên tử `Micro-Checkpoint` trong `< 5ms`.
2. **Given** Worker đang tạm dừng do quá nhiệt (`>= 78°C`) hoặc vừa đóng Game, **When** nhiệt độ mới giảm nhẹ xuống `75°C` hoặc máy mới rảnh được `10 giây`, **Then** hệ thống giữ nguyên trạng thái tạm dừng (chống nhấp nháy bật/tắt liên tục) và chỉ tiếp tục tính toán từ bước checkpoint khi nhiệt độ hạ xuống `<= 70°C` (`temp_limit - 8°C`) và máy rảnh liên tục đủ `30 giây`.
3. **Given** người dùng chạy laptop bằng pin với cờ bảo vệ pin bật hoặc cấu hình ngưỡng giá điện tối thiểu, **When** rút nguồn sạc AC hoặc nhận được lời mời thầu có phần thưởng thấp hơn chi phí điện năng ước tính, **Then** Worker lập tức tạm dừng tính toán nặng / từ chối đặt thầu để bảo vệ tuổi thọ pin và lợi nhuận ròng của chủ máy.

---

### User Story 3 - Huấn Luyện Vi Mô Phân Tán, Tiến Hóa Gen & Kiểm Chứng PoUW Cắt Cọc (Priority: P1)

Khách hàng hoặc AI Agent phát sóng một tác vụ huấn luyện nơ-ron / tinh chỉnh trọng số / tiến hóa `Plasmid` lên mạng lưới; tác vụ được chia thành các phân mảnh truyền qua luồng giọt nước mã xóa chịu mất gói, các Worker nhàn rỗi thực hiện nhiều bước tối ưu nội bộ rồi gửi chênh lệch trọng số nén `16-bit` về tổng hợp, trong khi cơ chế bẫy ngẫu nhiên `Canary Spot-Check` tự động loại bỏ và cắt cọc các nút gian lận.

**Why this priority**: Mở rộng SkyNet từ suy luận đơn thuần sang huấn luyện AI phân tán thực thụ với chi phí xác thực thấp hơn `> 85%` so với cơ chế chạy lặp toàn phần của mô hình tập trung.

**Independent Test**: Phát một tác vụ huấn luyện phân mảnh qua kênh mô phỏng rớt gói UDP `40%` tới nhóm Worker trung thực và nhóm Worker gian lận trả về gradient ngẫu nhiên/`NaN`; kiểm chứng `100%` phân mảnh được khôi phục không cần gửi lại, toàn bộ Worker gian lận bị phát hiện qua lát Canary kèm cắt cọc bảo lãnh, và hàm mất mát của mô hình tổng hợp giảm hội tụ rõ rệt.

**Acceptance Scenarios**:

1. **Given** một tác vụ huấn luyện được chia thành các phân mảnh dữ liệu và trọng số khởi tạo, **When** truyền qua môi trường mạng rớt gói ngẫu nhiên tới `40%` trong giới hạn khung tin UDP `<= 1200 Bytes`, **Then** bộ giải mã xóa khôi phục chính xác `100%` phân mảnh (khớp mã băm toàn vẹn) mà không cần xin gửi lại gói bị mất.
2. **Given** Worker nhận phân mảnh huấn luyện hợp lệ, **When** hoàn tất chuỗi bước tối ưu nội bộ và xuất chênh lệch trọng số nén `16-bit`, **Then** hàm mất mát cuối thấp hơn hàm mất mát đầu, không chứa giá trị `NaN`/`Inf`, và mã băm chuẩn hóa lượng tử hóa khớp nhất quán giữa các kiến trúc phần cứng khác nhau.
3. **Given** luồng giao việc trộn ngầm `10%` lát bẫy `Canary Spot-Check` tất định và có Worker gửi chênh lệch trọng số sai lệch (`Cosine Similarity < 0.985`) hoặc chứa giá trị vô cực, **When** nút kiểm toán xác thực kết quả nộp về, **Then** sổ cái thực thi lệnh phạt cắt toàn bộ tiền cọc bảo lãnh của Worker gian lận (chia thưởng cho nút phát hiện và hoàn trả khách hàng), hạ điểm uy tín PoUW của Worker đó về `0`, và bộ tổng hợp `Trimmed Mean` loại bỏ các cực trị nhiễu trước khi cập nhật mô hình hoặc đóng gói `Plasmid`.

---

### User Story 4 - Đa Hợp Tác Vụ 3 Nấc, Thanh Toán Streaming Từng Lát & Tự Động Quét Lãi Về Ví Lạnh (Priority: P2)

Chủ máy tối đa hóa thu nhập trên từng phút nhàn rỗi nhờ khả năng chạy đan xen suy luận nhanh `< 10ms` với huấn luyện vi mô tùy theo mức độ rảnh của thiết bị, nhận tiền công tức thì sau mỗi lát công việc qua kênh biên lai lũy kế chịu rớt gói, và tự động nhận phần lãi ròng chuyển thẳng về Ví Lạnh cá nhân mà không lo bị tác tử tự hành tiêu lẹm vốn gốc.

**Why this priority**: Giải quyết triệt để bài toán hiệu suất khai thác phần cứng theo thời gian thực và đảm bảo an toàn tài sản tuyệt đối theo kiến trúc lưu ký **Soul–Body Custody**.

**Independent Test**: Mô phỏng chu trình 10 lát công việc liên tiếp giữa Khách hàng và Worker có gắn `AgentBond` với Ví Lạnh chủ sở hữu; xác nhận biên lai `NanoSkn` tăng đơn điệu sau từng lát, Worker tất toán đơn phương thành công khi Khách hàng ngắt kết nối ở lát thứ 6, và giao dịch `Yield Sweep` tự động chuyển phần lãi vượt ngưỡng về Ví Lạnh trong khi vẫn giữ nguyên hạn mức vốn lưu động và bảo toàn tổng cung `21T SKN`.

**Acceptance Scenarios**:

1. **Given** thiết bị chuyển đổi giữa các trạng thái `DeepIdle` và `LightIdle`, **When** ở `DeepIdle` hệ thống chạy huấn luyện vi mô xen kẽ ưu tiên các yêu cầu suy luận `< 10ms`, và **When** chuyển sang `LightIdle` (người dùng đang lướt web/soạn thảo nhẹ), **Then** hệ thống tạm dừng huấn luyện nặng để giữ máy mát êm nhưng vẫn tiếp tục xử lý các yêu cầu suy luận nhẹ `< 10ms` và kiểm toán Canary.
2. **Given** Worker hoàn thành từng lát công việc `k / N` đạt chuẩn PoUW, **When** Khách hàng phát biên lai vi thanh toán lũy kế `NanoSkn` qua UDP (kể cả khi một số biên lai trung gian bị rớt gói) và sau đó Khách hàng mất kết nối trước khi xong toàn bộ `N` lát, **Then** biên lai mới nhất đến đích bảo toàn `100%` thu nhập của mọi lát trước đó và Worker tất toán đơn phương thành công lên sổ cái.
3. **Given** số dư lãi ròng tích lũy của Worker vượt ngưỡng quét tự động (`auto_sweep_skn`, mặc định `50 SKN`), **When** tác vụ `Yield Sweep` kích hoạt, **Then** toàn bộ phần lãi thặng dư được chuyển sang đầu ra trực tiếp thuộc quyền kiểm soát duy nhất của Ví Lạnh chủ sở hữu (`0%` hao phí trao đổi chất), giữ nguyên phần vốn lưu động trong `AgentBond`, chặn đứng mọi hành vi chuyển hướng vốn gốc sang địa chỉ thứ ba (`HumanPrincipalDiversionBlocked`), và chủ sở hữu vẫn thực thi được quyền thu hồi tối cao (`Sovereign Reclaim`) bất cứ lúc nào.

---

### User Story 5 - CLI Turnkey 1 Lệnh Cho Game Thủ/Khách Hàng & Trạm Giám Sát Thu Nhập 60fps (Priority: P2)

Game thủ khởi động nút kiếm tiền chỉ bằng một dòng lệnh duy nhất không cần cấu hình phức tạp, khách hàng phát lệnh huấn luyện/suy luận thực tế từ dòng lệnh, và cả hai quan sát trực tiếp tốc độ thu nhập, nhiệt độ, trạng thái nhường máy cũng như lịch sử quét lãi trên giao diện giám sát thời gian thực `60fps`.

**Why this priority**: Đóng gói toàn bộ sức mạnh giao thức thành sản phẩm hoàn chỉnh đầu-cuối (Turnkey UX), xóa bỏ mọi đoạn mã giả lập cũ ở tầng giao diện dòng lệnh và bảng điều khiển.

**Independent Test**: Thực thi các lệnh dòng lệnh của Worker, Node và Client trên tiến trình thực, đồng thời kết nối tới máy chủ Dashboard để kiểm chứng luồng đẩy khung hình `60fps` và các nút tương tác kịch bản trên bảng `Worker Earnings & Opportunistic Studio`.

**Acceptance Scenarios**:

1. **Given** người dùng sở hữu địa chỉ Ví Lạnh và địa chỉ nút Seed, **When** chạy lệnh đơn `skynet-agent --opportunistic --owner <PUBKEY> --seed <ADDR>` (kèm tùy chọn `--schedule`, `--auto-sweep-skn`, `--game-watchlist`), **Then** tiến trình khởi chạy socket UDP thực, kích hoạt bộ điều tốc phần cứng thực và tự động đấu thầu – thực thi – nhận biên lai `NanoSkn` thực (không còn vòng lặp cộng điểm giả lập).
2. **Given** khách hàng sử dụng công cụ dòng lệnh `skynet`, **When** gọi các lệnh `skynet genesis generate|verify`, `skynet airdrop claim`, `skynet compute dispatch` và `skynet compute train`, **Then** hệ thống thực hiện kiểm chứng mật mã PoPE / Genesis thật và phát sóng các gói tin công việc chuẩn hóa qua mạng UDP tới các nút đang lắng nghe.
3. **Given** người dùng mở giao diện `skynet-dashboard` trên trình duyệt, **When** kết nối kênh truyền trực tiếp `60fps` và bấm các nút kiểm thử nhanh (*Giả lập Bật Game AAA*, *Giả lập Nhiệt Tăng Đột Biến*, *Bơm Worker Gian Lận Canary*, *Kích hoạt Quét Lãi*), **Then** bảng `Worker Earnings Studio` cập nhật tức thì trong `< 100ms` huy hiệu trạng thái bộ điều tốc, đồng hồ GPU/VRAM/CPU/Nhiệt độ/Pin, điểm uy tín PoUW, tốc độ thu nhập `SKN/giờ` và nhật ký giao dịch về Ví Lạnh.

---

### Edge Cases

- **Bật Game đột ngột đúng lúc đang ghi hoặc tính toán giữa lát huấn luyện:** Tín hiệu ngắt nguyên tử dừng ngay sau vi lát cắt hiện tại (`<= 20ms`), tệp `Micro-Checkpoint` được ghi qua tệp tạm rồi đổi tên nguyên tử (`.tmp -> rename`) nên kể cả mất điện hoặc tắt cưỡng bức cũng không bao giờ tạo tệp checkpoint hỏng nửa chừng.
- **Nhiệt độ GPU dao động liên tục quanh ngưỡng cắt (`77°C ↔ 79°C`):** Nhờ dải trễ nhiệt `8°C` (`cắt ở 78°C`, `chỉ mở lại ở <= 70°C`) kết hợp thời gian giữ nhàn rỗi `30 giây`, Worker tuyệt đối không bị hiện tượng bật/tắt liên hồi gây sụt giảm tuổi thọ linh kiện hoặc giật khung hình.
- **Mạng Wi-Fi / 4G mất gói nặng (`30%–40%`) hoặc nằm sau Symmetric NAT kép:** Các phân mảnh huấn luyện tự phục hồi nhờ giọt nước mã xóa dư thừa không cần gói báo nhận; biên lai `SynapticVoucher` mang giá trị tổng lũy kế nên mất biên lai cũ được bù đắp hoàn toàn bởi biên lai kế tiếp; nếu đục lỗ UDP trực tiếp thất bại sau loạt xung thăm dò, lưu lượng điều khiển tự động đi qua nút chuyển tiếp `GangliaRelay` (`<= 1200 Bytes`).
- **Worker gian lận trả về vector toàn số `0`, nhiễu ngẫu nhiên hoặc `NaN`/`Inf`:** Bộ kiểm định từ chối ngay các vector có `NaN`/`Inf` hoặc chuẩn L2 vượt ngưỡng, đồng thời lát bẫy `Canary Spot-Check` bắt giữ các Worker giả mạo tính toán để kích hoạt cắt cọc bảo lãnh tức thì.
- **Khách hàng phát việc rồi tắt máy giữa chừng sau khi Worker đã tính xong một phần:** Nhờ cơ chế giải ngân từng lát (`Chunk-Streaming Settlement`), Worker giữ các biên lai đã ký của những lát hoàn thành để tất toán đơn phương lên sổ cái và nhận lại phần hoàn cọc cho thời gian chờ hết hạn.
- **Máy tính không có GPU rời hoặc môi trường ảo hóa hạn chế cảm biến phần cứng:** Bộ thu thập thông số tự động chuyển về chế độ đọc tải CPU/RAM tiêu chuẩn kết hợp bộ dò độ trễ lát cắt nội tại mà không báo lỗi dừng chương trình hay yêu cầu quyền quản trị hệ thống.

---

## 3. Requirements *(mandatory)*

### Functional Requirements

- **FR-001** *(Khởi nguyên Sổ cái & Bất biến 21 Nghìn Tỉ SKN — `[US1]`)*: Hệ thống PHẢI định nghĩa cấu hình khởi nguyên `genesis.json`, tính toán mã băm định danh mạng `GENESIS_HASH` 32-byte tất định và cưỡng chế bất biến bảo toàn tuyệt đối $\text{Kho Halving (85\%)} + \text{Lưu thông ban đầu (15\%)} \equiv 21{,}000{,}000{,}000{,}000\text{ SKN}$ ($2.1 \times 10^{22}\text{ nanoSKN}$), từ chối nạp cấu hình nếu sai lệch dù chỉ $1\text{ nanoSKN}$.
- **FR-002** *(Màng lọc Biên Tế bào Da Không Cấp phát Động — `[US1]`)*: Mọi gói tin UDP đến ($\le 1200\text{ Bytes}$) PHẢI được kiểm tra định danh mạng khớp `GENESIS_HASH`, dung sai thời gian thực $\pm 30\text{ giây}$ (chống tấn công phát lại) và cấu trúc tiêu đề hợp lệ ngay trên bộ đệm ngăn xếp trong $< 5\mu\text{s}$ với $0$ lần cấp phát bộ nhớ động đối với gói không hợp lệ.
- **FR-003** *(Khám phá LAN, Đục lỗ NAT UDP Điểm hẹn & Chuyển tiếp Dự phòng — `[US1]`)*: Hệ thống PHẢI hỗ trợ phát hiện nút nội mạng LAN tự động (`< 1s`), giao thức báo danh định kỳ `PheromoneHello` (`25s`), bảng điểm hẹn hết hạn tự động (`120s`), ghép cặp đục lỗ đồng thời `PunchRendezvous` ($\le 300\text{ Bytes}$, bắn $3\text{--}5$ xung thăm dò UDP) và gói chuyển tiếp dự phòng `GangliaRelay` ($\le 1200\text{ Bytes}$) cho mạng NAT đối xứng.
- **FR-004** *(Giám sát Phần cứng & Tiến trình Đa nền tảng Thích nghi — `[US2]`)*: Hệ thống PHẢI thu thập không cần quyền quản trị các chỉ số tải GPU, VRAM, tải CPU, RAM, nhiệt độ (°C), trạng thái nguồn điện AC/Pin và tiến trình Game/ứng dụng toàn màn hình ưu tiên trên macOS, Linux và Windows với chu kỳ lấy mẫu thích nghi (`250ms` khi đang xử lý công việc, giãn ra `1500ms` khi đang tạm dừng).
- **FR-005** *(Nhường máy `< 100ms`, Vi điểm dừng Nguyên tử & Trễ nhiệt Chống nhấp nháy — `[US2]`)*: Hệ thống PHẢI ngắt vòng lặp tính toán trong `< 100ms` (mục tiêu `< 25ms` qua dò xung trễ lát cắt `> 1.8x EMA`) ngay khi phát hiện tiến trình Game/ứng dụng ưu tiên, rút sạc pin (khi bật bảo vệ pin), vượt trần VRAM (`50%`) hoặc chạm trần nhiệt (`78°C`); ghi nguyên tử trạng thái dở dang (`Micro-Checkpoint` `< 5ms`) và chỉ nối lại khi rảnh liên tục đủ `30 giây` đồng thời nhiệt độ hạ xuống `<= temp_limit - 8°C`.
- **FR-006** *(Huấn luyện Vi mô Nhiều bước, Tiến hóa `Plasmid` & Nén Trọng số `16-bit` — `[US3]`)*: Hệ thống PHẢI thực thi được các lát huấn luyện nơ-ron nhiều bước tối ưu nội bộ (`inner_steps`) và bản vá di truyền `Plasmid` trên GPU/CPU, xuất chênh lệch trọng số nén `16-bit` (hoặc dạng thưa Top-K cho tinh chỉnh nhẹ) kèm mã băm chuẩn hóa lượng tử hóa cố định (`thang 10,000.0`) khử nhiễu dấu phẩy động đa nền tảng.
- **FR-007** *(Truyền tải Phân mảnh Mã xóa Trên Khung tin `<= 1200 Bytes` & Miền Mã lệnh `0x13` — `[US3]`)*: Hệ thống PHẢI chia nhỏ dữ liệu huấn luyện và trọng số thành các phân mảnh ($\le 128\text{ KB}$/shard) truyền qua luồng giọt nước mã xóa trên khung tin UDP $\le 1200\text{ Bytes}$ (khôi phục toàn vẹn dưới mức mất gói `40%` không cần gói báo nhận), và đăng ký chính thức miền mã lệnh chuẩn `0x13` cho các tác vụ huấn luyện vi mô và tiến hóa liên kết.
- **FR-008** *(Kiểm chứng Bẫy `Canary Spot-Check`, Phạt Cắt cọc & Tổng hợp Chống đầu độc — `[US3]`)*: Hệ thống PHẢI chèn ngẫu nhiên các lát bẫy `Canary` tất định (mặc định `10%`), kiểm tra tính giảm hàm mất mát, giới hạn chuẩn L2 và độ tương đồng hướng cập nhật ($\text{Cosine Similarity} \ge 0.985$); tự động cắt cọc bảo lãnh của Worker nộp kết quả gian lận/`NaN` (thưởng nút kiểm toán + hoàn tiền khách hàng + đặt điểm uy tín PoUW về `0`) và tổng hợp các cập nhật hợp lệ bằng trung bình cắt tỉa (`Trimmed Mean` loại bỏ `10%` mỗi đầu cực trị).
- **FR-009** *(Đa hợp Tác vụ 3 Nấc & Chốt chặn Lợi nhuận Điện năng — `[US4]`)*: Hệ thống PHẢI điều phối công việc theo 3 nấc (`DeepIdle`: chạy huấn luyện vi mô đan xen suy luận `< 10ms` và tiền xử lý dữ liệu; `LightIdle`: dừng huấn luyện nặng, chỉ nhận suy luận `< 10ms` và kiểm toán Canary; `Paused*`: ngừng hoàn toàn tính toán AI) và tự động từ chối thầu nếu đơn giá phần thưởng thấp hơn chi phí điện năng ước tính trên mỗi giây tính toán.
- **FR-010** *(Thanh toán Streaming Từng Lát & Tự động Quét Lãi Lưu ký Soul–Body — `[US4]`)*: Hệ thống PHẢI giải ngân thu nhập tức thì theo từng lát hoàn thành (`Chunk-Streaming Settlement`) qua biên lai vi thanh toán lũy kế `NanoSkn` (hỗ trợ Worker tất toán đơn phương khi mất kết nối) và tự động tạo giao dịch `Yield Sweep` chuyển phần lãi ròng vượt ngưỡng (mặc định `50 SKN`) về địa chỉ Ví Lạnh của chủ sở hữu (`0%` phí đốt trao đổi chất), giữ nguyên vốn lưu động `AgentBond` và bảo toàn tuyệt đối quy tắc chặn chuyển hướng vốn gốc (`HumanPrincipalDiversionBlocked`) cùng quyền thu hồi tối cao (`Sovereign Reclaim`).
- **FR-011** *(Giao diện Dòng lệnh Turnkey Thực chiến Cho Worker, Node & Khách hàng — `[US1]`, `[US2]`, `[US3]`, `[US5]`)*: Hệ thống PHẢI cung cấp đầy đủ cờ khởi chạy 1 lệnh `skynet-agent --opportunistic --owner <PUBKEY> --seed <ADDR>` (cùng `--schedule`, `--auto-sweep-skn`, `--game-watchlist`, `--max-vram-pct`, `--idle-hold-secs`), cờ `--genesis <PATH>` trên nút mạng, và các lệnh `skynet genesis generate|verify`, `skynet airdrop claim`, `skynet compute dispatch`, `skynet compute train` hoạt động thật trên mạng UDP và sổ cái (xóa bỏ hoàn toàn các đoạn mã giả lập cũ).
- **FR-012** *(Trạm Giám sát Thời gian thực `60fps` & Bảng Điều khiển Worker Earnings Studio — `[US5]`)*: Trạm giám sát PHẢI hỗ trợ luồng đẩy dữ liệu trực tiếp `60fps` (kèm cơ chế dự phòng HTTP) hiển thị bảng `Worker Earnings & Opportunistic Studio` (huy hiệu trạng thái bộ điều tốc, độ trễ nhường máy, thông số GPU/VRAM/CPU/Nhiệt độ/Pin, tiến độ phân mảnh mã xóa, điểm uy tín PoUW, tốc độ thu nhập `nanoSKN` trực tiếp, lịch sử quét lãi về Ví Lạnh) và các nút kích hoạt mô phỏng tình huống thực tế ngay trên giao diện.

### Key Entities

- **`GenesisConfig` & `GenesisAllocation`**: Bản khai sinh mạng lưới quy định định danh mạng, tổng cung bất biến `21T SKN`, tỷ lệ khóa Kho Halving (`85%`) và phân bổ lưu thông ban đầu (`15%`), tạo nên mã băm định danh `GENESIS_HASH` 32-byte.
- **`RendezvousSession` (`PheromoneHello` / `PunchRendezvous` / `GangliaRelay`)**: Thực thể phiên bắt tay mạng ngang hàng lưu ánh xạ khóa công khai sang địa chỉ mạng ngoại vi thực tế, mã phiên đục lỗ NAT đồng thời và lộ trình chuyển tiếp dự phòng.
- **`GovernorPolicy` & `SystemTelemetrySnapshot`**: Tập quy tắc an toàn phần cứng (trần nhiệt, dải trễ nhiệt, ngưỡng VRAM, bảo vệ pin, lịch hoạt động, danh sách tiến trình ưu tiên, giá điện) và bản chụp chỉ số hệ thống tại từng thời điểm quyết định trạng thái vận hành (`DeepIdle`, `LightIdle`, `Throttled`, `Paused*`).
- **`MicroTrainShard` & `MicroTrainCheckpoint`**: Đơn vị công việc huấn luyện vi mô chứa siêu dữ liệu kiến trúc, số bước tối ưu nội bộ, phân mảnh trọng số/dữ liệu và bản lưu trạng thái trung gian nguyên tử cho phép tạm dừng – tiếp tục không mất tiến độ.
- **`WeightDeltaClaim`, `CanarySpotCheck` & `PlasmidPatch`**: Bản công bố chênh lệch trọng số nén `16-bit` kèm mã băm chuẩn hóa, hồ sơ bẫy kiểm định xác thực công việc hữu ích (PoUW) phục vụ thưởng/cắt cọc, và gói gen tiến hóa có hướng sau khi tổng hợp chống đầu độc.
- **`ChunkEscrowSchedule` & `YieldSweepOrder`**: Lịch trình giải ngân ký quỹ theo từng cột mốc phân mảnh kết hợp biên lai lũy kế `SynapticVoucher` và lệnh quét phần thặng dư thu nhập của Worker về Ví Lạnh chủ sở hữu theo chuẩn lưu ký **Soul–Body Custody**.
- **`WorkerStudioTelemetry`**: Cấu trúc dữ liệu tổng hợp thời gian thực đẩy lên giao diện giám sát `60fps` phản ánh sức khỏe phần cứng, trạng thái nhường máy, uy tín PoUW và dòng tiền thu nhập của Worker.

---

## 4. Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** *(Độ trễ nhường máy Zero-Impact)*: `100%` sự kiện bật Game/ứng dụng nặng hoặc chạm ngưỡng an toàn nhiệt/pin được bộ điều tốc phát hiện và dừng vòng lặp tính toán AI trong **`< 100ms`** (trung vị **`< 25ms`**), ghi tệp `Micro-Checkpoint` nguyên tử trong **`< 5ms`** và khôi phục chính xác `100%` bước đã hoàn thành khi máy nhàn rỗi trở lại.
- **SC-002** *(Kỷ luật khung tin UDP & Tốc độ lọc biên)*: `100%` gói tin giao thức có kích thước đường truyền **`<= 1200 Bytes`** (bản tin bắt tay điểm hẹn **`<= 300 Bytes`**); màng lọc biên xác thực `GENESIS_HASH` và khung thời gian `±30 giây` hoàn tất trong **`< 5µs`/gói** với **0 cấp phát bộ nhớ động** cho các gói tin rác/sai mạng.
- **SC-003** *(Chịu lỗi rớt gói & Hội tụ huấn luyện chống đầu độc)*: Truyền tải phân mảnh qua giọt nước mã xóa đạt tỷ lệ khôi phục **`100%` dưới mức rớt gói UDP ngẫu nhiên `40%`** mà không cần xin gửi lại tuần tự; mô hình huấn luyện vi mô đạt mức giảm hàm mất mát hội tụ ngay cả khi có tới **`30%` Worker gửi cập nhật nhiễu/Byzantine**.
- **SC-004** *(Hiệu quả PoUW Canary & Cắt cọc)*: Cơ chế kiểm chứng ngẫu nhiên `Canary Spot-Check` (`10%`) tiết kiệm **tối thiểu `85%` năng lực GPU/CPU xác thực** so với phương pháp tính lặp 3 lần toàn phần, đồng thời bắt giữ và cắt cọc **`100%` Worker nộp kết quả sai lệch** (`Cosine Similarity < 0.985` hoặc `NaN`/`Inf`).
- **SC-005** *(Bất biến Tiền tệ & Lưu ký Soul–Body)*: Tổng cung trên toàn bộ kho, sổ cái và kênh ký quỹ bảo toàn chính xác tuyệt đối **`21,000,000,000,000 SKN` (`2.1 × 10^22 nanoSKN`)** với sai lệch `0 nanoSKN` qua mọi chu trình giải ngân từng lát, phạt cắt cọc và `Yield Sweep`; `100%` giao dịch xâm phạm vốn gốc của chủ sở hữu bị chặn đứng.
- **SC-006** *(Kết nối P2P đa nút thực tế)*: Hai nút độc lập nạp cùng `genesis.json` tự động khám phá nội mạng LAN trong **`< 1 giây`** hoặc đục lỗ NAT UDP qua nút điểm hẹn trong **`< 3 giây`** và hoàn tất chu trình giao việc – huấn luyện vi mô – thanh toán biên lai thực trên socket mạng.
- **SC-007** *(Giám sát thời gian thực `60fps`)*: Trạm giám sát duy trì tốc độ đẩy dữ liệu **`60fps` (~`16.6ms`/khung hình)** và cập nhật trực quan mọi thay đổi trạng thái bộ điều tốc, cắt cọc PoUW và `Yield Sweep` trên bảng `Worker Earnings Studio` trong **`< 100ms`**.
- **SC-008** *(Chuẩn mực Vật lý Mã nguồn & Kiểm thử)*: `100%` tệp mã nguồn tạo mới hoặc chỉnh sửa tuân thủ mục tiêu `<= 250 dòng` (trần cứng `350 dòng`), hàm `<= 40 dòng`, tối đa `4` tham số, độ lồng `<= 3`, không thêm phụ thuộc ngoài danh mục chuẩn và đạt **`100%` vượt qua kiểm định biên dịch, kiểm tra quy tắc mã nguồn và bộ kiểm thử tự động**.

---

## 5. Assumptions & Deterministic Production Defaults

Để hệ thống vận hành tức thì với cấu hình tối ưu ngay từ lần chạy đầu tiên mà không bắt buộc người dùng tinh chỉnh thủ công (vẫn cho phép ghi đè qua cờ dòng lệnh), các giá trị mặc định chuẩn Production sau được ấn định rõ ràng:

- **Ngưỡng điều tốc phần cứng mặc định (`--opportunistic`):**
  - Trần nhiệt cắt tính toán (`temp_limit_c`): **`78°C`**; dải trễ hồi nhiệt (`hysteresis_cool_c`): **`8°C`** (chỉ nối lại khi nhiệt độ `<= 70°C`); ngưỡng giảm tải 50% (`Throttled`): **`73°C`** (`temp_limit_c - 5°C`).
  - Thời gian xác nhận nhàn rỗi liên tục trước khi chạy/nối lại (`idle_hold_secs`): **`30 giây`**.
  - Trần chiếm dụng bộ nhớ đồ họa tối đa (`max_vram_pct`): **`50%`** tổng bộ nhớ khả dụng.
  - Bảo vệ pin (`battery_guard`): **Bật mặc định (`true`)** — tạm dừng ngay khi chuyển sang nguồn pin.
  - Ngưỡng phát hiện tranh chấp lát cắt nội tại: Thời gian thực thi vi lát cắt vượt **`1.8×` trung bình động hàm mũ (`EMA`)**.
  - Chu kỳ lấy mẫu cảm biến hệ điều hành thích nghi: **`250ms`** khi đang tính toán hoạt động và **`1500ms`** khi đang ở trạng thái tạm dừng.
- **Ngưỡng huấn luyện vi mô, PoUW & tổng hợp mặc định:**
  - Kích thước tối đa mỗi phân mảnh truyền tải (`max_shard_bytes`): **`128 KB`** (`<= 256` khối mã xóa, nằm gọn trong giới hạn bộ mã hóa giọt nước hiện hữu và khung tin UDP `<= 1200 Bytes`).
  - Tỷ lệ chèn lát bẫy kiểm định (`canary_rate_bps`): **`1000 bps` (`10%`)**; ngưỡng tương đồng hướng cập nhật hợp lệ tối thiểu: **`Cosine Similarity >= 0.985`**; tỷ lệ cắt tỉa cực trị hai đầu khi tổng hợp (`trim_ratio_bps`): **`1000 bps` (`10%` mỗi đầu)**.
- **Ngưỡng mạng P2P & kinh tế lưu ký mặc định:**
  - Dung sai chống phát lại gói tin (`max_timestamp_drift_sec`): **`±30 giây`**; chu kỳ giữ cổng NAT (`seed_keepalive_sec`): **`25 giây`**; thời gian sống bảng điểm hẹn (`rendezvous_ttl_sec`): **`120 giây`**.
  - Ngưỡng tự động quét lãi về Ví Lạnh (`auto_sweep_skn`): **`50 SKN`** (`50,000,000,000 nanoSKN`), giữ lại khoản dự phòng lưu động tối thiểu được khai báo trong `AgentBond`.

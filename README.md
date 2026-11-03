# SkyNet — Mạng lưới AI Agent Đa bào & Sổ cái SKN trên Mạng UDP

**SkyNet** là nền tảng kết hợp giữa **Sinh học Đa bào**, **Mạng Nơ-ron Tiến hóa (`candle`)** và **Blockchain Đồng thuận Xác suất qua UDP**:

- **1 Node = 1 AI Agent (Cơ thể sống đa bào):** Tự động co giãn từ thiết bị ngoại vi CPU yếu (`TinyPeripheral` — vài chục tế bào) đến máy trạm/máy chủ mạnh (`ColossalOrganism` — hàng ngàn đến hàng vạn tế bào chạy theo lô Tensor 3 chiều).
- **1 Tế bào (`MicroCell`) = 1 AI Model siêu mini:** Khởi nguồn từ **Tế bào Gốc (`Stem`)**, tự biệt hóa qua công tắc thượng di truyền (`EpigeneticMask`) thành **Da (`Epidermal`)**, **Máu (`Vascular`)**, **Nội tạng (`Visceral`)** và **Thần kinh (`Neural`)**. Có khả năng kế thừa DNA, phân tách `1 -> 2` (`Mitosis`), đột biến tiến hóa trọng số và tự đào thải (`Apoptosis`).
- **Kinh tế học SKN bất biến:**
  - Tổng cung cố định tuyệt đối tại Genesis: **`21,000,000,000,000 SKN`** (21 nghìn tỉ SKN).
  - Đơn vị nguyên tử tối thiểu: **`1 nanoSKN`** (`10^-9 SKN`, lưu bằng `u128`).
  - Lịch phân phối **Halving**: Giảm **50%** mỗi năm.
- **Mạng UDP Chấp nhận Mất gói (`<= 1200 Bytes`):** Không dùng hàng đợi gửi lại kiểu TCP; đồng thuận lấy mẫu **Snowball** kháng rớt gói và truyền dữ liệu lớn không cần ACK qua **GF(2) Fountain Droplets**.

## Tài liệu & Sách trắng Đa ngôn ngữ (16 Ngôn ngữ Toàn cầu)
- **Cổng Sách trắng Đa ngôn ngữ (`16` ngôn ngữ):** [`docs/whitepaper/README.md`](docs/whitepaper/README.md) (`vi`, `en`, `zh-CN`, `ja`, `ko`, `es`, `hi`, `ar`, `fr`, `pt`, `ru`, `id`, `bn`, `de`, `tr`, `sw`)
  - [Bản Tiếng Việt (`vi`)](docs/whitepaper/vi/README.md) | [English Edition (`en`)](docs/whitepaper/en/README.md) | [Bảng Hằng số & Thuật ngữ Chuẩn (`GLOSSARY`)](docs/whitepaper/GLOSSARY.md)
- **Tóm tắt Kiến trúc Hệ thống:** [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)

## Cấu trúc Crates
- `crates/skynet-dna`: Mã di truyền `CellDna`, biệt hóa thượng di truyền, nhân đôi & lai ghép gen ngang.
- `crates/skynet-cell`: Mô hình Liquid Micro-Net dùng `candle-core` (CPU / Metal / CUDA) & `TissueBatch`.
- `crates/skynet-ledger`: Đơn vị `NanoSkn(u128)`, Kho Halving 21 nghìn tỉ SKN, Máu nội bào & `UtxoDagLedger`.
- `crates/skynet-wire`: Giao thức nhị phân UDP `<= 1200B`, chữ ký `Ed25519` & mã sửa lỗi `FountainEncoder`/`FountainDecoder`.
- `crates/skynet-agent`: Cơ thể `AgentNode` đa bào hoàn chỉnh & bộ đồng thuận `SnowballEngine`.
- `crates/skynet-sim`: Kịch bản mô phỏng sinh thái 6 AI Agents thực tế qua socket UDP có bơm 25% mất gói tin.

## Chạy Kiểm thử & Mô phỏng
```bash
# Chạy toàn bộ unit tests trên tất cả 6 crates
cargo test --workspace

# Chạy mô phỏng quần thể AI Agent đa bào qua mạng UDP rớt gói 25%
cargo run -p skynet-sim
```

## Khởi chạy Dashboard Trực quan hóa Thời gian thực
```bash
# Chạy máy chủ Dashboard trực tiếp (mặc định mở tại http://127.0.0.1:8787)
cargo run -p skynet-dashboard
```
Dashboard cho phép theo dõi trực tiếp ở tốc độ 60fps:
- Đồ thị 8 AI Agents đa bào (`ColossalOrganism`, `StandardNode`, `TinyPeripheral`) bắn xung UDP, tự phân tách tế bào, đào thải `Apoptosis` và truyền gen `Fountain DNA`.
- Kéo thanh trượt điều chỉnh **tỷ lệ rớt gói UDP (`0% – 55%`)** ngay khi mạng đang chạy.
- Bấm nút **+5.000 IP mới** để kiểm chứng bảng định tuyến `SynapticPeerTable` ($O(\log_2 N)$ Kademlia + cắt tỉa Hebbian) nén hàng vạn IP xuống chỉ còn vài KB RAM mỗi Node.
- Bắn thử nghiệm **Double-Spend** và thực thi **Thu hồi Khẩn cấp Ví Chủ nhân (`Sovereign Reclaim`)**.

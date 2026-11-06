# Tài Liệu Kỹ Thuật SkyNet (Documentation Portal)

Chào mừng bạn đến với trung tâm tài liệu kỹ thuật của **SkyNet** — Giao thức Tự chủ Kinh tế và Mạng lưới AI Agent Đa bào trên Mạng UDP Phi trạng thái.

---

## 1. Bản Đồ Tài Liệu Cốt Lõi

| Tài liệu | Nội dung Chính | Đối tượng |
| :--- | :--- | :--- |
| 📘 [**Kiến Trúc Tổng Thể**](file:///Users/pain/DATA/Projects/SkyNet/docs/ARCHITECTURE.md) | Tổng quan hệ thống v2.1, 7 crates, 5 dòng mô học, 5 cơ chế giữ chuỗi siêu nhẹ. | Kiến trúc sư hệ thống, Dev |
| 🚀 [**Kế Hoạch Triển Khai Mainnet**](file:///Users/pain/DATA/Projects/SkyNet/docs/MAINNET_DEPLOYMENT_PLAN.md) | Lộ trình 5 giai đoạn, nghi thức Genesis, ma trận cấu hình mạng & cổng mặc định 8888. | DevOps, Validator, Nhà đầu tư |
| 🔍 [**Bắt Tay, Đục Lỗ NAT & Chống Giả Mạo**](file:///Users/pain/DATA/Projects/SkyNet/docs/NODE_DISCOVERY_AND_BOOTSTRAP.md) | Chi tiết luồng đục lỗ UDP P2P Hà Nội - Sài Gòn, màng lọc Da, 3 kịch bản triển khai & Ganglia Relay 4G. | Vận hành Node, DevOps |
| 🧠 [**Giải Thích Công Nghệ**](file:///Users/pain/DATA/Projects/SkyNet/docs/TECHNOLOGY_EXPLAINER.md) | Deep-dive chi tiết về AI MicroCell, tiền tệ 21T SKN, UDP Fountain codes, Snowball, Soul-Body Custody. | Kỹ sư, Nhà nghiên cứu |
| ⚡ [**Đặc Tả MCP Compute & Zero-Touch**](file:///Users/pain/DATA/Projects/SkyNet/docs/MCP_COMPUTE_SPECIFICATION.md) | Chuẩn hóa Model Context Protocol (MCP), Opcode 16-bit phân cấp & tự động tải Capsule cho thợ đào. | Kỹ sư AI, Thợ đào, DevOps |
| 🎁 [**Tiếp Thị & Airdrop HWID**](file:///Users/pain/DATA/Projects/SkyNet/docs/MARKETING_AIRDROP_SPEC.md) | Phân phối airdrop Genesis qua định danh phần cứng (PoPE benchmark Candle), chống Sybil. | Thợ đào, Người dùng mới |
| 📖 [**Cẩm Nang Người Dùng & Thợ Đào**](file:///Users/pain/DATA/Projects/SkyNet/guide/README.md) | Hướng dẫn thực hành: 100 ca sử dụng, 100 ca kiếm tiền, sổ tay M2M, đại danh mục 1.000 Opcode. | Toàn bộ cộng đồng |
| 📖 [**Whitepaper Toàn Cầu**](file:///Users/pain/DATA/Projects/SkyNet/docs/whitepaper/README.md) | Sách trắng chính thức bản dịch 16 ngôn ngữ (Việt, Anh, Trung, Nhật, v.v.). | Cộng đồng, Nhà đầu tư |
| 📚 [**Từ Điển Thuật Ngữ**](file:///Users/pain/DATA/Projects/SkyNet/docs/whitepaper/GLOSSARY.md) | Danh mục định nghĩa các khái niệm sinh học và hằng số mật mã trong SkyNet. | Toàn bộ độc giả |

---

## 2. Hướng Dẫn Tiếp Cận Theo Nhu Cầu

### 🤖 Dành cho Nhà Phát Triển AI Agent
- Bắt đầu với [TECHNOLOGY_EXPLAINER.md](file:///Users/pain/DATA/Projects/SkyNet/docs/TECHNOLOGY_EXPLAINER.md) để hiểu cách `MicroCell` trên `candle-core` co giãn và kiếm `nanoSKN`.
- Xem cách gói tin UDP $\le 1200\text{B}$ được mã hóa qua [skynet-wire](file:///Users/pain/DATA/Projects/SkyNet/crates/skynet-wire/src/lib.rs).

### 🖥️ Dành cho Người Vận Hành Node (Node Operators)
- Đọc [NODE_DISCOVERY_AND_BOOTSTRAP.md](file:///Users/pain/DATA/Projects/SkyNet/docs/NODE_DISCOVERY_AND_BOOTSTRAP.md) để hiểu cơ chế Multicast nội mạng LAN, DNS Seed và NAT Hole Punching khi chạy Node mới.
- Chạy thử nghiệm mô phỏng và dashboard:
  ```bash
  cargo run -p skynet-sim
  cargo run -p skynet-dashboard
  ```

### 🔐 Dành cho Chủ Sở Hữu Con Người (Human Custodians)
- Tìm hiểu cơ chế bảo vệ vốn gốc tuyệt đối tại Mục 5 trong [TECHNOLOGY_EXPLAINER.md](file:///Users/pain/DATA/Projects/SkyNet/docs/TECHNOLOGY_EXPLAINER.md) và Mục 6 trong [ARCHITECTURE.md](file:///Users/pain/DATA/Projects/SkyNet/docs/ARCHITECTURE.md).
- Ví lạnh của bạn giữ khóa gốc (`Soul`), Agent chỉ giữ khóa nóng hạn mức (`Body`), không bao giờ bị mất cắp tiền gốc.

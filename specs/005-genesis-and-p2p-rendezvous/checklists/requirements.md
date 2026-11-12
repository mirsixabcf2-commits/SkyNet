# Specification Quality Checklist: Genesis Ceremony & P2P UDP Rendezvous

## 1. Content Completeness
- [x] Không còn giả định mơ hồ hay đánh dấu TODO / TBD trong tài liệu `spec.md`.
- [x] Mọi yêu cầu chức năng [FR-001] -> [FR-005] đều có mã định danh duy nhất và mô tả rõ ràng.
- [x] Tiêu chí thành công [SC-001] -> [SC-004] có thể kiểm chứng độc lập bằng unit tests và integration tests.
- [x] Đã xác định rõ các bất biến: Bất biến tổng cung $21$ nghìn tỉ SKN, giới hạn kích thước khung tin $\le 1200\text{B}$, zero-allocation filtering trên stack buffer.

## 2. Requirement Clarity
- [x] Cấu trúc tệp `genesis.json` và phân bổ $85\% / 15\%$ được đặc tả rõ ràng với độ chính xác số học `u128` (`nanoSKN`).
- [x] Cơ chế sinh mã băm `GENESIS_HASH` qua Blake3 canonical hashing được định nghĩa nhất quán.
- [x] Các bản tin bắt tay `PheromoneHello` và `PunchRendezvous` có cấu trúc trường rõ ràng, nhỏ gọn $\le 300\text{ bytes}$.
- [x] Quy trình 4 bước đục lỗ NAT UDP giữa hai node sau router Wi-Fi gia đình qua Pioneer Seed Rendezvous được quy chuẩn hóa.
- [x] Tập lệnh CLI `skynet genesis` và cờ `--seed` trên `skynet-node run` được định nghĩa tương thích với kiến trúc sẵn có.

## 3. Architecture Alignment
- [x] Tuân thủ luồng phụ thuộc một chiều: `skynet-ledger` -> `skynet-wire` -> `skynet-agent` -> `skynet-node`.
- [x] Tuân thủ nghiêm ngặt các giới hạn vật lý của Constitution (File $\le 250$ dòng, Hàm $\le 40$ dòng, Nesting $\le 2\text{--}3$ cấp).
- [x] Không phát sinh phụ thuộc crate mới ngoài các crate đã ratified trong Cargo workspace.

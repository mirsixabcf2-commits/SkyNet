# Specification Quality Checklist: Turnkey Preprocessing, Presets & Real-World CLI

## 1. Content Completeness
- [x] Không còn giả định mơ hồ hay đánh dấu TODO / TBD trong tài liệu.
- [x] Mọi yêu cầu chức năng [FR-001] -> [FR-005] đều có mã định danh duy nhất và mô tả rõ ràng.
- [x] Tiêu chí thành công [SC-001] -> [SC-004] có thể kiểm chứng bằng kiểm thử tự động (automated tests) và giao diện trực quan.
- [x] Đã xác định rõ các bất biến: Bảo toàn quyền riêng tư PII, chuẩn hóa tensor tương thích `CandleTaskInput`, tính toàn vẹn chữ ký và bảo toàn gói tin MTU <= 1200B.

## 2. Requirement Clarity
- [x] Danh sách các thuật toán tiền xử lý (PII masking, sliding-window chunking, MinMax scaling, E.164, envelope encryption) có định nghĩa đầu vào/đầu ra rõ ràng.
- [x] Danh mục Presets (`text-audit`, `csv-clean-normalize`, `image-batch-upscale`, `log-anomaly-detection`, `phone-standardize`) có đầy đủ opcode và bounty gợi ý.
- [x] Danh sách lệnh và cờ dòng lệnh CLI (`skynet` và `skynet-agent`) đồng bộ 100% với các kịch bản trong thư mục `guide/`.
- [x] Bảng điều khiển trên Dashboard có phương án tương tác trực quan 2 cột (Raw Input vs Cleaned Output) và 1-click execution.

## 3. Architecture Alignment
- [x] Tuân thủ cấu trúc crate phân tầng: `crates/skynet-node` cung cấp động cơ thực thi, CLI và tiền xử lý, phối hợp cùng `skynet-agent`, `skynet-wire`, `skynet-cell`.
- [x] Tuân thủ giới hạn kích thước vật lý của Constitution (File <= 250 dòng, hàm <= 40 dòng, nesting <= 3).
- [x] Đảm bảo không phụ thuộc vòng (No circular dependency) và không tạo file helper/util rác chung chung.

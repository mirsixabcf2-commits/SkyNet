# Hệ Thống Quản Lý Nhiệm Vụ SkyNet (`Task/`)

Thư mục này quản lý toàn bộ các kế hoạch, nhiệm vụ kỹ thuật, vá lỗi bảo mật và nâng cấp hệ thống của dự án **SkyNet** theo dòng thời gian thực thi (Timestamp-based Task Tracking).

---

## 1. Quy Chuẩn Đặt Tên File Task

Mọi file task mới phải được đặt tên theo định dạng timestamp chuẩn hóa:
```text
YYYY-MM-DD_HH-mm-ss_<ten_nhiem_vu_ngan_gon>.md
```
- **Ví dụ:** `2026-11-04_182500_security_hardening.md`
- **Quy tắc:** Sử dụng giờ địa phương chuẩn ISO, ký tự viết thường (`snake_case`), không dấu, gạch nối giữa các từ.

---

## 2. Cấu Trúc Bắt Buộc Của Mỗi File Task

Mỗi file task phải tuân thủ cấu trúc định dạng sau:

```markdown
# [MÃ-TASK] Tiêu Đề Nhiệm Vụ

## Thông Tin Chung
- **Mã Định Danh:** TASK-YYYYMMDD-XXX
- **Mức Độ Ưu Tiên:** Critical / High / Medium / Low
- **Thời Điểm Mở (Open Time):** YYYY-MM-DD HH:mm:ss (+07:00)
- **Thời Điểm Đóng (Closed Time):** YYYY-MM-DD HH:mm:ss (+07:00) (hoặc [Đang Mở])
- **Trạng Thái Hiện Tại:** [Mở | Đang Thực Hiện | Đã Hoàn Thành | Đã Hủy]
- **Phạm Vi / Crates Ảnh Hưởng:** `skynet-agent`, `skynet-ledger`, ...

## Bảng Checklist Chi Tiết
| ID | Hạng Mục Công Việc | Trạng Thái | Thời Điểm Mở (Open Time) | Thời Điểm Đóng (Closed Time) | Ghi Chú Kỹ Thuật |
| :--- | :--- | :---: | :---: | :---: | :--- |
| T01 | Mô tả công việc... | [ ] | YYYY-MM-DD HH:mm:ss | -- | Chi tiết triển khai |

## Nhật Ký Tiến Độ (Activity Log)
- `YYYY-MM-DD HH:mm:ss`: Khởi tạo nhiệm vụ...
```

---

## 3. Danh Sách Các File Task Hiện Có

| Thời Điểm (Timestamp) | Tên File Task | Chủ Đề / Mục Tiêu | Trạng Thái |
| :--- | :--- | :--- | :---: |
| `2026-11-04 18:25:00` | [`2026-11-04_182500_security_hardening.md`](file:///Users/pain/DATA/Projects/SkyNet/Task/2026-11-04_182500_security_hardening.md) | Khắc phục 11 rủi ro bảo mật & chống gian lận hệ thống | **Đã Hoàn Thành** |

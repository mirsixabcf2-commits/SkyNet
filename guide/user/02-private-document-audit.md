# Ca Thực Chiến 02: Phân Tích & Kiểm Toán Hợp Đồng Mật 500 Trang (Private AI Document Audit)

---

## 1. Tình Huống Cụ Thể Của Bạn

- **Hiện trạng:** Bạn là một luật sư, giám đốc tài chính hoặc chuyên viên M&A. Bạn đang nắm trong tay tệp hồ sơ sáp nhập doanh nghiệp dài **500 trang** (`contract_audit.pdf`, dung lượng ~18 MB), chứa bảng lương toàn bộ nhân sự, giá trị định giá độc quyền và các điều khoản phạt vi phạm hàng triệu USD.
- **Nhu cầu:** Bạn cần dùng mô hình ngôn ngữ lớn (LLM 70B thông minh) để rà soát toàn bộ 500 trang, phát hiện các điều khoản bẫy pháp lý, mâu thuẫn số liệu và tổng hợp thành bản tóm tắt rủi ro 5 trang.
- **Nỗi sợ hãi tột cùng với Cloud truyền thống:**
  - Nếu tải file lên ChatGPT/Claude hoặc gọi API OpenAI: Dữ liệu bay thẳng lên máy chủ của bên thứ ba ở Mỹ. Bạn có nguy cơ đối mặt với án phạt nặng vì làm lộ bí mật kinh doanh, vi phạm quy định bảo vệ dữ liệu (GDPR/Luật An ninh mạng).
  - Nếu muốn tự chạy mô hình 70B cục bộ (Local LLM): Bạn cần dàn máy trạm có ít nhất **48GB VRAM** (trị giá hơn 150 triệu VNĐ), điều mà chiếc laptop cá nhân của bạn không thể đáp ứng.
- **Mục tiêu với SkyNet:**
  - Thuê một máy chủ GPU khủng từ xa trên mạng SkyNet để chạy mô hình AI 70B trong **3 phút**.
  - **Bảo mật tuyệt đối $100\%$:** Chủ máy thợ không thể xem trộm nội dung hợp đồng, hacker đứng giữa không thể nghe lén, OpenAI không có dữ liệu.
  - Chi phí: Chỉ khoảng **15,000 nanoSKN** (~vài ngàn đồng).

---

## 2. Thao Tác Thực Tế Trên Máy Tính Của Bạn

### Bước 1: Chuẩn bị tệp tài liệu mật
Đặt tệp hợp đồng vào thư mục làm việc của bạn:
`~/confidential/merger_contract.pdf`

---

### Bước 2: Phát lệnh phân tích với cờ bảo vệ phần cứng TEE (Hardware Enclave)

Gõ dòng lệnh sau vào Terminal:

```bash
skynet compute dispatch \
  --input ~/confidential/merger_contract.pdf \
  --output ~/confidential/audit_report.md \
  --opcode 0x0109 \
  --model "Llama-3.3-70B-Instruct-Q4" \
  --prompt "Rà soát toàn bộ hợp đồng, chỉ ra 10 điều khoản bẫy pháp lý và lập bảng đối chiếu nghĩa vụ bồi thường." \
  --require-tee \
  --bounty 15000 \
  --timeout 300
```

#### Ý nghĩa của tham số đặc biệt `--require-tee`:
- Bắt buộc mạng lưới chỉ được giao tác vụ này cho các máy chủ có trang bị công nghệ **Vùng Nhớ Bảo Mật Phần Cứng (Trusted Execution Environment — TEE)** như Intel SGX hoặc AMD SEV-SNP.
- Những máy tính thông thường không có chứng thực phần cứng TEE sẽ bị từ chối ngay lập tức ở tầng giao thức.

---

### Bước 3: Xem báo cáo kết quả hoàn thành

Sau 2 phút 40 giây, màn hình hiển thị:

```text
🔒 [SkyNet Security] Đã xác thực thành công Hardware Remote Attestation Quote (Intel DCAP).
🔑 Khóa phiên tạm thời (Ephemeral Key) đã được nạp an toàn vào Enclave máy thợ.
⚡ Mô hình Llama-3.3-70B đang xử lý 500 trang tài liệu trong vùng nhớ cô lập...
📥 Đã nhận kết quả báo cáo mã hóa từ máy thợ.
✨ Giải mã thành công bằng khóa bí mật cục bộ của bạn!

🎉 HOÀN TẤT KIỂM TOÁN HỢP ĐỒNG!
📄 Báo cáo phân tích rủi ro đã lưu tại: ~/confidential/audit_report.md
💰 Chi phí quyết toán: 15,000 nanoSKN
```

Mở tệp `~/confidential/audit_report.md`, bạn có ngay bản phân tích pháp lý sắc sảo, đầy đủ dẫn chứng điều khoản và bảng tính rủi ro chi tiết.

---

## 3. Cơ Chế Bảo Vệ "Hộp Đen Bất Khả Xâm Phạm" (Hardware TEE Pipeline)

Làm sao bạn có thể tin tưởng một chiếc máy tính của một người lạ ở nửa bên kia bán cầu? Đây là kiến trúc bảo vệ dữ liệu của SkyNet:

```
                            QUY TRÌNH MÃ HÓA PHONG BÌ & TEE
Laptop Của Bạn (Client)                                          Máy Chủ Thợ (Worker Node)
+------------------------------------+               +--------------------------------------+
| 1. Sinh khóa ngẫu nhiên K_task     |               | 3. Chip CPU (Intel/AMD) sinh bằng    |
|    (ChaCha20-Poly1305 256-bit)     |               |    chứng phần cứng: Remote Quote     |
| 2. Mã hóa file PDF bằng K_task     |               +--------------------------------------+
+------------------------------------+                                  │
                  │                                                     ▼
                  │                                             +────────────────────────+
                  │ 4. Kiểm tra chữ ký chip CPU thành công      | VÙNG NHỚ BẢO MẬT (TEE) |
                  ├────────────────────────────────────────────►| Enclave Bất Khả Xâm    |
                  │ Gửi K_task đã mã hóa bằng Enclave PubKey    | Phạm                   |
                  │ Gửi File PDF đã mã hóa qua Fountain UDP     |                        |
                  │                                             | 5. Giải mã K_task      |
                  │                                             | 6. Đọc PDF & Chạy AI   |
                  ▼                                             | 7. Mã hóa kết quả      |
+------------------------------------+                          | 8. Xóa trắng RAM       |
| 9. Bạn nhận bản tóm tắt, giải mã   |◄─────────────────────────+────────────────────────+
|    bằng K_task cục bộ              |           Gửi Report đã mã hóa qua UDP
+------------------------------------+
```

### 3 Tầng Bảo Mật Tuyệt Đối:
1. **Dữ liệu trên đường truyền:** Được mã hóa bằng `ChaCha20-Poly1305`. Hacker có bắt trộm gói tin UDP trên đường truyền Internet cũng chỉ thấy những chuỗi byte ngẫu nhiên vô nghĩa.
2. **Dữ liệu trên máy thợ:** Ngay cả khi chủ sở hữu máy thợ có quyền quản trị tối cao (`root / administrator`), cố tình dùng phần mềm soi RAM hay cắm máy đọc tín hiệu vật lý vào bo mạch chủ, họ cũng **không thể đọc được dữ liệu**. Công nghệ AMD SEV / Intel SGX mã hóa toàn bộ dữ liệu trong thanh RAM bằng các khóa phần cứng được khắc cố định bên trong con chip CPU.
3. **Tự hủy tức thì (Zero-Trace Oblivion):** Ngay sau khi ký số và gửi bản báo cáo phân tích về máy bạn, vùng nhớ Enclave tự động được ghi đè bằng các byte `0x00`. Không một trang văn bản nào còn tồn tại trên máy thợ.

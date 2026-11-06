# Ca Thực Chiến 03: Nông Trại Render 3D Blender 300 Khung Hình 4K (Distributed 3D Render Farm)

---

## 1. Tình Huống Cụ Thể Của Bạn

- **Hiện trạng:** Bạn là một nghệ sĩ đồ họa 3D, kiến trúc sư hoặc nhà làm phim hoạt hình. Bạn vừa hoàn thiện một cảnh quay 10 giây (30 khung hình/giây = **300 frames**) trong phần mềm Blender (`sci_fi_city.blend`, dung lượng 85 MB).
- **Yêu cầu kỹ thuật:** Render ở độ phân giải **4K UHD** ($3840 \times 2160$), sử dụng bộ dựng hình Raytracing vật lý **Blender Cycles**, 256 samples khử nhiễu OptiX.
- **Vấn đề trên máy tính cá nhân của bạn:**
  - Bạn có một chiếc PC với card đồ họa tầm trung RTX 3060.
  - Mỗi khung hình 4K mất khoảng **3 phút** để render xong.
  - $300 \text{ frames} \times 3 \text{ phút} = 900 \text{ phút} = \mathbf{15 \text{ tiếng đồng hồ}}$ cày liên tục!
  - Trong suốt 15 tiếng đó, máy tính bị chiếm dụng 100% GPU, bạn không thể làm việc khác, phòng ngủ nóng bức như lò sưởi, và nếu lỡ mất điện ở tiếng thứ 14 thì công sức đi tong.
  - Nếu gửi lên các nông trại render thương mại Web2 (FoxRenderfarm, RebusFarm): Chi phí từ **$30 đến $60** và phải xếp hàng chờ đợi cả tiếng đồng hồ.
- **Mục tiêu với SkyNet:**
  - Chia 300 khung hình cho **15 đến 20 máy thợ GPU mạnh** trên thế giới cùng làm song song.
  - Thời gian render giảm từ 15 tiếng xuống còn **chưa đầy 25 phút**.
  - Chi phí: Chỉ khoảng **45,000 nanoSKN** (~vài ngàn đồng), trả tiền theo từng khung hình hoàn thành.

---

## 2. Thao Tác Thực Tế Trên Máy Tính

### Bước 1: Chuẩn bị tệp dự án 3D
Đảm bảo các texture và chất liệu đã được nhúng vào file `.blend`:
`~/projects/blender/sci_fi_city.blend`

Tạo thư mục nhận khung hình hoàn thiện:
```bash
mkdir -p ~/projects/blender/frames_4k
```

---

### Bước 2: Phát lệnh render phân tán trên SkyNet

Gõ lệnh sau vào Terminal:

```bash
skynet compute dispatch \
  --input ~/projects/blender/sci_fi_city.blend \
  --output ~/projects/blender/frames_4k/ \
  --opcode 0x0216 \
  --frames 1-300 \
  --bounty-per-item 150 \
  --max-workers 20 \
  --timeout 1800
```

#### Ý nghĩa các tham số:
- `--opcode 0x0216`: Mã tác vụ render đồ họa 3D Blender Cycles.
- `--frames 1-300`: Phân rã dự án thành 300 nhiệm vụ con (mỗi frame là 1 nhiệm vụ).
- `--bounty-per-item 150`: Trả công **150 nanoSKN** cho mỗi khung hình 4K hoàn thành hợp lệ (tổng 300 frames = 45,000 nanoSKN).
- `--max-workers 20`: Cho phép tối đa 20 máy thợ GPU nhận việc cùng lúc.

---

### Bước 3: Quan sát 20 máy thợ cùng cày song song

Màn hình Terminal hiển thị lưới giám sát trực quan thời gian thực:

```text
🎬 [SkyNet Render Farm] Đang phát tán dự án sci_fi_city.blend (85 MB)...
📦 Đã nén và chia nhỏ thành các Fountain Droplets GF(2) qua UDP.
🌐 18 Máy Thợ GPU đã tham gia (RTX 4090 x 8, RTX 4080 x 6, RTX 3090 x 4).

[Worker #01 - RTX 4090]: Render Frame 001..015 -> Hoàn tất! [150 nanoSKN]
[Worker #02 - RTX 4090]: Render Frame 016..030 -> Hoàn tất! [150 nanoSKN]
[Worker #03 - RTX 4080]: Render Frame 031..045 -> Đang xử lý 80%...
...
Tiến độ chung: [====================================>   ] 285/300 Frames (95%)
- Thời gian thực tế: 21 phút 10 giây
- Tiền đã giải phóng: 42,750 nanoSKN

🎉 TOÀN BỘ 300 KHUNG HÌNH 4K ĐÃ HOÀN TẤT!
📂 Thư mục: ~/projects/blender/frames_4k/ (frame_0001.png -> frame_0300.png)
```

Chỉ sau 23 phút, bạn đã có trọn vẹn 300 khung hình 4K trong tay. Bạn chỉ cần mở Blender hoặc Premiere Pro kéo vào để xuất video thành phẩm.

---

## 3. Cơ Chế Thanh Toán Vi Mô Từng Khung Hình (`SynapticVoucher`)

Trong các tác vụ render số lượng lớn, nếu máy thợ số 7 bị rớt mạng thì sao?

SkyNet áp dụng **Kênh thanh toán vi mô lũy kế (`SynapticVoucher`)**:
- Bạn không cần trả trước toàn bộ tiền.
- Cứ mỗi khi một máy thợ nộp về 1 khung hình đã render xong:
  - Máy bạn tự động kiểm tra tính hợp lệ (ảnh không bị rách, kích thước đúng $3840 \times 2160$, mã băm BLAKE3 hợp lệ).
  - Máy bạn tự động gửi một gói tin UDP siêu nhẹ ký nhận thêm **150 nanoSKN** cho thợ đó.
- Nếu một máy thợ mới render được 4 khung hình mà bị cúp điện:
  - Thợ đó nhận đúng 600 nanoSKN cho 4 khung hình đã giao.
  - Các khung hình còn lại tự động được hệ thống chuyển giao cho máy thợ khác render tiếp.
  - **Không một bên nào chịu thiệt thòi!**

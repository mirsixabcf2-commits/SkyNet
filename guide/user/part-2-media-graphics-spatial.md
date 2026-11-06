# Phần 2: 20 Tình Huống Đồ Họa, 3D & Đa Phương Tiện (Cases 021 – 040)

---

### Case 021: Nông Trại Render 3D Blender Cycles 300 Khung Hình 4K Song Song
- **Bối cảnh:** Đoạn phim hoạt hình 10 giây (300 frames) 4K Raytracing, cày trên PC cá nhân mất 15 tiếng.
- **Đầu vào / Đầu ra:** File `project.blend` (85 MB) $\to$ Thư mục 300 ảnh PNG 4K (`frame_0001.png` .. `frame_0300.png`).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input project.blend --output ./frames_4k/ --opcode 0x0216 --frames 1-300 --bounty-per-item 150 --max-workers 20
  ```
- **Kinh tế:** Tốn **45,000 nanoSKN**, 18 máy thợ GPU hoàn tất trong **23 phút** (tiết kiệm 95% thời gian).
- *Chi tiết xem tại:* [03-blender-3d-render-farm.md](./03-blender-3d-render-farm.md).

---

### Case 022: Huấn Luyện 3D Gaussian Splatting (3DGS) Từ 200 Ảnh Chụp Drone
- **Bối cảnh:** Kiến trúc sư có 200 ảnh chụp flycam khu đất dự án, cần tạo mô hình không gian 3D tương tác thời gian thực.
- **Đầu vào / Đầu ra:** Thư mục 200 ảnh chụp có tọa độ COLMAP $\to$ File point cloud `point_cloud.ply` (30,000 Gaussians).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./drone_dataset/ --output ./splat_3d/ --opcode 0x0217 --iterations 30000 --bounty 40000 --timeout 2400
  ```
- **Kinh tế:** Tốn **40,000 nanoSKN**, Worker có GPU 24GB VRAM tối ưu hội tụ trong 20 phút.

---

### Case 023: Khớp Trường Bức Xạ Thần Kinh NeRF Tái Lập Không Gian Nội Thất
- **Bối cảnh:** Công ty bất động sản cần tạo tour xem nhà thực tế ảo NeRF từ video quay chậm quanh căn hộ.
- **Đầu vào / Đầu ra:** Video quay 60 giây $\to$ Mô hình trường mật độ thần kinh Instant-NGP.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input apartment_tour.mp4 --output ./nerf_model/ --opcode 0x0218 --bounty 25000
  ```
- **Kinh tế:** Tốn **25,000 nanoSKN**, hoàn thành trong 5 phút.

---

### Case 024: Tái Tạo Lưới Đa Giác 3D Photogrammetry Từ 1,000 Ảnh Quét Cổ Vật
- **Bối cảnh:** Bảo tàng cần số hóa 3D chiếc bình cổ triều Nguyễn từ 1,000 ảnh chụp siêu phân giải macro.
- **Đầu vào / Đầu ra:** Thư mục 1,000 ảnh RAW $\to$ Mô hình lưới 3D chi tiết cao `antique_vase.obj` + bộ Texture 8K.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./vase_photos/ --output ./3d_mesh/ --opcode 0x0219 --bounty 60000 --timeout 3600
  ```
- **Kinh tế:** Tốn **60,000 nanoSKN**, máy thợ chạy Meshroom/COLMAP đa luồng giải quyết trong 40 phút.

---

### Case 025: Chuyển Mã Video 8K ProRes Sang Chuẩn Nén AV1 Chia Nhỏ Theo Cụm GOP
- **Bối cảnh:** Hãng phim có tệp video 8K quay bằng máy RED dài 15 phút (dung lượng 120GB), cần nén thành AV1 phân phối online.
- **Đầu vào / Đầu ra:** File `raw_8k.mov` $\to$ File nén `stream_8k.mp4` chuẩn AV1 (chỉ 2.5GB mà không giảm độ nét).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input raw_8k.mov --output ./encoded_av1/ --opcode 0x021A --codec av1 --bounty 50000 --parallel 12
  ```
- **Kinh tế:** Video được cắt thành 50 phân đoạn GOP, 12 máy thợ mã hóa song song trong **14 phút**, chi phí **50,000 nanoSKN**.

---

### Case 026: Nội Suy Tăng Tốc Khung Hình Video Từ 24fps Lên 120fps (RIFE AI)
- **Bối cảnh:** Video cảnh hành động quay chậm 10 giây cần tăng độ mượt lên 120 khung hình/giây để phát trên màn hình chuyên dụng.
- **Đầu vào / Đầu ra:** File `action_24fps.mp4` $\to$ File `action_120fps.mp4` (2,400 khung hình nội suy quang thông lượng).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input action_24fps.mp4 --output action_120fps.mp4 --opcode 0x021B --model "RIFE-v4.6" --bounty 8000
  ```
- **Kinh tế:** Tốn **8,000 nanoSKN**, Worker GPU xử lý hoàn tất trong 45 giây.

---

### Case 027: Nướng Bản Đồ Ánh Sáng Tĩnh (Light Baking) Cho Màn Game Unity
- **Bối cảnh:** Lập trình viên game indie cần nướng Lightmap cho bản đồ thành phố rộng lớn để game chạy mượt trên điện thoại.
- **Đầu vào / Đầu ra:** Scene Unity và tệp chiếu sáng $\to$ Bộ bản đồ ánh sáng `Lightmap_*.exr`.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./game_level/ --output ./baked_lightmaps/ --opcode 0x021C --bounty 30000
  ```
- **Kinh tế:** Tốn **30,000 nanoSKN**, hoàn thành trong 15 phút thay vì máy cá nhân cày suốt đêm.

---

### Case 028: Sinh Bản Đồ Chất Liệu PBR Textures (Albedo, Normal, Height) Bằng AI
- **Bối cảnh:** Họa sĩ game chụp ảnh bức tường gạch vỡ, cần tạo trọn bộ chất liệu 3D PBR tương thích Unreal Engine 5.
- **Đầu vào / Đầu ra:** 1 ảnh chụp bề mặt gạch $\to$ 5 bản đồ chất liệu (Albedo, Normal, Roughness, Height, Ambient Occlusion).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input brick_wall.jpg --output ./pbr_brick/ --opcode 0x021D --bounty 300
  ```
- **Kinh tế:** Tốn **300 nanoSKN**, xong trong 4 giây.

---

### Case 029: Phục Chế Màu Phim Tư Liệu Lịch Sử Đen Trắng Sang Màu 4K
- **Bối cảnh:** Đoạn phim tài liệu lịch sử năm 1945 dài 5 phút bị ố mờ, xước phim và chỉ có 2 màu đen trắng.
- **Đầu vào / Đầu ra:** Video đen trắng 480p $\to$ Video 4K phục hồi màu sắc tự nhiên và khử xước quang học bằng Deep-Colorize AI.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input historic_footage.mp4 --output restored_4k.mp4 --opcode 0x021E --bounty 20000 --parallel 6
  ```
- **Kinh tế:** Tốn **20,000 nanoSKN**, 6 máy thợ chia nhau xử lý trong 6 phút.

---

### Case 030: Lọc Nhiễu Điểm Point Cloud LiDAR 50 Triệu Điểm Cho Khảo Sát Địa Hình
- **Bối cảnh:** Dữ liệu quét LiDAR từ máy bay trực thăng bị lẫn hàng triệu điểm tán xạ do cây cối, chim chóc và bụi sương mù.
- **Đầu vào / Đầu ra:** Tệp LAS 50 triệu điểm (2.5GB) $\to$ Tệp LAS sạch chỉ chứa mặt đất phục vụ trắc địa.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input terrain_raw.las --output terrain_clean.las --opcode 0x021F --bounty 12000
  ```
- **Kinh tế:** Tốn **12,000 nanoSKN**, thuật toán k-NN phân tán xử lý trong 3 phút.

---

### Case 031: Mô Phỏng Vật Lý Khói Lửa & Chất Lỏng (Fluid Simulation) Bằng Blender MantaFlow
- **Bối cảnh:** Cảnh quay đập vỡ hồ chứa nước cần mô phỏng 10 triệu hạt chất lỏng dâng trào trong 5 giây.
- **Đầu vào / Đầu ra:** Kịch bản cấu hình mô phỏng $\to$ Tập dữ liệu hạt nung kết `fluid_cache/` (35 GB).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --config fluid_sim.blend --output ./cache/ --opcode 0x0220 --bounty 45000 --timeout 3600
  ```
- **Kinh tế:** Tốn **45,000 nanoSKN**, máy thợ RAM 64GB nướng cache trong 28 phút.

---

### Case 032: Xóa Watermark & Vật Thể Thừa Khỏi Video 4K Bằng Video Inpainting
- **Bối cảnh:** Video phong cảnh 4K vô tình bị người đi bộ và logo quảng cáo lọt vào khung hình trong suốt 30 giây.
- **Đầu vào / Đầu ra:** Video gốc + Mặt nạ vùng cần xóa (Mask) $\to$ Video 4K sạch bóng với phần nền được tái tạo liền mạch.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --video scene_4k.mp4 --mask mask.png --output clean_scene.mp4 --opcode 0x0221 --model "E2FGVI" --bounty 10000
  ```
- **Kinh tế:** Tốn **10,000 nanoSKN**, hoàn tất trong 2 phút.

---

### Case 033: Tính Toán Trường Ánh Sáng VR/AR Cho Kính Apple Vision Pro
- **Bối cảnh:** Nhà phát triển ứng dụng không gian số cần tính toán phản xạ ánh sáng tương tác đa góc nhìn cho phòng trưng bày ảo.
- **Đầu vào / Đầu ra:** Dữ liệu không gian phòng $\to$ Bản đồ Light Field nén chuẩn USDZ.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input room_scan/ --output room_lightfield.usdz --opcode 0x0222 --bounty 35000
  ```
- **Kinh tế:** Tốn **35,000 nanoSKN**, xong trong 18 phút.

---

### Case 034: Tạo Avatar 3D Siêu Chân Thực Từ 3 Ảnh Chụp Chân Dung
- **Bối cảnh:** Người dùng muốn tạo mô hình 3D khuôn mặt của chính mình để làm nhân vật trong game nhập vai.
- **Đầu vào / Đầu ra:** 3 ảnh chụp góc chính diện và 2 bên tai $\to$ File mô hình 3D `avatar.fbx` có đầy đủ xương khớp biểu cảm facial blendshapes.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./selfies/ --output my_avatar.fbx --opcode 0x0223 --bounty 5000
  ```
- **Kinh tế:** Tốn **5,000 nanoSKN**, hoàn thành trong 1 phút 15 giây.

---

### Case 035: Render Kiến Trúc Quy Hoạch Đô Thị 50 Hecta Bằng V-Ray
- **Bối cảnh:** Công ty kiến trúc cần kết xuất ảnh phối cảnh chim bay góc rộng độ phân giải 12,000 x 8,000 pixels cho triển lãm.
- **Đầu vào / Đầu ra:** File 3ds Max / V-Ray Scene (450 MB) $\to$ Ảnh siêu phân giải 96 Megapixels.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input masterplan.max --output render_96mp.tif --opcode 0x0224 --tiles 16 --bounty 75000
  ```
- **Kinh tế:** Tác vụ được chia thành 16 lát gạch (tiles), 8 máy thợ render song song trong **12 phút**, chi phí **75,000 nanoSKN**.

---

### Case 036: Tự Động Cắt Ghép Highlight Trận Đấu Bóng Đá 90 Phút
- **Bối cảnh:** Biên tập viên thể thao cần trích xuất 10 bàn thắng và pha bóng nguy hiểm nhất từ luồng phát sóng 90 phút.
- **Đầu vào / Đầu ra:** Video trận đấu 90 phút (1080p 60fps) $\to$ Video tổng hợp highlight 7 phút kèm hiệu ứng chuyển cảnh.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input match_full.mp4 --output highlights.mp4 --opcode 0x0225 --bounty 8000
  ```
- **Kinh tế:** Tốn **8,000 nanoSKN**, mô hình AI phân tích âm thanh reo hò và thị giác quả bóng cắt xong trong 3 phút.

---

### Case 037: Tách Phông Xanh Tự Động Không Cần Màn Xanh Vật Lý (AI Background Matting)
- **Bối cảnh:** Diễn viên quay video trước bức tường văn phòng bình thường, cần tách nền tóc tơ siêu mịn để ghép vào bối cảnh ngoài không gian.
- **Đầu vào / Đầu ra:** Video 4K 30 giây $\to$ Video có kênh trong suốt Alpha Channel ProRes 4444.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input actor_office.mp4 --output actor_alpha.mov --opcode 0x0226 --model "RobustVideoMatting" --bounty 6000
  ```
- **Kinh tế:** Tốn **6,000 nanoSKN**, xử lý mượt mà 900 khung hình trong 1 phút 10 giây.

---

### Case 038: Biến Đổi Giọng Nói Diễn Viên Lồng Tiếng Sang 10 Ngôn Ngữ Giữ Nguyên Ngữ Điệu
- **Bối cảnh:** Nhà làm phim cần lồng tiếng bộ phim tài liệu sang tiếng Pháp, Tây Ban Nha, Nhật Bản, Đức... mà giữ nguyên chất giọng truyền cảm của người thuyết minh gốc.
- **Đầu vào / Đầu ra:** Track tiếng Việt gốc $\to$ 10 track âm thanh 10 thứ tiếng đã khớp khẩu hình môi.
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --audio narrative_vi.wav --target-langs "en,fr,es,de,ja,ko,zh,ru,ar,pt" --output ./dubbed/ --opcode 0x0227 --bounty 15000
  ```
- **Kinh tế:** Tốn **15,000 nanoSKN**, xong trong 2 phút 30 giây.

---

### Case 039: Ước Tính Bản Đồ Độ Sâu (Depth Map) Cho 500 Ảnh Phục Vụ Màn Hình 3D Hologram
- **Bối cảnh:** Triển lãm nghệ thuật số cần chuyển thể 500 bức tranh hội họa 2D thành hiệu ứng chiều sâu 3D thị sai Parallax.
- **Đầu vào / Đầu ra:** Thư mục 500 ảnh hội họa $\to$ Thư mục 500 bản đồ độ sâu 16-bit Grayscale tương ứng (Depth Anything v2).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./paintings/ --output ./depth_maps/ --opcode 0x0228 --model "Depth-Anything-V2-Large" --bounty-per-item 10 --parallel 8
  ```
- **Kinh tế:** Tốn **5,000 nanoSKN**, hoàn tất trong 1 phút 40 giây.

---

### Case 040: Nén Hàng Loạt 1,000 Video TikTok Sang Chuẩn HEVC/H.265 Tiết Kiệm Băng Thông
- **Bối cảnh:** Mạng xã hội video ngắn cần nén hàng loạt 1,000 video người dùng tải lên để giảm tải chi phí lưu trữ CDN.
- **Đầu vào / Đầu ra:** 1,000 video MP4 kích thước 50MB/video (tổng 50GB) $\to$ 1,000 video HEVC kích thước 12MB/video (tổng 12GB).
- **Lệnh thực thi:**
  ```bash
  skynet compute dispatch --input ./user_uploads/ --output ./optimized_hevc/ --opcode 0x0229 --codec hevc_nvenc --crf 23 --bounty-per-item 5 --parallel 20
  ```
- **Kinh tế:** Tốn **5,000 nanoSKN**, 20 máy thợ nén xong toàn bộ 1,000 video trong **7 phút**.

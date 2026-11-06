# Phân Khu 10: Truyền Thông & Mã Hóa Đa Phương Tiện (60 Opcodes)

Phân khu này định nghĩa 60 mã tác vụ chuyên biệt phục vụ các Đài truyền hình, Hãng sản xuất phim, Nền tảng phát trực tuyến (Streaming Platforms), Phòng thu âm thanh và Nhà sáng tạo nội dung số: từ Chuyển đổi mã hóa video thế hệ mới (SVT-AV1, Apple ProRes, VVC H.266), Đánh giá chất lượng video cảm thụ mắt người (Netflix VMAF), Tách cảnh tự động, Mastering âm thanh chuẩn truyền hình EBU R128 cho đến Khử rung video và Xử lý phụ đề song ngữ.

---

## Danh Mục Chi Tiết 60 Opcodes Truyền Thông & Media Codecs (`0xA001` – `0xA03C`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0xA001` | `skynet://media.encode.svt_av1_10bit_4k` | Mã hóa video chuẩn mở AV1 10-bit chất lượng cao tiết kiệm 30% băng thông| 16 GB RAM | `capsule-svt-av1` |
| `0xA002` | `skynet://media.encode.apple_prores_4444_xq`| Transcode video quay máy quay điện ảnh sang Apple ProRes 4444 XQ master| 32 GB UMA | `capsule-apple-prores` |
| `0xA003` | `skynet://media.encode.vvc_h266_intra_frame`| Thử nghiệm nén video thế hệ mới nhất VVC (H.266) nén sâu gấp đôi HEVC | 32 GB RAM | `capsule-vvenc-h266` |
| `0xA004` | `skynet://media.encode.hevc_nvenc_realtime` | Nén video HEVC 4K 60fps phần cứng card đồ họa NVIDIA NVENC siêu trễ thấp| 6 GB VRAM | `capsule-nvenc-hevc` |
| `0xA005` | `skynet://media.eval.netflix_vmaf_score` | Đo lường điểm số chất lượng cảm thụ hình ảnh mắt người Netflix VMAF | 16 GB RAM | `capsule-vmaf-metric` |
| `0xA006` | `skynet://media.eval.psnr_ssim_batch` | Tính chỉ số độ sai lệch tín hiệu PSNR và tương đồng cấu trúc SSIM | 8 GB RAM | `capsule-psnr-ssim` |
| `0xA007` | `skynet://media.scene.shot_boundary_detect` | Tự động phát hiện các điểm chuyển cảnh và góc máy quay trong bộ phim | 8 GB RAM | `capsule-pyscenedetect`|
| `0xA008` | `skynet://media.hdr.tone_mapping_reinhard` | Chuyển đổi dải màu tương phản cao HDR10 sang SDR bằng thuật toán Reinhard| 8 GB RAM | `capsule-hdr-tonemap` |
| `0xA009` | `skynet://media.hdr.dolby_vision_metadata` | Trích xuất và tiêm siêu dữ liệu động RPU tương thích chuẩn Dolby Vision | 8 GB RAM | `capsule-dovi-tools` |
| `0xA00A` | `skynet://media.stream.hls_dash_packaging` | Đóng gói phân đoạn video tương thích luồng phát trực tuyến HLS và MPEG-DASH| 8 GB RAM | `capsule-shaka-pack` |
| `0xA00B` | `skynet://media.audio.mastering_ebu_r128` | Chuẩn hóa âm lượng tự động -23 LUFS theo tiêu chuẩn truyền hình Châu Âu | 2 GB RAM | `capsule-loudnorm-r128`|
| `0xA00C` | `skynet://media.audio.surround_51_downmix` | Giảm trộn âm thanh vòm 5.1 xuống Stereo 2 kênh không bị triệt tiêu pha | 2 GB RAM | `capsule-downmix-audio`|
| `0xA00D` | `skynet://media.audio.flac_lossless_archive`| Nén bảo tồn nguyên vẹn 100% chất lượng âm thanh phòng thu 192kHz/24bit | 4 GB RAM | `capsule-flac-codec` |
| `0xA00E` | `skynet://media.video.stabilizer_deshake` | Khử rung lắc video quay cầm tay bằng thuật toán ghép đặc trưng biến dạng | 8 GB RAM | `capsule-vidstab` |
| `0xA00F` | `skynet://media.video.deinterlace_yadif` | Khử răng cưa quét xen kẽ (Deinterlace) phim tài liệu cũ chuẩn truyền hình | 4 GB RAM | `capsule-yadif-deint` |
| `0xA010` | `skynet://media.video.color_lut_3d_apply` | Áp bảng màu điện ảnh 3D LUT (.cube) lên toàn bộ các shot quay log máy quay| 4 GB RAM | `capsule-lut3d-apply` |
| `0xA011` | `skynet://media.sub.ocr_vobsub_to_srt` | Bóc tách phụ đề ảnh đĩa DVD (.sub/.idx) sang định dạng văn bản .srt | 4 GB RAM | `capsule-vobsub-ocr` |
| `0xA012` | `skynet://media.sub.ass_styled_karaoke` | Render phụ đề hiệu ứng chữ bay màu Karaoke phức tạp lên khung hình video| 4 GB RAM | `capsule-libass-render`|
| `0xA013` | `skynet://media.thumb.smart_crop_subject` | Tự động cắt cúp ảnh thu nhỏ Thumbnail tập trung vào khuôn mặt nhân vật | 4 GB RAM | `capsule-smartcrop` |
| `0xA014` | `skynet://media.thumb.animated_webp_preview`| Tạo ảnh xem trước động WebP 5 giây dung lượng siêu nhẹ khi rê chuột | 4 GB RAM | `capsule-webp-anim` |
| `0xA015` | `skynet://media.clean.dead_pixel_conceal` | Tự động phát hiện và xóa điểm chết cảm biến máy ảnh (Dead pixel) | 4 GB RAM | `capsule-deadpixel-fix`|
| `0xA016` | `skynet://media.clean.flicker_timelapse_fix`| Khử hiện tượng nhấp nháy sáng tối giật cục của video tua nhanh Timelapse| 8 GB RAM | `capsule-deflicker` |
| `0xA017` | `skynet://media.clean.scratch_dust_archive` | Tẩy xóa vết xước nhựa và bụi bẩn trên phim nhựa điện ảnh 35mm được scan| 16 GB RAM | `capsule-film-cleaner` |
| `0xA018` | `skynet://media.dcp.dcinema_j2k_package` | Đóng gói gói chiếu rạp chuẩn DCI Digital Cinema Package (JPEG 2000 XYZ) | 32 GB RAM | `capsule-dcp-o-matic` |
| `0xA019` | `skynet://media.watermark.forensic_cinavia`| Nhúng thủy vân chống quay trộm trong rạp chiếu phim (Cinavia Watermark)| 8 GB RAM | `capsule-cinavia-wm` |
| `0xA01A` | `skynet://media.watermark.burnt_in_timecode`| Đóng dấu mã thời gian khung hình SMPTE Timecode phục vụ dựng phim nháp | 2 GB RAM | `capsule-timecode-burn`|
| `0xA01B` | `skynet://media.meta.exif_geotag_strip` | Xóa sạch siêu dữ liệu Exif, GPS, số seri máy ảnh bảo vệ bí mật nguồn tin| 2 GB RAM | `capsule-exiftool-strip`|
| `0xA01C` | `skynet://media.broadcast.scte35_ad_splice` | Dò tìm điểm chèn quảng cáo truyền hình cáp SCTE-35 trong luồng MPEG-TS | 4 GB RAM | `capsule-scte35-detect`|
| `0xA01D` | `skynet://media.broadcast.closed_caption_608`| Trích xuất dải phụ đề người khiếm thính CEA-608/708 từ dải quét ẩn VANC | 2 GB RAM | `capsule-ccextractor` |
| `0xA01E` | `skynet://media.color.vectorscope_gamut_check`| Kiểm tra tín hiệu video không vượt quá dải màu an toàn phát sóng truyền hình| 4 GB RAM | `capsule-broadcast-safe`|
| `0xA01F` | `skynet://media.video.matte_chroma_key_green`| Tách phông xanh Green Screen và khử viền ám xanh (Despill) diễn viên | 8 GB RAM | `capsule-chromakey-gpu`|
| `0xA020` | `skynet://media.video.alpha_channel_prores444`| Tạo video nền trong suốt cho đồ họa phát sóng bảng tỉ số bóng đá | 8 GB RAM | `capsule-alpha-video` |
| `0xA021` | `skynet://media.video.slowmo_optical_flow_rife`| Kéo chậm video 10 lần Slow-motion 240fps bằng nội suy dòng quang học AI | 12 GB VRAM | `capsule-slowmo-rife` |
| `0xA022` | `skynet://media.video.reverse_playback_smooth`| Đảo ngược video chuyển động mượt mà không bị giật khung hình I-frame | 4 GB RAM | `capsule-reverse-vid` |
| `0xA023` | `skynet://media.video.motion_blur_shutter_add`| Bổ sung hiệu ứng mờ nhòe chuyển động tự nhiên theo góc mở màn trập 180 độ| 8 GB RAM | `capsule-motion-blur` |
| `0xA024` | `skynet://media.video.telecine_inverse_ivtc` | Đảo ngược quá trình kéo dãn 3:2 Pulldown khôi phục phim 24fps gốc | 4 GB RAM | `capsule-ivtc-telecine`|
| `0xA025` | `skynet://media.video.aspect_ratio_letterbox`| Tự động thêm dải đen Letterbox/Pillarbox giữ đúng tỷ lệ khung hình 16:9 | 2 GB RAM | `capsule-letterbox-pad`|
| `0xA026` | `skynet://media.audio.ac3_dolby_digital_encode`| Mã hóa bản mix âm thanh vòm sang định dạng Dolby Digital AC-3 640kbps | 2 GB RAM | `capsule-aften-ac3` |
| `0xA027` | `skynet://media.audio.aac_he_v2_low_bitrate` | Nén nhạc phát trực tuyến di động chất lượng cao ở mức 32kbps (HE-AAC v2)| 2 GB RAM | `capsule-fdk-aac` |
| `0xA028` | `skynet://media.audio.truehd_atmos_parse` | Đọc siêu dữ liệu vị trí âm thanh 3D của luồng Dolby TrueHD Atmos | 4 GB RAM | `capsule-atmos-parser` |
| `0xA029` | `skynet://media.audio.dither_triangular_tpdf`| Áp nhiễu hạt Dither tam giác khử méo lượng tử khi hạ từ 24bit xuống 16bit| 2 GB RAM | `capsule-audio-dither` |
| `0xA02A` | `skynet://media.audio.resample_soxr_extreme` | Chuyển đổi tần số lấy mẫu âm thanh siêu chính xác băng thông phẳng SoX | 2 GB RAM | `capsule-soxr-resample`|
| `0xA02B` | `skynet://media.audio.phase_correlation_meter`| Đo hệ số tương quan pha giữa kênh trái và phải cảnh báo triệt tiêu âm Mono| 2 GB RAM | `capsule-phase-meter` |
| `0xA02C` | `skynet://media.audio.vocal_remover_center_sub`| Triệt tiêu giọng ca sĩ nằm ở chính giữa sân khấu âm thanh Stereo | 2 GB RAM | `capsule-center-cancel`|
| `0xA02D` | `skynet://media.image.jxl_lossless_archive` | Nén ảnh kho lưu trữ số JPEG XL không mất dữ liệu nhỏ hơn JPEG 35% | 4 GB RAM | `capsule-jxl-codec` |
| `0xA02E` | `skynet://media.image.avif_hdr_wide_gamut` | Nén ảnh AVIF dải màu rộng Rec.2020 10-bit cho web thế hệ mới | 4 GB RAM | `capsule-avif-encode` |
| `0xA02F` | `skynet://media.image.raw_bayer_demosaic_ahd`| Tái tạo màu ảnh RAW cảm biến máy ảnh theo thuật toán AHD nội suy | 8 GB RAM | `capsule-libraw-demos` |
| `0xA030` | `skynet://media.image.icc_color_profile_convert`| Ánh xạ màu sắc giữa các hồ sơ ICC (Adobe RGB sang sRGB) bảo toàn màu da| 4 GB RAM | `capsule-lcms2-icc` |
| `0xA031` | `skynet://media.image.tiling_deepzoom_dz` | Cắt ảnh bản đồ chụp Gigapixel thành các tầng thu phóng DeepZoom | 8 GB RAM | `capsule-deepzoom-tile`|
| `0xA032` | `skynet://media.image.exr_half_float_clamp` | Nén file đồ họa OpenEXR 16-bit float bằng thuật toán nén PIZ | 8 GB RAM | `capsule-openexr-tool` |
| `0xA033` | `skynet://media.video.looping_seamless_crossfade`| Tạo video vòng lặp vô tận mượt mà cho hình nền chuyển động | 4 GB RAM | `capsule-loop-video` |
| `0xA034` | `skynet://media.video.gif_palette_generate` | Tạo ảnh động GIF màu sắc rực rỡ không bị bết màu qua bảng màu 256 màu tối ưu| 4 GB RAM | `capsule-gif-palette` |
| `0xA035` | `skynet://media.video.variable_framerate_fix`| Đồng bộ video quay từ điện thoại về tốc độ khung hình cố định CFR 30fps | 4 GB RAM | `capsule-vfr-to-cfr` |
| `0xA036` | `skynet://media.video.audio_sync_waveform` | Tự động khớp đồng bộ hình và tiếng từ 2 thiết bị quay độc lập qua sóng âm| 8 GB RAM | `capsule-wave-sync` |
| `0xA037` | `skynet://media.video.multi_angle_multiview` | Ghép 4 góc máy quay thể thao thành 1 màn hình chia ô Quad-Split | 8 GB RAM | `capsule-multiview-grid`|
| `0xA038` | `skynet://media.stream.low_latency_cmaf_chunk`| Cắt nhỏ video thành các đoạn chunk CMAF siêu ngắn 200ms cho livestream | 8 GB RAM | `capsule-cmaf-chunk` |
| `0xA039` | `skynet://media.stream.rtmp_to_webrtc_transmux`| Chuyển gói luồng phát sóng RTMP từ OBS sang WebRTC xem trên trình duyệt | 4 GB RAM | `capsule-webrtc-relay` |
| `0xA03A` | `skynet://media.meta.mp4_faststart_moov_atom`| Đẩy siêu dữ liệu Moov Atom lên đầu file MP4 để video phát ngay lập tức | 2 GB RAM | `capsule-qt-faststart` |
| `0xA03B` | `skynet://media.meta.mkv_merge_tracks` | Ghép các luồng video, 5 đường tiếng lồng tiếng và 10 ngôn ngữ phụ đề MKV | 4 GB RAM | `capsule-mkvtoolnix` |
| `0xA03C` | `skynet://media.eval.ebu_r128_loudness_log` | Xuất đồ thị biến thiên âm lượng theo thời gian kiểm tra chuẩn kiểm duyệt đài| 2 GB RAM | `capsule-loudness-plot`|

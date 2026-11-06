# Phân Khu 01: Trí Tuệ Nhân Tạo, Thị Giác & Ngôn Ngữ (150 Opcodes)

Phân khu này định nghĩa 150 mã tác vụ chuyên biệt về Thị giác máy tính (Computer Vision), Xử lý âm thanh & tiếng nói (Audio & Speech), và Mô hình ngôn ngữ lớn (LLMs & Generative Media).

---

## 1. Thị Giác Máy Tính & Mô Hình Sinh Ảnh (Opcodes `0x1001` – `0x1032`: 50 Mã)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | VRAM Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x1001` | `skynet://vision.upscale.real_esrgan_x4` | Phóng to ảnh 4x khử nhiễu (Real-ESRGAN Plus) | 8 GB | `capsule-vision-cuda` |
| `0x1002` | `skynet://vision.upscale.real_esrgan_anime` | Phóng to tranh vẽ hoạt hình/anime 4x nét biên | 6 GB | `capsule-vision-cuda` |
| `0x1003` | `skynet://vision.generate.flux_schnell` | Sinh ảnh nghệ thuật tốc độ cao 4 bước (FLUX.1) | 12 GB | `capsule-flux-fp8` |
| `0x1004` | `skynet://vision.generate.flux_dev` | Sinh ảnh độ phân giải cao chất lượng phòng thu | 24 GB | `capsule-flux-cuda` |
| `0x1005` | `skynet://vision.generate.sdxl_turbo` | Sinh ảnh thời gian thực 1 bước (SDXL Turbo) | 8 GB | `capsule-sdxl-cuda` |
| `0x1006` | `skynet://vision.detect.yolov10_nano` | Phát hiện vật thể thời gian thực siêu nhẹ | 2 GB | `capsule-yolo-onnx` |
| `0x1007` | `skynet://vision.detect.yolov10_extra` | Phát hiện vật thể độ chính xác cao 80 lớp COCO | 6 GB | `capsule-yolo-cuda` |
| `0x1008` | `skynet://vision.segment.sam2_large` | Phân đoạn đối tượng bất kỳ theo click chuột (SAM 2)| 16 GB | `capsule-sam2-cuda` |
| `0x1009` | `skynet://vision.segment.sam2_realtime` | Phân đoạn video thời gian thực 60fps (SAM 2 Small)| 8 GB | `capsule-sam2-metal` |
| `0x100A` | `skynet://vision.depth.depth_anything_v2` | Ước lượng bản đồ độ sâu đơn sắc không gian 3D | 8 GB | `capsule-depth-cuda` |
| `0x100B` | `skynet://vision.restore.gfpgan_face` | Phục hồi khuôn mặt mờ nhòe trong ảnh cũ | 6 GB | `capsule-gfpgan-cuda` |
| `0x100C` | `skynet://vision.restore.codeformer` | Tái tạo chi tiết tóc, mắt, da bị vỡ nét | 8 GB | `capsule-codeformer` |
| `0x100D` | `skynet://vision.inpaint.lama_cleaner` | Xóa vật thể thừa, xóa watermark tự động (LaMa) | 6 GB | `capsule-lama-torch` |
| `0x100E` | `skynet://vision.color.ddcolor_photo` | Tô màu thông minh cho ảnh tư liệu lịch sử đen trắng| 8 GB | `capsule-ddcolor` |
| `0x100F` | `skynet://vision.deblur.nafnet` | Khử nhòe chuyển động do rung tay máy ảnh (NAFNet)| 10 GB | `capsule-nafnet` |
| `0x1010` | `skynet://vision.denoise.scunet` | Lọc nhiễu hạt ISO cao trong ảnh chụp thiếu sáng | 8 GB | `capsule-scunet` |
| `0x1011` | `skynet://vision.ocr.surya_multilingual` | OCR nhận diện văn bản 90 ngôn ngữ định dạng dòng| 6 GB | `capsule-surya-ocr` |
| `0x1012` | `skynet://vision.ocr.paddleocr_v4` | OCR văn bản hóa đơn bảng biểu phức tạp góc nghiêng| 4 GB | `capsule-paddleocr` |
| `0x1013` | `skynet://vision.pose.dwpose_body` | Ước lượng tư thế khung xương 26 khớp toàn thân | 6 GB | `capsule-dwpose` |
| `0x1014` | `skynet://vision.pose.rtmo_crowd` | Ước lượng chuyển động đám đông nhiều người | 8 GB | `capsule-rtmo-cuda` |
| `0x1015` | `skynet://vision.matting.bria_rmbg` | Bóc tách nền ảnh sản phẩm thương mại điện tử | 4 GB | `capsule-rmbg-onnx` |
| `0x1016` | `skynet://vision.matting.matany_alpha` | Tách chi tiết sợi tóc và vải voan trong suốt | 8 GB | `capsule-matany` |
| `0x1017` | `skynet://vision.video.rife_interpolator` | Bù khung hình video từ 24fps lên 60fps/120fps (RIFE)| 12 GB | `capsule-rife-cuda` |
| `0x1018` | `skynet://vision.video.topaz_video_upscale`| Nâng cấp video cổ độ phân giải 480p lên 4K | 16 GB | `capsule-topaz-like` |
| `0x1019` | `skynet://vision.reid.fastreid_person` | Nhận diện lại cùng một người qua nhiều camera | 8 GB | `capsule-reid-cuda` |
| `0x101A` | `skynet://vision.tracking.bytetrack` | Bám đuổi hàng trăm phương tiện giao thông trên luồng| 4 GB | `capsule-tracking` |
| `0x101B` | `skynet://vision.face.insightface_verify`| Đối chiếu sinh trắc học khuôn mặt 1:N 512-dim | 4 GB | `capsule-insightface`|
| `0x101C` | `skynet://vision.face.anonymizer_blur` | Tự động làm mờ mặt và biển số xe bảo vệ riêng tư | 2 GB | `capsule-anon-fast` |
| `0x101D` | `skynet://vision.style.controlnet_canny` | Điều khiển sinh ảnh theo biên nét phác thảo | 8 GB | `capsule-controlnet` |
| `0x101E` | `skynet://vision.style.ip_adapter_face` | Giữ nguyên nhận diện khuôn mặt nhân vật khi vẽ AI | 12 GB | `capsule-ip-adapter` |
| `0x101F` | `skynet://vision.doc.donut_vqa` | Hỏi đáp trực tiếp trên tài liệu quét không cần OCR | 8 GB | `capsule-donut-vqa` |
| `0x1020` | `skynet://vision.anomaly.patchcore_defect`| Phát hiện lỗi xước bề mặt sản phẩm công nghiệp | 8 GB | `capsule-patchcore` |
| `0x1021` | `skynet://vision.medical.skin_lesion_isic`| Phân loại vết nám u da ác tính tiêu chuẩn y tế | 6 GB | `capsule-med-skin` |
| `0x1022` | `skynet://vision.agri.crop_pest_yolo` | Nhận diện sâu bệnh trên lá lúa và cây ăn trái | 4 GB | `capsule-agri-vision`|
| `0x1023` | `skynet://vision.vectorize.vtracer_svg` | Vector hóa ảnh bitmap sang định dạng vector SVG | 2 GB | `capsule-vtracer` |
| `0x1024` | `skynet://vision.gen_video.cogvideox_5b` | Sinh video ngắn 6 giây từ câu lệnh văn bản | 24 GB | `capsule-cogvideo` |
| `0x1025` | `skynet://vision.gen_video.animatediff_v3`| Sinh hoạt hình chuyển động mượt mà từ ảnh tĩnh | 12 GB | `capsule-animatediff`|
| `0x1026` | `skynet://vision.3d_gen.triposr_mesh` | Tạo mô hình 3D textured mesh từ 1 ảnh đơn (TripoSR)| 10 GB | `capsule-triposr` |
| `0x1027` | `skynet://vision.3d_gen.trellis_sparse` | Tạo asset game 3D từ text prompt (Trellis) | 16 GB | `capsule-trellis` |
| `0x1028` | `skynet://vision.stereo.crestereo_disp` | Tính thị sai bản đồ stereo từ 2 góc camera song song| 6 GB | `capsule-crestereo` |
| `0x1029` | `skynet://vision.light.zerodce_enhance` | Tăng sáng ảnh chụp đêm không làm cháy vùng sáng | 4 GB | `capsule-zerodce` |
| `0x102A` | `skynet://vision.hdr.hdrunet_fusion` | Ghép nhiều ảnh đa phơi sáng thành dải tương phản HDR| 8 GB | `capsule-hdr-fusion`|
| `0x102B` | `skynet://vision.super.swinir_denoise` | Khử nén ảnh JPEG bị vỡ khối nặng (SwinIR) | 8 GB | `capsule-swinir` |
| `0x102C` | `skynet://vision.shadow.shadowformer_rm` | Tự động xóa bóng đổ mặt trời trên mặt đường | 8 GB | `capsule-shadow-rm` |
| `0x102D` | `skynet://vision.cloth.ootdiffusion_tryon`| Thử đồ ảo thời trang lên người mẫu (Virtual Try-On)| 16 GB | `capsule-ootdiff` |
| `0x102E` | `skynet://vision.motion.raft_optical_flow`| Đo lường dòng quang học vận tốc chuyển động từng điểm| 8 GB | `capsule-raft-flow` |
| `0x102F` | `skynet://vision.qr.qreader_damaged` | Giải mã mã QR bị rách, xước, mờ góc độ quét khó | 2 GB | `capsule-qreader` |
| `0x1030` | `skynet://vision.hand.mediapipe_mesh` | Nhận diện cử chỉ 21 khớp bàn tay thời gian thực | 2 GB | `capsule-mediapipe` |
| `0x1031` | `skynet://vision.synthetic.sim_render_rgb`| Render cảnh tổng hợp dữ liệu huấn luyện xe tự lái | 16 GB | `capsule-synth-cam` |
| `0x1032` | `skynet://vision.watermark.imwatermark_enc`| Đóng dấu bản quyền số ẩn vô hình vào ảnh (DWT-DCT)| 2 GB | `capsule-watermark` |

---

## 2. Âm Thanh, Tiếng Nói & Xử Lý Tín Hiệu (Opcodes `0x1101` – `0x1132`: 50 Mã)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | VRAM / RAM | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x1101` | `skynet://audio.transcribe.whisper_large_v3`| Chép lời đa ngữ độ chính xác cao nhất (Whisper v3)| 10 GB | `capsule-whisper-v3`|
| `0x1102` | `skynet://audio.transcribe.whisper_distil` | Chép lời siêu nhanh gấp 6 lần cho tóm tắt cuộc họp | 4 GB | `capsule-whisper-fast`|
| `0x1103` | `skynet://audio.stem.demucs_4stems` | Bóc tách Vocal, Drum, Bass, Other từ bài hát WAV | 8 GB | `capsule-demucs-v4` |
| `0x1104` | `skynet://audio.stem.demucs_6stems` | Tách riêng cả tiếng Guitar và Piano trong bản hòa âm| 12 GB | `capsule-demucs-6s` |
| `0x1105` | `skynet://audio.denoise.deepfilternet3` | Lọc sạch tiếng quạt, còi xe, tiếng gió lồng micro | 2 GB | `capsule-deepfilter`|
| `0x1106` | `skynet://audio.voice_clone.cosyvoice` | Nhân bản giọng nói Zero-shot từ đoạn mẫu 3 giây | 12 GB | `capsule-cosyvoice` |
| `0x1107` | `skynet://audio.tts.f5_tts_expressive` | Chuyển văn bản thành giọng đọc truyền cảm xúc tự nhiên| 8 GB | `capsule-f5-tts` |
| `0x1108` | `skynet://audio.diarization.pyannote3` | Phân đoạn & nhận diện ai đang nói gì trong cuộc họp | 4 GB | `capsule-pyannote` |
| `0x1109` | `skynet://audio.vad.silero_realtime` | Phát hiện giọng người có mặt trong luồng micro (<5ms)| 1 GB | `capsule-silero-vad`|
| `0x110A` | `skynet://audio.enhance.voicefixer` | Phục hồi âm thanh băng thông hẹp cổ thành chuẩn 48kHz| 6 GB | `capsule-voicefixer`|
| `0x110B` | `skynet://audio.codec.encodec_24khz` | Nén âm thanh thần kinh cực thấp bitrate (EnCodec) | 4 GB | `capsule-encodec` |
| `0x110C` | `skynet://audio.event.audiotransformer_ast`| Nhận diện 527 loại âm thanh (tiếng kính vỡ, tiếng súng)| 4 GB | `capsule-ast-event` |
| `0x110D` | `skynet://audio.music_gen.musicgen_melody`| Soạn nhạc nền không lời theo giai điệu huýt sáo mẫu | 16 GB | `capsule-musicgen` |
| `0x110E` | `skynet://audio.music_gen.audioldm2` | Tạo hiệu ứng âm thanh kỹ xảo phim từ mô tả văn bản | 12 GB | `capsule-audioldm2` |
| `0x110F` | `skynet://audio.align.gentle_phoneme` | Gióng hàng từng âm vị với sóng âm để làm phụ đề hát | 2 GB | `capsule-gentle` |
| `0x1110` | `skynet://audio.binaural.spatial_3d` | Giả lập âm thanh vòm 360 độ theo vị trí đầu người | 2 GB | `capsule-spatial-3d`|
| `0x1111` | `skynet://audio.tempo.soundstretch` | Thay đổi tốc độ nói và nâng hạ tông không méo tiếng | 1 GB | `capsule-soundtouch`|
| `0x1112` | `skynet://audio.pitch.crepe_fundamental` | Dò cao độ nốt nhạc chính xác từng Hertz (CREPE) | 4 GB | `capsule-crepe` |
| `0x1113` | `skynet://audio.mastering.matchering` | Tự động cân bằng âm lượng và EQ theo bản nhạc tham chiếu| 2 GB | `capsule-matchering`|
| `0x1114` | `skynet://audio.fingerprint.dejavu` | Nhận diện bài hát qua đoạn thu ngắn (Audio Fingerprint)| 2 GB | `capsule-dejavu` |
| `0x1115` | `skynet://audio.karaoke.cdg_generator` | Tạo file phụ đề nhảy chữ đổi màu Karaoke CDG | 1 GB | `capsule-karaoke-gen`|
| `0x1116` | `skynet://audio.noise.spectral_subtraction`| Khử tiếng hú rít micro thời gian thực trong hội trường| 1 GB | `capsule-dsp-denoise`|
| `0x1117` | `skynet://audio.beat.madmom_downbeat` | Định vị chính xác nhịp phách mạnh/nhẹ trong bài nhạc| 2 GB | `capsule-madmom` |
| `0x1118` | `skynet://audio.convert.dts_to_flac` | Chuyển đổi định dạng đa kênh DTS/AC3 sang FLAC | 2 GB | `capsule-ffmpeg-audio`|
| `0x1119` | `skynet://audio.tts.vietnamese_vivos` | Đọc tiếng Việt chuẩn giọng 3 miền Bắc - Trung - Nam | 4 GB | `capsule-vivos-tts` |
| `0x111A` | `skynet://audio.transcribe.mms_african` | Chép lời 1.000 ngôn ngữ thổ dân hiếm (Meta MMS) | 8 GB | `capsule-meta-mms` |
| `0x111B` | `skynet://audio.equalizer.autoeq_headphone`| Cân chỉnh đáp tuyến tai nghe theo đồ thị Harman Target| 1 GB | `capsule-autoeq` |
| `0x111C` | `skynet://audio.repair.spectral_repair` | Xóa tiếng ho, tiếng lật giấy trong bản ghi âm podcast| 4 GB | `capsule-spec-repair`|
| `0x111D` | `skynet://audio.sfx.impact_synth` | Tổng hợp tiếng va chạm kim loại vật lý cho game | 2 GB | `capsule-sfx-synth` |
| `0x111E` | `skynet://audio.acoustic.rt60_reverb_time`| Đo thời gian vang âm RT60 của phòng hòa nhạc | 1 GB | `capsule-room-acoust`|
| `0x111F` | `skynet://audio.codec.opus_lowdelay` | Nén thoại thời gian thực trễ cực thấp cho bộ đàm | 1 GB | `capsule-opus-codec`|
| `0x1120` | `skynet://audio.vocal.autotune_scale` | Chỉnh phô giọng hát tự động theo thang âm cố định | 2 GB | `capsule-autotune` |
| `0x1121` | `skynet://audio.speech.emotion_detector` | Đánh giá mức độ hài lòng/giận dữ của khách qua hotline| 4 GB | `capsule-speech-emo`|
| `0x1122` | `skynet://audio.dsp.convolution_reverb` | Tái tạo âm vang đại giáo đường qua Impulse Response | 2 GB | `capsule-conv-reverb`|
| `0x1123` | `skynet://audio.analysis.mfcc_extractor` | Trích xuất 39 hệ số phổ MFCC phục vụ huấn luyện máy | 1 GB | `capsule-mfcc-lib` |
| `0x1124` | `skynet://audio.loudness.ebu_r128_norm` | Chuẩn hóa âm lượng LUFS truyền hình phát thanh chuẩn | 1 GB | `capsule-r128-norm` |
| `0x1125` | `skynet://audio.noise.anc_coefficients` | Tính hệ số chống ồn chủ động thích nghi LMS | 1 GB | `capsule-anc-dsp` |
| `0x1126` | `skynet://audio.speech.stutter_removal` | Loại bỏ các từ ngập ngừng ừ, à, lặp từ tự động | 4 GB | `capsule-disfluency`|
| `0x1127` | `skynet://audio.hearing.audiogram_dsp` | Bù tần số trợ thính cá nhân hóa cho người khiếm thính| 1 GB | `capsule-audiogram` |
| `0x1128` | `skynet://audio.watermark.audioseal_detect`| Phát hiện giọng nói deepfake nhân tạo có đóng dấu số| 6 GB | `capsule-audioseal` |
| `0x1129` | `skynet://audio.transcribe.meeting_minutes`| Tóm tắt biên bản họp kèm phân vai hành động cụ thể | 8 GB | `capsule-meeting-llm`|
| `0x112A` | `skynet://audio.radio.am_demodulation` | Giải điều chế tín hiệu sóng vô tuyến AM/FM/SSB | 2 GB | `capsule-sdr-audio` |
| `0x112B` | `skynet://audio.bio.heart_sound_murmur` | Phân tích tiếng thổi tim phonocardiogram chẩn đoán | 2 GB | `capsule-pcg-cardio` |
| `0x112C` | `skynet://audio.bio.birdsong_id` | Định danh 1.500 loài chim hoang dã qua tiếng hót | 4 GB | `capsule-birdnet` |
| `0x112D` | `skynet://audio.synth.wavetable_generator`| Sinh sóng âm wavetable cho đàn synthesizer | 1 GB | `capsule-wavetable` |
| `0x112E` | `skynet://audio.spatial.ambisonics_bformat`| Chuyển đổi định dạng Ambisonics bậc 3 sang Stereo | 2 GB | `capsule-ambisonics`|
| `0x112F` | `skynet://audio.voice.gender_age_predict` | Dự đoán độ tuổi và giới tính từ mẫu giọng nói | 2 GB | `capsule-voice-demo`|
| `0x1130` | `skynet://audio.podcast.auto_ducking` | Tự động hạ nhỏ nhạc nền khi có người bắt đầu nói | 1 GB | `capsule-ducking` |
| `0x1131` | `skynet://audio.dsp.crossover_filter` | Chia dải tần số 3 đường tiếng cho loa siêu trầm | 1 GB | `capsule-crossover` |
| `0x1132` | `skynet://audio.synthetic.speech_aug` | Tạo biến thể âm thanh (thêm ồn, méo tiếng) tăng data| 2 GB | `capsule-audio-aug` |

---

## 3. Mô Hình Ngôn Ngữ Lớn & NLP (Opcodes `0x1201` – `0x1232`: 50 Mã)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | VRAM / RAM | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x1201` | `skynet://llm.generate.llama3_8b_instruct`| Suy luận phản hồi văn bản mô hình Llama-3.1 8B | 8 GB | `capsule-llama-8b` |
| `0x1202` | `skynet://llm.generate.llama3_70b_fp8` | Suy luận sâu mô hình Llama-3.3 70B độ chính xác FP8 | 48 GB | `capsule-llama-70b`|
| `0x1203` | `skynet://llm.generate.qwen25_72b_coder` | Lập trình phần mềm, viết unit test phức tạp | 48 GB | `capsule-qwen-coder`|
| `0x1204` | `skynet://llm.generate.deepseek_v2_lite` | Mô hình Mixture-of-Experts MoE suy luận toán học | 24 GB | `capsule-deepseek` |
| `0x1205` | `skynet://llm.finetune.lora_8b` | Huấn luyện tinh chỉnh adapter LoRA cho model 8B | 16 GB | `capsule-lora-8b` |
| `0x1206` | `skynet://llm.finetune.lora_70b_deepspeed`| Huấn luyện song song đa card LoRA 70B DeepSpeed | 96 GB | `capsule-lora-70b` |
| `0x1207` | `skynet://llm.quantize.gguf_q4km` | Nén trọng số mô hình PyTorch thành định dạng GGUF | 32 GB RAM | `capsule-quantize` |
| `0x1208` | `skynet://llm.quantize.awq_4bit` | Lượng tử hóa mô hình theo kênh kích hoạt AWQ | 16 GB | `capsule-awq` |
| `0x1209` | `skynet://llm.embed.bge_m3_multilingual` | Sinh vector ngữ nghĩa 1024 chiều đa ngôn ngữ BGE-M3| 4 GB | `capsule-bge-embed` |
| `0x120A` | `skynet://llm.embed.colbert_v2` | Trích xuất biểu diễn token đa vector muilti-vector | 8 GB | `capsule-colbert` |
| `0x120B` | `skynet://llm.rerank.bge_reranker_large` | Xếp hạng lại độ phù hợp của 100 đoạn văn bản tìm kiếm| 8 GB | `capsule-reranker` |
| `0x120C` | `skynet://llm.summarize.long_context_128k`| Tóm tắt hồ sơ vụ án 200 trang ngữ cảnh cực dài | 24 GB | `capsule-long-ctx` |
| `0x120D` | `skynet://llm.agent.python_sandbox_exec` | Môi trường sandbox chạy mã Python an toàn cho Agent| 2 GB RAM | `capsule-py-sandbox`|
| `0x120E` | `skynet://llm.vlm.qwen2_vl_7b` | VLM giải thích sơ đồ kiến trúc, biểu đồ kỹ thuật | 16 GB | `capsule-qwen-vl` |
| `0x120F` | `skynet://llm.vlm.minicpm_v2_fast` | Nhận diện biển báo giao thông trên kính thông minh | 6 GB | `capsule-minicpm-v` |
| `0x1210` | `skynet://llm.merge.slerp_weights` | Hợp nhất trọng số 2 model bằng phép xoay cầu SLERP | 64 GB RAM | `capsule-model-merge`|
| `0x1211` | `skynet://llm.reward.armorm_safety_score` | Đánh giá tính an toàn, phát hiện prompt injection | 8 GB | `capsule-armorm` |
| `0x1212` | `skynet://llm.dpo.alignment_step` | Căn chỉnh mô hình theo phản hồi sở thích con người | 32 GB | `capsule-dpo-align` |
| `0x1213` | `skynet://llm.eval.helm_benchmark` | Đánh giá toàn diện năng lực suy luận của mô hình mới| 24 GB | `capsule-helm-eval` |
| `0x1214` | `skynet://llm.translate.nllb_200_3b` | Dịch thuật chuẩn xác giữa 200 thứ tiếng (Meta NLLB)| 12 GB | `capsule-nllb-trans`|
| `0x1215` | `skynet://llm.grammar.languagetool_fast` | Soát lỗi chính tả và ngữ pháp tiếng Anh học thuật | 2 GB RAM | `capsule-grammar` |
| `0x1216` | `skynet://llm.ner.spacy_trf_entities` | Trích xuất tên người, địa danh, tổ chức trong văn bản| 4 GB | `capsule-spacy-trf` |
| `0x1217` | `skynet://llm.extract.table_to_markdown` | Chuyển đổi bảng dữ liệu phi cấu trúc sang Markdown | 4 GB | `capsule-table-llm` |
| `0x1218` | `skynet://llm.sentiment.finbert_market` | Phân tích tâm lý thị trường chứng khoán từ tin tức | 4 GB | `capsule-finbert` |
| `0x1219` | `skynet://llm.sql.defog_sqlcoder_7b` | Viết câu lệnh SQL truy vấn database từ câu hỏi thường| 14 GB | `capsule-sqlcoder` |
| `0x121A` | `skynet://llm.json.instructor_structured`| Ràng buộc mô hình trả về đúng 100% JSON Schema | 8 GB | `capsule-instructor`|
| `0x121B` | `skynet://llm.synthetic.ultrafeedback_gen`| Sinh dữ liệu đối thoại nhân tạo chất lượng cao | 24 GB | `capsule-synth-text`|
| `0x121C` | `skynet://llm.redact.pii_presidio` | Tự động phát hiện và bôi đen số CCCD, tài khoản ngân hàng| 2 GB | `capsule-presidio` |
| `0x121D` | `skynet://llm.rag.raptor_tree_index` | Xây dựng cây đệ quy tóm tắt tài liệu RAPTOR cho RAG | 16 GB RAM | `capsule-raptor-rag`|
| `0x121E` | `skynet://llm.legal.counsel_analyzer` | Đối soát điều khoản loại trừ trong hợp đồng bảo hiểm| 12 GB | `capsule-legal-llm` |
| `0x121F` | `skynet://llm.classify.setfit_fewshot` | Phân loại văn bản chỉ cần 8 mẫu học tập (SetFit) | 4 GB | `capsule-setfit` |
| `0x1220` | `skynet://llm.prompt.dsp_optimizer` | Tự động viết lại tối ưu hóa câu lệnh Prompt (DSPy) | 8 GB | `capsule-dspy` |
| `0x1221` | `skynet://llm.tokenize.byte_pair_train` | Huấn luyện từ điển Tokenizer mới cho ngôn ngữ địa phương| 8 GB RAM | `capsule-tokenizer` |
| `0x1222` | `skynet://llm.math.deepseek_math_7b` | Giải các bài toán phương trình vi phân và tích phân | 14 GB | `capsule-math-llm` |
| `0x1223` | `skynet://llm.kg.rebel_relation_extract`| Trích xuất các bộ ba tri thức (Chủ thể-Quan hệ-Đối thể)| 6 GB | `capsule-rebel-kg` |
| `0x1224` | `skynet://llm.rewrite.polite_tone` | Viết lại email khiếu nại thành ngôn từ lịch thiệp | 4 GB | `capsule-tone-style`|
| `0x1225` | `skynet://llm.debate.multi_agent_consensus`| Dàn xếp 3 Agent tranh biện để ra quyết định đầu tư| 16 GB | `capsule-debate-swm`|
| `0x1226` | `skynet://llm.security.jailbreak_probe` | Quét phát hiện lỗ hổng an toàn prompt injection | 8 GB | `capsule-jailbreak` |
| `0x1227` | `skynet://llm.chem.molformer_smiles` | Mã hóa cấu trúc phân tử hóa học từ chuỗi SMILES | 6 GB | `capsule-molformer` |
| `0x1228` | `skynet://llm.transcribe.code_commenter`| Đọc hiểu mã nguồn C++/Rust cổ và viết tài liệu hướng dẫn| 8 GB | `capsule-code-doc` |
| `0x1229` | `skynet://llm.spec.gherkin_generator` | Sinh kịch bản kiểm thử hành vi BDD Gherkin tự động | 6 GB | `capsule-gherkin-gen`|
| `0x122A` | `skynet://llm.rag.graphrag_local_search`| Truy vấn đồ thị tri thức Microsoft GraphRAG | 16 GB RAM | `capsule-graphrag` |
| `0x122B` | `skynet://llm.medical.clinical_notes_icd10`| Gán mã bệnh án quốc tế ICD-10 từ sổ khám bệnh | 10 GB | `capsule-icd10-llm` |
| `0x122C` | `skynet://llm.distill.student_knowledge` | Chắt lọc tri thức từ mô hình lớn sang mô hình 1B siêu nhẹ| 32 GB | `capsule-distill` |
| `0x122D` | `skynet://llm.agent.tree_of_thoughts` | Thuật toán Cây suy nghĩ (Tree of Thoughts) giải đố | 12 GB | `capsule-tot-agent` |
| `0x122E` | `skynet://llm.moderation.hate_speech_vi` | Phát hiện nội dung thù ghét, xúc phạm tiếng Việt | 4 GB | `capsule-hate-vi` |
| `0x122F` | `skynet://llm.finance.earning_call_qa` | Tự động đọc báo cáo tài chính quý và trả lời cổ đông| 16 GB | `capsule-earning-qa`|
| `0x1230` | `skynet://llm.poem.vietnamese_lucbat` | Sáng tác thơ lục bát, song thất lục bát đúng luật vần| 4 GB | `capsule-poem-gen` |
| `0x1231` | `skynet://llm.privacy.membership_infer` | Kiểm tra độ rò rỉ dữ liệu huấn luyện của checkpoint | 12 GB | `capsule-mia-test` |
| `0x1232` | `skynet://llm.eval.perplexity_stride` | Đo lường độ ngạc nhiên (Perplexity) đánh giá độ trôi chảy| 8 GB | `capsule-ppl-eval` |

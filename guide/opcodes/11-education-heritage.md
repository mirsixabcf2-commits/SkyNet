# Phân Khu 11: Giáo Dục, Ngôn Ngữ & Di Sản Văn Hóa (50 Opcodes)

Phân khu này định nghĩa 50 mã tác vụ chuyên biệt phục vụ các Viện nghiên cứu Hán Nôm, Thư viện quốc gia, Bảo tàng lịch sử số, Các trường đại học và Nền tảng EdTech: từ Số hóa nhận diện văn bản cổ (Ancient Han-Nom OCR, Chữ Phạn, Chữ tượng hình Ai Cập), Phục chế màu tranh khắc gỗ dân gian, Dịch thuật chuyên ngành học thuật chính xác cao cho đến Chấm điểm bài luận và Tự động đánh giá mã nguồn lập trình sinh viên.

---

## Danh Mục Chi Tiết 50 Opcodes Giáo Dục & Di Sản Số (`0xB001` – `0xB032`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0xB001` | `skynet://edu.heritage.hannom_ocr_transformer`| Nhận diện chữ Hán Nôm cổ từ ảnh chụp mộc bản triều Nguyễn | 8 GB VRAM | `capsule-hannom-ocr` |
| `0xB002` | `skynet://edu.heritage.woodblock_restoration` | Khử nấm mốc, vết mực loang và phục chế nét khắc gỗ cổ | 8 GB VRAM | `capsule-woodblock-fix`|
| `0xB003` | `skynet://edu.heritage.ancient_sanskrit_transcribe`| Chép lại các văn tự tiếng Phạn trên bia đá cổ sang ký tự Latinh IAST | 8 GB RAM | `capsule-sanskrit-ocr` |
| `0xB004` | `skynet://edu.heritage.hieroglyph_egyptian_detect`| Nhận diện và dịch nghĩa các biểu tượng chữ tượng hình Ai Cập cổ đại | 8 GB VRAM | `capsule-hieroglyph-ai`|
| `0xB005` | `skynet://edu.heritage.dongho_folk_color_enhance`| Phục dựng màu sắc tranh dân gian Đông Hồ bằng sắc tố tự nhiên số hóa| 4 GB RAM | `capsule-dongho-paint` |
| `0xB006` | `skynet://edu.heritage.calligraphy_stroke_order`| Phân tích thứ tự các nét bút lông thư pháp và tính độ cân đối lực nhấn| 4 GB RAM | `capsule-calligraphy` |
| `0xB007` | `skynet://edu.heritage.bronze_drum_pattern_vector`| Vector hóa hoa văn chim Lạc, mặt trời trên trống đồng Đông Sơn | 4 GB RAM | `capsule-dongson-vector`|
| `0xB008` | `skynet://edu.heritage.parchment_iron_gall_ink_fix`| Khôi phục bản thảo giấy da bị mực sắt ăn mòn thủng giấy | 8 GB RAM | `capsule-parchment-fix`|
| `0xB009` | `skynet://edu.trans.academic_paper_en_vi` | Dịch thuật bài báo khoa học chuẩn thuật ngữ chuyên ngành IEEE/Nature | 12 GB VRAM | `capsule-academic-trans`|
| `0xB00A` | `skynet://edu.trans.patent_terminology_align`| Dịch hồ sơ bằng sáng chế đa quốc gia đảm bảo phạm vi bảo hộ pháp lý | 12 GB RAM | `capsule-patent-trans` |
| `0xB00B` | `skynet://edu.trans.ancient_to_modern_vietnamese`| Dịch nghĩa văn bản Hán Nôm cổ sang tiếng Việt hiện đại có chú giải | 16 GB VRAM | `capsule-hannom-to-vi` |
| `0xB00C` | `skynet://edu.trans.sign_language_video_to_text`| Nhận diện ngôn ngữ ký hiệu cử chỉ tay của người khiếm thính sang chữ| 8 GB VRAM | `capsule-sign-lang-ai` |
| `0xB00D` | `skynet://edu.trans.braille_unicode_convert` | Chuyển đổi tài liệu sách giáo khoa sang hệ thống ký tự chữ nổi Braille | 2 GB RAM | `capsule-braille-conv` |
| `0xB00E` | `skynet://edu.grade.essay_coherence_scoring` | Tự động chấm điểm bài luận tiếng Anh IELTS Task 2 theo 4 tiêu chí | 8 GB VRAM | `capsule-essay-grader` |
| `0xB00F` | `skynet://edu.grade.coding_sandbox_autograder`| Chạy chấm bài lập trình Python/C++ qua 100 test case ẩn trong sandbox | 4 GB RAM | `capsule-code-autograd`|
| `0xB010` | `skynet://edu.grade.omr_multiple_choice_sheet`| Chấm phiếu trả lời trắc nghiệm thi đại học 50 câu từ ảnh scan (<0.1s)| 2 GB RAM | `capsule-omr-grader` |
| `0xB011` | `skynet://edu.grade.math_handwritten_step_check`| Nhận diện chữ viết tay lời giải toán và kiểm tra từng bước biến đổi | 8 GB VRAM | `capsule-math-step-ai` |
| `0xB012` | `skynet://edu.math.latex_from_image_pix2tex` | Chuyển ảnh chụp công thức toán học phức tạp thành mã nguồn LaTeX | 4 GB VRAM | `capsule-pix2tex` |
| `0xB013` | `skynet://edu.math.symbolic_integration_cas` | Tính toán giải tích tích phân từng phần biểu tượng phức tạp bằng CAS | 4 GB RAM | `capsule-sympy-cas` |
| `0xB014` | `skynet://edu.math.geometry_proof_deduction` | Tự động sinh lời giải chứng minh hình học phẳng bằng phương pháp Wu | 4 GB RAM | `capsule-geom-prover` |
| `0xB015` | `skynet://edu.learn.spaced_repetition_sm2` | Tính toán chu kỳ giãn cách ôn tập thẻ nhớ Anki tối ưu theo thuật toán SM-2| 2 GB RAM | `capsule-supermemo-sm2`|
| `0xB016` | `skynet://edu.learn.knowledge_trace_dkt` | Mô hình hóa độ thành thạo kiến thức học sinh qua lịch sử làm bài (DKT) | 8 GB RAM | `capsule-deep-kt` |
| `0xB017` | `skynet://edu.learn.adaptive_quiz_cat` | Sinh câu hỏi trắc nghiệm thích ứng theo thuyết ứng đáp câu hỏi (IRT) | 4 GB RAM | `capsule-cat-quiz` |
| `0xB018` | `skynet://edu.learn.dyslexia_reading_aid` | Điều chỉnh khoảng cách chữ và phông chữ hỗ trợ trẻ mắc hội chứng khó đọc| 2 GB RAM | `capsule-dyslexia-aid` |
| `0xB019` | `skynet://edu.plag.cross_language_plagiarism`| Quét phát hiện đạo văn dịch thuật (ví dụ dịch từ bài tiếng Anh sang tiếng Việt)| 16 GB RAM | `capsule-cross-plag` |
| `0xB01A` | `skynet://edu.plag.source_code_moss_similarity`| So sánh cây cú pháp trừu tượng AST phát hiện sinh viên chép code của nhau| 4 GB RAM | `capsule-moss-plag` |
| `0xB01B` | `skynet://edu.course.mindmap_generator_nlp`| Tự động tóm tắt chương sách thành sơ đồ tư duy dạng cây đồ thị SVG | 4 GB RAM | `capsule-mindmap-gen` |
| `0xB01C` | `skynet://edu.course.flashcard_qa_extract` | Trích xuất 100 câu hỏi - đáp nhanh từ tệp slide bài giảng PDF | 8 GB RAM | `capsule-flashcard-ai` |
| `0xB01D` | `skynet://edu.voice.pronunciation_scoring_ipa`| Đánh giá độ chuẩn xác phát âm từng âm vị tiếng Anh so với bảng phiên âm IPA| 4 GB RAM | `capsule-ipa-pronounce`|
| `0xB01E` | `skynet://edu.voice.accent_conversion_neutral`| Chuyển đổi chất giọng địa phương nặng sang giọng Anh-Mỹ chuẩn mực | 8 GB VRAM | `capsule-accent-neutral`|
| `0xB01F` | `skynet://edu.music.sheet_music_omr_xml` | Nhận diện bản ký âm nốt nhạc quét từ sách in sang định dạng MusicXML | 4 GB RAM | `capsule-music-omr` |
| `0xB020` | `skynet://edu.music.ear_training_solfege` | Tự động chấm bài thi xướng âm và ký âm của học viên nhạc viện | 4 GB RAM | `capsule-ear-training` |
| `0xB021` | `skynet://edu.history.timeline_event_align` | Gióng hàng và liên kết các mốc sự kiện lịch sử từ nhiều nguồn tài liệu | 8 GB RAM | `capsule-timeline-hist`|
| `0xB022` | `skynet://edu.history.genealogy_tree_builder`| Tự động dựng cây phả hệ dòng họ từ sách gia phả chữ Hán cổ | 8 GB RAM | `capsule-genealogy-tree`|
| `0xB023` | `skynet://edu.archaeo.pottery_3d_fragment_puzzle`| Tự động ghép nối các mảnh gốm vỡ khảo cổ học 3D thành bình nguyên vẹn | 16 GB RAM | `capsule-pottery-puzzle`|
| `0xB024` | `skynet://edu.archaeo.c14_radiocarbon_calib`| Hiệu chuẩn niên đại carbon phóng xạ C-14 theo đường cong chuẩn hóa IntCal20| 2 GB RAM | `capsule-c14-calibrate`|
| `0xB025` | `skynet://edu.chem.molecule_nomenclature_iupac`| Chuyển đổi cấu trúc phân tử hóa học sang tên gọi quốc tế IUPAC chuẩn | 4 GB RAM | `capsule-iupac-name` |
| `0xB026` | `skynet://edu.phys.feynman_diagram_draw` | Vẽ sơ đồ tương tác hạt cơ bản Feynman từ biểu thức ma trận tán xạ | 2 GB RAM | `capsule-feynman-diag` |
| `0xB027` | `skynet://edu.bio.cell_organelle_vr_slice` | Cắt lớp mô hình tế bào sinh học hiển thị tương tác trong kính VR | 8 GB RAM | `capsule-cell-vr-edu` |
| `0xB028` | `skynet://edu.geo.geology_rock_microscope_ocr`| Định danh khoáng vật qua ảnh chụp lát mỏng thạch học dưới kính phân cực | 6 GB VRAM | `capsule-rock-thin-sec`|
| `0xB029` | `skynet://edu.lang.ipa_phonetic_transcribe` | Phiên âm tự động văn bản tiếng Việt sang ký hiệu ngữ âm quốc tế IPA | 2 GB RAM | `capsule-vi-ipa-trans` |
| `0xB02A` | `skynet://edu.lang.tone_sandhi_taiwanese` | Phân tích quy luật biến điệu thanh điệu trong ngôn ngữ địa phương | 2 GB RAM | `capsule-tone-sandhi` |
| `0xB02B` | `skynet://edu.lit.rhyme_meter_analyzer` | Phân tích nhịp điệu bằng trắc và gieo vần trong thơ cổ điển | 2 GB RAM | `capsule-rhyme-analyzer`|
| `0xB02C` | `skynet://edu.lib.dewey_decimal_classify` | Tự động phân loại sách thư viện theo khung phân loại thập phân Dewey | 4 GB RAM | `capsule-dewey-library`|
| `0xB02D` | `skynet://edu.lib.marc21_bibliographic_record`| Tạo biểu ghi thư mục theo tiêu chuẩn quốc tế MARC21 từ trang bìa sách | 4 GB RAM | `capsule-marc21-record`|
| `0xB02E` | `skynet://edu.preserv.newspaper_microfilm_clean`| Khử ố vàng và vết nhăn trên phim vi nhỏ (Microfilm) báo chí xuất bản 100 năm| 8 GB RAM | `capsule-microfilm-fix`|
| `0xB02F` | `skynet://edu.preserv.audio_tape_wow_flutter_fix`| Khử hiện tượng méo tiếng do dão băng cassette tư liệu lịch sử | 4 GB RAM | `capsule-tape-flutter` |
| `0xB030` | `skynet://edu.sim.optics_lab_virtual_bench`| Phòng thí nghiệm ảo mô phỏng khúc xạ ánh sáng qua lăng kính cho học sinh | 4 GB RAM | `capsule-virtual-lab` |
| `0xB031` | `skynet://edu.sim.orbital_gravity_gravity_toy`| Mô phỏng lực hấp dẫn tương tác các hành tinh trong chương trình vật lý | 4 GB RAM | `capsule-gravity-edu` |
| `0xB032` | `skynet://edu.game.educational_trivia_gen` | Tự động sinh bộ câu hỏi đố vui kiến thức tổng hợp có kiểm chứng nguồn | 4 GB RAM | `capsule-trivia-edu` |

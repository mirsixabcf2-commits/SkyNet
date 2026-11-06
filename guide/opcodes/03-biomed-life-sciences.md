# Phân Khu 03: Khoa Học Sự Sống, Y Sinh & Dược Phẩm (100 Opcodes)

Phân khu này định nghĩa 100 mã tác vụ chuyên biệt phục vụ các Viện nghiên cứu sinh học, Trường Y, Công ty dược phẩm và Bệnh viện: từ Dự đoán gấp nếp cấu trúc protein (AlphaFold), Sàng lọc dược chất ảo (Virtual Screening / Molecular Docking), Động lực học phân tử (Molecular Dynamics) cho đến Phân tích giải trình tự gen (Genomics) và Ảnh cắt lớp y tế (Medical Imaging).

---

## Danh Mục Chi Tiết 100 Opcodes Y Sinh & Dược Phẩm (`0x3001` – `0x3064`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x3001` | `skynet://bio.folding.alphafold3_multimer` | Dự đoán cấu trúc phức hợp Protein-Kháng thể-DNA 3D (AF3) | 32 GB VRAM | `capsule-alphafold3` |
| `0x3002` | `skynet://bio.folding.esmfold_v1_fast` | Dự đoán nhanh gấp nếp chuỗi đơn peptide bằng ESMFold | 16 GB VRAM | `capsule-esmfold` |
| `0x3003` | `skynet://bio.folding.rosettafold_all_atom`| Dự đoán cấu trúc nguyên tử toàn diện kết hợp kim loại/ligand | 24 GB VRAM | `capsule-rosettafold` |
| `0x3004` | `skynet://bio.docking.autodock_vina_batch` | Docking phối tử ảo sàng lọc 100.000 phân tử ức chế thụ thể | 32 Core CPU | `capsule-vina-simd` |
| `0x3005` | `skynet://bio.docking.gnina_deep_learning` | Dự đoán ái lực liên kết thuốc bằng mạng nơ-ron tích chập 3D | 12 GB VRAM | `capsule-gnina-cuda` |
| `0x3006` | `skynet://bio.docking.diffdock_diffusion` | Dự đoán tư thế gắn kết phối tử bằng mô hình khuếch tán | 16 GB VRAM | `capsule-diffdock` |
| `0x3007` | `skynet://bio.md.gromacs_membrane_100ns` | Mô phỏng động lực học phân tử màng lipid 100ns (GROMACS) | 16 GB VRAM | `capsule-gromacs-gpu` |
| `0x3008` | `skynet://bio.md.amber_free_energy_fep` | Tính toán biến thiên năng lượng tự do liên kết thuốc FEP | 24 GB VRAM | `capsule-amber-pmemd` |
| `0x3009` | `skynet://bio.md.lammps_nanoparticle` | Mô phỏng tương tác hạt nano dẫn thuốc trong huyết tương | 64 Core CPU | `capsule-lammps-mpi` |
| `0x300A` | `skynet://bio.cryoem.relion_3d_refine` | Tái tạo bản đồ mật độ điện tử nguyên tử 3D từ ảnh Cryo-EM | 32 GB VRAM | `capsule-relion-gpu` |
| `0x300B` | `skynet://bio.cryoem.cryosparc_ab_initio` | Dựng mô hình không gian protein nguyên thủy từ hàng nghìn hạt | 24 GB VRAM | `capsule-cryosparc` |
| `0x300C` | `skynet://bio.seq.bwa_mem_human_genome` | Gióng hàng chuỗi đọc DNA thế hệ mới (NGS) lên hệ gen người | 64 GB RAM | `capsule-bwa-mem2` |
| `0x300D` | `skynet://bio.seq.gatk_variant_calling` | Phát hiện biến thể đột biến gen SNPs và Indels gây ung thư | 64 GB RAM | `capsule-gatk4-spark` |
| `0x300E` | `skynet://bio.rna.scanpy_single_cell_100k`| Phân cụm và vẽ biểu đồ UMAP 100.000 tế bào đơn scRNA-seq | 64 GB RAM | `capsule-scanpy-rapids`|
| `0x300F` | `skynet://bio.rna.seurat_harmony_integrate`| Tích hợp nhiều mẻ dữ liệu giải trình tự đơn bào khử nhiễu | 32 GB RAM | `capsule-seurat-r` |
| `0x3010` | `skynet://bio.rna.deseq2_diff_expression` | Phân tích biểu hiện gen vi phân so sánh nhóm bệnh vs chứng | 16 GB RAM | `capsule-deseq2` |
| `0x3011` | `skynet://bio.img.nnunet_ct_organ_seg` | Phân đoạn tự động 3D tim, gan, thận trên ảnh cắt lớp CT y tế | 16 GB VRAM | `capsule-nnunet-med` |
| `0x3012` | `skynet://bio.img.brain_tumor_mri_brats` | Đo thể tích khối u não glioblastoma trên 4 chuỗi xung MRI | 12 GB VRAM | `capsule-brats-seg` |
| `0x3013` | `skynet://bio.img.dicom_mpr_reconstruct` | Tái tạo ảnh đa bình diện cắt lớp dọc/ngang từ file DICOM thô | 8 GB RAM | `capsule-dicom-mpr` |
| `0x3014` | `skynet://bio.img.pet_suv_calc` | Tính toán chỉ số hấp thu phóng xạ chuẩn hóa SUV trên ảnh PET | 8 GB RAM | `capsule-pet-suv` |
| `0x3015` | `skynet://bio.img.retina_diabetic_grade` | Tự động chấm điểm giai đoạn bệnh võng mạc tiểu đường | 6 GB VRAM | `capsule-retina-eyeq` |
| `0x3016` | `skynet://bio.img.chest_xray_chexnet_14` | Chẩn đoán 14 bệnh lý lồng ngực (tràn dịch, viêm phổi) trên X-quang | 6 GB VRAM | `capsule-chexnet` |
| `0x3017` | `skynet://bio.img.wsi_histopath_cancer` | Phân tích lam kính vi thể bệnh học Gigapixel WSI phát hiện K | 32 GB RAM | `capsule-wsi-histo` |
| `0x3018` | `skynet://bio.qsar.toxicity_tox21_pred` | Dự đoán độc tính tế bào và khả năng gây đột biến của hóa chất | 8 GB RAM | `capsule-tox21-qsar` |
| `0x3019` | `skynet://bio.chem.rdkit_similarity_search`| Tìm kiếm các phân tử tương đồng theo dấu vân tay Morgan | 16 GB RAM | `capsule-rdkit-search` |
| `0x301A` | `skynet://bio.chem.conformer_etkdg_gen` | Sinh cấu hình không gian 3D năng lượng thấp nhất cho phân tử | 8 GB RAM | `capsule-conformer-gen`|
| `0x301B` | `skynet://bio.chem.admet_pharmacokinetics`| Dự đoán hấp thu, phân bố, chuyển hóa, thải trừ (ADMET) thuốc | 8 GB RAM | `capsule-admet-ai` |
| `0x301C` | `skynet://bio.quantum.dft_orca_geometry` | Tối ưu hóa cấu trúc electron phân tử bằng hóa học lượng tử DFT| 64 Core CPU | `capsule-orca-dft` |
| `0x301D` | `skynet://bio.quantum.pyscf_hartree_fock`| Giải phương trình Hartree-Fock tự hợp năng lượng phân tử nhỏ | 32 GB RAM | `capsule-pyscf` |
| `0x301E` | `skynet://bio.protein.proteinmpnn_design`| Thiết kế chuỗi amino acid mới khớp với hình dạng cấu trúc 3D | 8 GB VRAM | `capsule-proteinmpnn` |
| `0x301F` | `skynet://bio.protein.rf_diffusion_scaffold`| Sinh de-novo cấu trúc protein khung gắn kết kháng thể mới | 16 GB VRAM | `capsule-rfdiffusion` |
| `0x3020` | `skynet://bio.protein.rmsd_superimpose` | So khớp tọa độ không gian nguyên tử và tính sai số RMSD / TM-score| 4 GB RAM | `capsule-tmscore` |
| `0x3021` | `skynet://bio.metagenome.kraken2_taxa` | Định danh vi khuẩn và virus trong mẫu nước sông từ đoạn đọc DNA | 64 GB RAM | `capsule-kraken2-db` |
| `0x3022` | `skynet://bio.metagenome.metaspades_asm` | Ráp nối hệ gen vi sinh vật quần thể phức tạp không cần mẫu chuẩn| 128 GB RAM | `capsule-metaspades` |
| `0x3023` | `skynet://bio.phylo.iqtree_phylogenetics`| Dựng cây phả hệ tiến hóa loài bằng phương pháp Maximum Likelihood| 16 GB RAM | `capsule-iqtree2` |
| `0x3024` | `skynet://bio.crispr.crispor_offtarget` | Quét phát hiện vị trí cắt nhầm ngoài mục tiêu của enzyme Cas9 | 8 GB RAM | `capsule-crispor` |
| `0x3025` | `skynet://bio.epi.bismark_dna_methylation`| Định vị các vị trí CpG bị methyl hóa điều hòa biểu hiện gen | 32 GB RAM | `capsule-bismark` |
| `0x3026` | `skynet://bio.gwas.plink_case_control` | Liên kết toàn bộ hệ gen (GWAS) tìm gen gây bệnh tiểu đường | 32 GB RAM | `capsule-plink2` |
| `0x3027` | `skynet://bio.pathway.gsea_enrichment` | Phân tích làm giàu lộ trình sinh học GSEA (KEGG, Reactome) | 8 GB RAM | `capsule-gsea-pathway` |
| `0x3028` | `skynet://bio.img.ultrasound_ef_echo` | Tính phân suất tống máu tim EF từ chuỗi video siêu âm tim | 8 GB VRAM | `capsule-echo-ef` |
| `0x3029` | `skynet://bio.img.mammography_birads` | Tầm soát tổn thương vôi hóa vi thể trên phim chụp nhũ ảnh | 8 GB VRAM | `capsule-birads-ai` |
| `0x302A` | `skynet://bio.eeg.seizure_detection_eeg` | Phát hiện sớm cơn động kinh từ 32 kênh điện não đồ EEG | 4 GB RAM | `capsule-eeg-seizure` |
| `0x302B` | `skynet://bio.ecg.arrhythmia_12lead` | Nhận diện rối loạn nhịp tim (rung nhĩ, block nhánh) từ ECG 12 cần| 2 GB RAM | `capsule-ecg-arrhythmia`|
| `0x302C` | `skynet://bio.wearable.sleep_stage_hypno`| Phân loại 5 giai đoạn giấc ngủ (N1, N2, N3, REM) từ cảm biến | 2 GB RAM | `capsule-sleep-hypno` |
| `0x302D` | `skynet://bio.blood.smear_rbc_wbc_count` | Đếm và phân loại bạch cầu, hồng cầu trên ảnh soi kính hiển vi | 6 GB VRAM | `capsule-blood-cell` |
| `0x302E` | `skynet://bio.dental.caries_bitewing_xray`| Phát hiện sâu răng kẽ và tiêu xương ổ răng trên X-quang nha khoa | 4 GB VRAM | `capsule-dental-ai` |
| `0x302F` | `skynet://bio.spine.scoliosis_cobb_angle`| Tự động đo góc vẹo cột sống Cobb Angle từ phim X-quang toàn trục | 6 GB VRAM | `capsule-cobb-angle` |
| `0x3030` | `skynet://bio.ortho.fracture_detection` | Phát hiện rạn nứt xương mắt cá và xương cổ tay khó nhìn | 6 GB VRAM | `capsule-bone-fracture`|
| `0x3031` | `skynet://bio.endo.polyp_colonoscopy_seg`| Đánh dấu thời gian thực khối polyp đại tràng trên video nội soi | 8 GB VRAM | `capsule-polyp-seg` |
| `0x3032` | `skynet://bio.pharm.dissolution_weibull`| Khớp đường cong hòa tan thuốc viên nén theo phân phối Weibull | 2 GB RAM | `capsule-dissolution` |
| `0x3033` | `skynet://bio.pharm.stability_arrhenius`| Dự đoán hạn dùng của thuốc trong điều kiện bảo quản nhiệt đới | 2 GB RAM | `capsule-arrhenius` |
| `0x3034` | `skynet://bio.bioreactor.cfd_agitation` | Mô phỏng dòng chảy khuấy trộn trong bình lên men vắc-xin 500L | 32 Core CPU | `capsule-bioreactor-cfd`|
| `0x3035` | `skynet://bio.micro.cell_colony_counting`| Đếm tự động số lượng khuẩn lạc vi khuẩn trên đĩa thạch Petri | 2 GB RAM | `capsule-petri-count` |
| `0x3036` | `skynet://bio.opt.fluorescence_deconv` | Khử nhòe tán xạ quang học kính hiển vi huỳnh quang 3D | 16 GB RAM | `capsule-deconv-micro` |
| `0x3037` | `skynet://bio.opt.spim_lightsheet_fusion`| Ghép nối các góc quay kính hiển vi tấm sáng Light-sheet | 32 GB RAM | `capsule-spim-fusion` |
| `0x3038` | `skynet://bio.opt.flow_cytometry_gate` | Tự động phân cổng dòng tế bào Flow Cytometry 12 màu | 8 GB RAM | `capsule-flow-gate` |
| `0x3039` | `skynet://bio.dna.barcoding_fish_species`| Định danh loài cá từ đoạn mã vạch DNA ty thể COI | 4 GB RAM | `capsule-dna-barcode` |
| `0x303A` | `skynet://bio.dna.primer_blast_design` | Thiết kế cặp mồi đặc hiệu PCR không bị kẹp tóc dimer | 4 GB RAM | `capsule-primer3-plus` |
| `0x303B` | `skynet://bio.rna.mirna_target_predict` | Dự đoán vị trí bám của microRNA lên vùng 3' UTR của mRNA | 8 GB RAM | `capsule-mirna-target` |
| `0x303C` | `skynet://bio.rna.splice_junction_detect`| Phát hiện đột biến vị trí cắt nối RNA Splicing gây dị tật | 16 GB RAM | `capsule-splice-ai` |
| `0x303D` | `skynet://bio.protein.epitope_bcell_pred`| Dự đoán kháng nguyên bề mặt Epitope B-cell điều chế vắc-xin | 8 GB RAM | `capsule-epitope-pred` |
| `0x303E` | `skynet://bio.protein.cleavage_protease` | Dự đoán vị trí enzyme protease cắt chuỗi peptide | 4 GB RAM | `capsule-cleavage-pred`|
| `0x303F` | `skynet://bio.protein.disorder_iupred` | Nhận diện các đoạn protein vô định hình không có cấu trúc cố định| 2 GB RAM | `capsule-iupred3` |
| `0x3040` | `skynet://bio.protein.transmembrane_helix`| Dự đoán các chuỗi xoắn alpha xuyên màng tế bào | 2 GB RAM | `capsule-tmhmm` |
| `0x3041` | `skynet://bio.lipid.lipidomics_mass_spec` | Định danh loài lipid từ phổ khối lượng MS/MS | 8 GB RAM | `capsule-lipidsearch` |
| `0x3042` | `skynet://bio.glyco.glycan_mass_profile` | Xác định thành phần đường gắn trên kháng thể đơn dòng (Glycan) | 8 GB RAM | `capsule-glyco-profile`|
| `0x3043` | `skynet://bio.metabol.metaboanalyst_pca` | Phân tích biến dưỡng chất PCA và phân biệt mẫu nước tiểu | 8 GB RAM | `capsule-metabo-pca` |
| `0x3044` | `skynet://bio.toxic.ld50_mouse_predict` | Ước lượng liều gây chết trung bình LD50 trên chuột thí nghiệm | 4 GB RAM | `capsule-ld50-predict` |
| `0x3045` | `skynet://bio.gene.crispr_guide_efficiency`| Chấm điểm hiệu quả cắt của các chuỗi RNA dẫn đường sgRNA | 4 GB RAM | `capsule-crispr-score` |
| `0x3046` | `skynet://bio.img.coronary_calcium_score`| Tính điểm vôi hóa động mạch vành Agatston phòng nhồi máu cơ tim| 8 GB RAM | `capsule-agatston-calc`|
| `0x3047` | `skynet://bio.img.liver_steatosis_mri_pdff`| Đo tỷ lệ thoái hóa mỡ gan PDFF không xâm lấn từ ảnh MRI | 8 GB RAM | `capsule-liver-pdff` |
| `0x3048` | `skynet://bio.img.lung_nodule_malignancy`| Đánh giá xác suất ác tính của nốt mờ phổi đơn độc trên phim CT | 8 GB VRAM | `capsule-lung-nodule` |
| `0x3049` | `skynet://bio.seq.hla_typing_transplant` | Định nhóm kháng nguyên bạch cầu người HLA phục vụ ghép tạng | 16 GB RAM | `capsule-hla-typing` |
| `0x304A` | `skynet://bio.seq.cfdna_liquid_biopsy` | Tầm soát sớm phân mảnh DNA khối u lưu hành trong máu (ctDNA) | 32 GB RAM | `capsule-cfdna-detect` |
| `0x304B` | `skynet://bio.seq.mitochondrial_dna_mut` | Phân tích đột biến hệ gen ty thể gây bệnh chuyển hóa di truyền | 8 GB RAM | `capsule-mtdna-mut` |
| `0x304C` | `skynet://bio.seq.telomere_length_estim` | Ước lượng độ dài đuôi Telomere đánh giá tuổi sinh học tế bào | 16 GB RAM | `capsule-telomere-calc`|
| `0x304D` | `skynet://bio.chem.reaction_yield_pred` | Dự đoán hiệu suất phản ứng tổng hợp hữu cơ hóa dược | 8 GB RAM | `capsule-reaction-yield`|
| `0x304E` | `skynet://bio.chem.retrosynthesis_path` | Hoạch định đường tổng hợp hóa học ngược từ phân tử đích | 16 GB RAM | `capsule-retro-synth` |
| `0x304F` | `skynet://bio.chem.pka_solubility_pred` | Tính hằng số ion hóa pKa và độ tan của hoạt chất trong nước | 4 GB RAM | `capsule-pka-solub` |
| `0x3050` | `skynet://bio.md.coarse_grained_martini`| Mô phỏng hạt thô Martini nghiên cứu biến dạng màng virus | 32 Core CPU | `capsule-martini-cg` |
| `0x3051` | `skynet://bio.md.umbrella_sampling_pmf` | Tính toán cấu hình thế năng lực trung bình (PMF) qua kênh ion | 16 GB VRAM | `capsule-umbrella-pmf` |
| `0x3052` | `skynet://bio.md.metadynamics_well_slice`| Mô phỏng siêu động lực học (Metadynamics) vượt rào năng lượng | 24 GB VRAM | `capsule-plumed-meta` |
| `0x3053` | `skynet://bio.img.bone_mineral_density_dxa`| Phân tích mật độ khoáng chất xương T-score từ máy quét DXA | 4 GB RAM | `capsule-dxa-bmd` |
| `0x3054` | `skynet://bio.img.knee_osteoarthritis_kl`| Phân độ thoái hóa khớp gối Kellgren-Lawrence từ phim X-quang | 4 GB VRAM | `capsule-knee-kl` |
| `0x3055` | `skynet://bio.img.fetal_ultrasound_biometry`| Đo kích thước lưỡng đỉnh và chiều dài xương đùi thai nhi | 4 GB VRAM | `capsule-fetal-biometry`|
| `0x3056` | `skynet://bio.img.prostate_pirads_mri` | Đánh giá nguy cơ ung thư tiền liệt tuyến theo chuẩn PI-RADS v2| 12 GB VRAM | `capsule-pirads-mri` |
| `0x3057` | `skynet://bio.img.cervical_pap_smear` | Tự động phát hiện tế bào biến đổi tiền ung thư cổ tử cung | 8 GB VRAM | `capsule-pap-smear` |
| `0x3058` | `skynet://bio.img.fundus_glaucoma_cdr` | Đo tỷ lệ lõm gai thị (Cup-to-Disc Ratio) tầm soát Glocom mắt | 4 GB VRAM | `capsule-glaucoma-cdr` |
| `0x3059` | `skynet://bio.implant.hip_prosthesis_stress`| Mô phỏng phân bố ứng suất khớp háng nhân tạo trong xương chậu| 32 GB RAM | `capsule-hip-implant` |
| `0x305A` | `skynet://bio.implant.stent_coronary_expand`| Mô phỏng độ bung nở đàn hồi của stent kim loại trong lòng mạch| 32 GB RAM | `capsule-stent-expand` |
| `0x305B` | `skynet://bio.cfd.blood_flow_aneurysm` | Mô phỏng lưu lượng máu và áp lực thành mạch túi phình động mạch| 64 GB RAM | `capsule-blood-cfd` |
| `0x305C` | `skynet://bio.cfd.nasal_airflow_breathe` | Tính toán trở lực dòng khí hít thở qua khoang mũi sau phẫu thuật| 16 GB RAM | `capsule-nasal-airflow`|
| `0x305D` | `skynet://bio.sensor.cgm_glucose_forecast`| Dự báo mức đường huyết trước 60 phút cho bệnh nhân tiểu đường | 2 GB RAM | `capsule-cgm-glucose` |
| `0x305E` | `skynet://bio.sensor.spo2_ppg_motion_fix`| Khử nhiễu rung lắc tay đo độ bão hòa oxy trong máu SpO2 | 2 GB RAM | `capsule-ppg-filter` |
| `0x305F` | `skynet://bio.sensor.emg_prosthetic_hand`| Giải mã tín hiệu cơ điện EMG điều khiển bàn tay giả bionic | 4 GB RAM | `capsule-emg-bionic` |
| `0x3060` | `skynet://bio.audit.hipaa_phi_anonymizer` | Tẩy xóa 18 trường thông tin định danh cá nhân y tế chuẩn HIPAA| 4 GB RAM | `capsule-hipaa-phi` |
| `0x3061` | `skynet://bio.trial.propensity_score_match`| Ghép đôi điểm xu hướng (PSM) loại trừ sai lệch thử nghiệm lâm sàng| 8 GB RAM | `capsule-psm-match` |
| `0x3062` | `skynet://bio.trial.kaplan_meier_survival` | Vẽ đường cong sống còn Kaplan-Meier và kiểm định Log-rank | 4 GB RAM | `capsule-survival-km` |
| `0x3063` | `skynet://bio.vaccine.antigenic_distance` | Đo khoảng cách kháng nguyên dự đoán hiệu quả bảo vệ vắc-xin cúm| 8 GB RAM | `capsule-antigenic-cart`|
| `0x3064` | `skynet://bio.phage.tail_fiber_specificity`| Dự đoán vi khuẩn đích của thực khuẩn thể (Phage) trị nhiễm khuẩn| 16 GB VRAM | `capsule-phage-target` |

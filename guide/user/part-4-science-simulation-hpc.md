# Tuyển tập Case 056 – 070: Mô phỏng Khoa học, Y sinh & Siêu tính toán (HPC)

Tài liệu này hướng dẫn chi tiết từ góc nhìn Người dùng (User / Nghiên cứu sinh / Kỹ sư R&D) về cách đóng gói dữ liệu, chọn thiết bị phần cứng, chạy lệnh CLI `skynet compute dispatch`, thiết lập kiểm chứng đại diện (Delegated Auditor Quorum) và nhận kết quả mô phỏng phức tạp trên mạng lưới SkyNet.

---

## Case 056: Dự đoán gấp nếp cấu trúc Protein đa chuỗi (AlphaFold3 / ESMFold)
- **Tình huống:** Nhà nghiên cứu sinh học phân tử cần dựng mô hình không gian 3D của phức hợp kháng thể - kháng nguyên gồm 4 chuỗi peptide nhưng không có cụm Nvidia A100/H100 nội bộ.
- **Dữ liệu đầu vào:** File FASTA `complex_seqs.fasta` (25 KB) chứa chuỗi amino acid và file mẫu template PDB.
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0301` (`AlphaFold3Fold`), yêu cầu tối thiểu GPU VRAM 32GB (Nvidia RTX 4090 24GB hoặc A100/H100), RAM 64GB.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0301 \
    --input ./data/complex_seqs.fasta \
    --output ./results/predicted_complex.pdb \
    --bounty 4500000000 \
    --auditor-quorum 3 \
    --timeout 7200 \
    --tags "bio,protein,alphafold,vram-32g"
  ```
- **Xác thực & Kết quả:** 3 node kiểm chứng chạy kiểm tra RMSD của backbone và độ tin cậy pLDDT > 85. User nhận file cấu trúc `.pdb` và `.cif` với độ chính xác nguyên tử.

---

## Case 057: Mô phỏng động lực học phân tử màng tế bào 100ns (GROMACS CUDA)
- **Tình huống:** Phòng thí nghiệm dược lý cần chạy mô phỏng tương tác thuốc xuyên màng lipid kép trong 100 nano-giây (10 triệu steps).
- **Dữ liệu đầu vào:** File topo `topol.top`, cấu trúc `system.gro` và thông số `md.mdp` (tổng cộng 120 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0302` (`GromacsMD100ns`), yêu cầu GPU CUDA hỗ trợ FP32 kép hoặc FP64, RAM 32GB.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0302 \
    --input ./sim_inputs/membrane_md.tar.gz \
    --output ./trajectories/membrane_100ns.xtc \
    --bounty 8000000000 \
    --auditor-quorum 2 \
    --timeout 14400 \
    --tags "cuda,gromacs,molecular-dynamics"
  ```
- **Xác thực & Kết quả:** Kiểm chứng quỹ đạo năng lượng tự do và log áp suất. User nhận file quỹ đạo `.xtc` và đồ thị RMSD/RMSF.

---

## Case 058: Khí động lực học xe điện thế hệ mới (OpenFOAM CFD Multi-Core)
- **Tình huống:** Nhóm kỹ sư ô tô startup cần tính hệ số cản khí động học $C_d$ của thân xe nguyên mẫu trên lưới 15 triệu ô (mesh cells).
- **Dữ liệu đầu vào:** Thư mục case OpenFOAM nén `car_aero_mesh.tar.gz` (450 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0303` (`OpenFoamCFD`), CPU Tier (tối thiểu 32 vCPU hoặc 64 threads, 128GB RAM).
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0303 \
    --input ./cfd/car_aero_mesh.tar.gz \
    --output ./cfd/drag_coefficient_results.tar.gz \
    --bounty 12000000000 \
    --auditor-quorum 3 \
    --timeout 18000 \
    --tags "cpu-hpc,openfoam,mesh-15m"
  ```
- **Xác thực & Kết quả:** Auditor trích xuất nghiệm hội tụ Navier-Stokes và kiểm tra sai số dư lượng (residual < 1e-5). Trả về bảng lực cản khí động học $C_d = 0.218$ và trường vận tốc gió.

---

## Case 059: Định giá quyền chọn phức tạp Monte Carlo 100M Path (Quant Finance)
- **Tình huống:** Quỹ định lượng cần định giá danh mục rổ quyền chọn Exotics và tính chỉ số Greeks thông qua 100 triệu kịch bản giả lập ngẫu nhiên Monte Carlo.
- **Dữ liệu đầu vào:** File ma trận hiệp phương sai biến động và danh mục hợp đồng `portfolio_cov.bin` (5 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0304` (`MonteCarloOptionPricing`), GPU CUDA hoặc AVX-512 CPU.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0304 \
    --input ./quant/portfolio_cov.bin \
    --output ./quant/greeks_pricing.json \
    --bounty 3500000000 \
    --auditor-quorum 3 \
    --timeout 1800 \
    --tags "quant,monte-carlo,cuda"
  ```
- **Xác thực & Kết quả:** Các auditor chạy 100k paths độc lập để kiểm chứng khoảng tin cậy 99% của giá kỳ vọng. Trả về vector Delta, Gamma, Vega, Theta hoàn chỉnh.

---

## Case 060: Dự báo vi khí hậu đô thị và lượng mưa cực đoan (WRF Weather Model)
- **Tình huống:** Trung tâm quản lý đô thị thông minh cần dự báo lượng mưa giông cực đoan cục bộ theo lưới phân giải cao 500m cho thành phố trong 24 giờ tới.
- **Dữ liệu đầu vào:** Dữ liệu biên khí tượng GFS vệ tinh `gfs_boundary_20261104.nc` (800 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0305` (`WrfWeatherSimulation`), Cụm máy chủ HPC hoặc Multi-GPU MPI.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0305 \
    --input ./weather/gfs_boundary_20261104.nc \
    --output ./weather/rain_forecast_24h.nc \
    --bounty 9500000000 \
    --auditor-quorum 2 \
    --timeout 10800 \
    --tags "wrf,weather,netcdf,mpi"
  ```
- **Xác thực & Kết quả:** Kiểm tra định luật bảo toàn khối lượng và năng lượng khí quyển. Kết quả dự báo bản đồ mưa cục bộ theo từng 15 phút.

---

## Case 061: Mô phỏng va chạm thiên hà và tương tác vật chất tối (N-Body Particle Mesh)
- **Tình huống:** Viện nghiên cứu thiên văn cần chạy mô phỏng tương tác hấp dẫn của 50 triệu hạt vật chất tối và sao trong vụ va chạm giữa hai thiên hà xoắn ốc.
- **Dữ liệu đầu vào:** File tọa độ và vận tốc ban đầu `galaxy_collision_50m.dat` (600 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0306` (`NBodyAstrophysics`), GPU CUDA năng lực tính toán FP64 mạnh mẽ.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0306 \
    --input ./astro/galaxy_collision_50m.dat \
    --output ./astro/snapshot_merged.bin \
    --bounty 7000000000 \
    --auditor-quorum 3 \
    --timeout 7200 \
    --tags "astrophysics,nbody,cuda-fp64"
  ```
- **Xác thực & Kết quả:** Kiểm chứng bảo toàn động lượng và mô-men động lượng toàn phần ($\Delta L / L < 10^{-6}$).

---

## Case 062: Phân tích độ bền ứng suất khung gầm máy bay không người lái (FEA Nastran/CalculiX)
- **Tình huống:** Startup hàng không kiểm tra ứng suất tập trung von Mises và độ mỏi vật liệu composite của thân drone dưới tải trọng gió giật 20G.
- **Dữ liệu đầu vào:** Mô hình phần tử hữu hạn lưới 3D `drone_airframe.inp` (180 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0307` (`FeaStressCalculix`), CPU đa nhân với bộ nhớ RAM kênh 8 kênh tối thiểu 64GB.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0307 \
    --input ./cad/drone_airframe.inp \
    --output ./cad/stress_displacement.frd \
    --bounty 5000000000 \
    --auditor-quorum 2 \
    --timeout 5400 \
    --tags "fea,stress,calculix,engineering"
  ```
- **Xác thực & Kết quả:** Đối chiếu cân bằng lực tại các điểm liên kết ngàm. Xuất file biến dạng và ứng suất cực đại.

---

## Case 063: Mô phỏng mạch lượng tử 40-qubit (Qiskit GPU Statevector)
- **Tình huống:** Nhóm nghiên cứu mật mã lượng tử cần mô phỏng thuật toán Grover tìm khóa bí mật trên mạch 40 qubits trước khi nạp vào máy tính lượng tử thực tế.
- **Dữ liệu đầu vào:** File mạch lượng tử QASM `grover_40qubit.qasm` (2 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0308` (`QuantumStatevector40Q`), yêu cầu hệ thống có RAM $\ge 128$GB hoặc GPU VRAM phân tán.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0308 \
    --input ./circuits/grover_40qubit.qasm \
    --output ./circuits/statevector_amplitudes.bin \
    --bounty 15000000000 \
    --auditor-quorum 2 \
    --timeout 12000 \
    --tags "quantum,statevector,40qubit,high-ram"
  ```
- **Xác thực & Kết quả:** Kiểm tra định lý uniter $\sum |c_i|^2 = 1.0$. Thu được vector phân bố xác suất của trạng thái đích.

---

## Case 064: Tầm soát thuốc ảo docking 500.000 phân tử ức chế protease (AutoDock Vina)
- **Tình huống:** Nhóm sàng lọc dược chất cần kiểm tra ái lực liên kết (binding affinity) của 500.000 hợp chất hóa học vào vị trí hoạt động của protease virus.
- **Dữ liệu đầu vào:** Protein thụ thể `protease_pocket.pdbqt` và tập tin thư viện phối tử `ligands_batch_500k.tar.zst` (350 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x0309` (`VinaVirtualScreening`), Multi-core CPU / GPU CUDA phân tán song song.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x0309 \
    --input ./drug_docking/ligands_batch_500k.tar.zst \
    --output ./drug_docking/top_100_candidates.csv \
    --bounty 11000000000 \
    --auditor-quorum 3 \
    --timeout 14400 \
    --tags "docking,autodock,drug-discovery"
  ```
- **Xác thực & Kết quả:** Kiểm tra lại ngẫu nhiên 50 phối tử hàng đầu bởi ban kiểm chứng. Trả về bảng xếp hạng năng lượng tự do Gibbs $\Delta G$ (kcal/mol).

---

## Case 065: Mô phỏng plasma nhiệt hạch Tokamak từ trường xoắn (PIC Plasma Simulation)
- **Tình huống:** Nhóm nghiên cứu năng lượng nhiệt hạch mô phỏng hiện tượng rối loạn từ thông và vi bất ổn định plasma ion trong buồng chân không Tokamak.
- **Dữ liệu đầu vào:** Cấu hình từ trường từ và mật độ ion `tokamak_eqdsk.bin` (85 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x030A` (`PlasmaTokamakPIC`), GPU GPU CUDA FP64 / High Bandwidth Memory.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x030A \
    --input ./fusion/tokamak_eqdsk.bin \
    --output ./fusion/plasma_turbulence.h5 \
    --bounty 14000000000 \
    --auditor-quorum 2 \
    --timeout 10800 \
    --tags "fusion,plasma,tokamak,h5"
  ```
- **Xác thực & Kết quả:** Kiểm tra nghiệm phương trình Vlasov-Maxwell và định luật cảm ứng điện từ Faraday.

---

## Case 066: Tái tạo ảnh cắt lớp địa chấn 3D Full Waveform Inversion (Geophysics FWI)
- **Tình huống:** Đơn vị khảo sát địa chất dầu khí tái tạo mô hình vận tốc sóng âm lòng đất 3D từ dữ liệu cảm biến địa chấn nổ mìn ngoài khơi.
- **Dữ liệu đầu vào:** File địa chấn định dạng SEG-Y `marine_seismic_shots.segy` (1.2 GB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x030B` (`SeismicFwi3D`), GPU Workstation $\ge 48$GB VRAM.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x030B \
    --input ./seismic/marine_seismic_shots.segy \
    --output ./seismic/velocity_model_3d.h5 \
    --bounty 18000000000 \
    --auditor-quorum 2 \
    --timeout 21600 \
    --tags "geophysics,fwi,segy,cuda"
  ```
- **Xác thực & Kết quả:** Kiểm tra gradient đạo hàm sóng ngược và hàm mục tiêu giảm dư lượng sóng đàn hồi.

---

## Case 067: Tối ưu hình học cánh tuabin gió ngoài khơi (Aeroelastic FAST)
- **Tình huống:** Nhà sản xuất năng lượng tái tạo phân tích rung động kết hợp thủy-khí-đàn hồi của tuabin gió nổi 15MW khi chịu bão cấp 14.
- **Dữ liệu đầu vào:** File cấu hình khí động cánh `nrel_15mw_offshore.fst` (40 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x030C` (`OpenFastOffshore`), Đa nhân CPU tốc độ xung cao.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x030C \
    --input ./turbine/nrel_15mw_offshore.fst \
    --output ./turbine/blade_load_summary.csv \
    --bounty 4200000000 \
    --auditor-quorum 3 \
    --timeout 3600 \
    --tags "renewables,wind-turbine,openfast"
  ```
- **Xác thực & Kết quả:** Kiểm toán biên độ dao động uốn cánh và momen xoắn trục rotor không vượt giới hạn mỏi vật liệu.

---

## Case 068: Phân tích phổ biểu hiện gen đơn bào 100.000 tế bào (Scanpy / Seurat)
- **Tình huống:** Viện ung thư cần giải mã dữ liệu Single-cell RNA-seq 100k tế bào miễn dịch để phát hiện dòng tế bào đột biến hiếm gặp.
- **Dữ liệu đầu vào:** File ma trận đếm gen thưa `raw_counts_100k.h5ad` (320 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x030D` (`SingleCellRnaAnalysis`), RAM $\ge 64$GB, tăng tốc GPU RAPIDS cuGraph / cuML.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x030D \
    --input ./bio/raw_counts_100k.h5ad \
    --output ./bio/clustered_umap_markers.h5ad \
    --bounty 6000000000 \
    --auditor-quorum 3 \
    --timeout 5400 \
    --tags "bioinformatics,single-cell,scanpy,rapids"
  ```
- **Xác thực & Kết quả:** Đối chiếu các marker gen sinh học đặc trưng và ma trận biểu hiện láng giềng gần nhất UMAP/Leiden.

---

## Case 069: Mô phỏng dòng chảy ngầm và lan truyền chất ô nhiễm (MODFLOW 6)
- **Tình huống:** Cơ quan bảo vệ môi trường mô phỏng rò rỉ hóa chất độc hại từ bãi thải công nghiệp thâm nhập vào tầng ngậm nước ngầm trong 10 năm.
- **Dữ liệu đầu vào:** Bộ tham số thủy văn tầng chứa nước `modflow_aquifer_model.zip` (110 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x030E` (`ModflowGroundwaterSim`), CPU đa nhân.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x030E \
    --input ./hydro/modflow_aquifer_model.zip \
    --output ./hydro/plume_dispersion_10yr.vtk \
    --bounty 4000000000 \
    --auditor-quorum 2 \
    --timeout 3600 \
    --tags "hydrology,environment,modflow6"
  ```
- **Xác thực & Kết quả:** Kiểm chứng cân bằng lưu lượng nước ngầm vào/ra (Mass balance error < 0.1%).

---

## Case 070: Phân tích phổ rung động âm học buồng động cơ phản lực (Acoustic BEM)
- **Tình huống:** Kỹ sư âm học hàng không tính toán mức áp suất âm thanh (SPL) và bức xạ tiếng ồn của vòi phun động cơ phản lực ở tần số cao.
- **Dữ liệu đầu vào:** File lưới bề mặt vòi phun và điều kiện biên âm thanh `jet_nozzle_mesh.bdf` (150 MB).
- **Opcode & Yêu cầu phần cứng:** Opcode `0x030F` (`AcousticBemJetNoise`), GPU CUDA giải hệ phương trình ma trận đầy đối xứng phức.
- **Lệnh điều phối:**
  ```bash
  skynet compute dispatch \
    --opcode 0x030F \
    --input ./acoustics/jet_nozzle_mesh.bdf \
    --output ./acoustics/sound_pressure_levels.json \
    --bounty 6800000000 \
    --auditor-quorum 3 \
    --timeout 7200 \
    --tags "acoustics,bem,noise-reduction,cuda"
  ```
- **Xác thực & Kết quả:** Kiểm tra giá trị áp suất vi phân Kirchhoff-Helmholtz và xuất bản đồ tiếng ồn trường xa (Far-field SPL polar map).

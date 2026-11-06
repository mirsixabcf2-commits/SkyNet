# Phân Khu 04: Kỹ Thuật, Cơ Học & Bán Dẫn (100 Opcodes)

Phân khu này định nghĩa 100 mã tác vụ chuyên biệt phục vụ các Kỹ sư cơ khí, Hàng không vũ trụ, Ô tô điện, Xây dựng dân dụng, Điện tử và Vi mạch bán dẫn: từ Mô phỏng khí động học chất lưu (OpenFOAM CFD), Phân tích phần tử hữu hạn ứng suất độ mỏi (CalculiX FEA), Tính toán tản nhiệt linh kiện điện tử cho đến Mô phỏng mạch tích hợp SPICE và Tối ưu hóa hình học Topo.

---

## Danh Mục Chi Tiết 100 Opcodes Kỹ Thuật & CAE/CFD (`0x4001` – `0x4064`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x4001` | `skynet://hpc.cfd.openfoam_simplefoam` | Mô phỏng khí động học dòng khí không nén ổn định xung quanh thân xe | 32 Core CPU | `capsule-openfoam-hpc` |
| `0x4002` | `skynet://hpc.cfd.openfoam_pimplefoam` | Mô phỏng trường xoáy cuộn khí không ổn định LES quanh cánh máy bay | 64 Core CPU | `capsule-openfoam-les` |
| `0x4003` | `skynet://hpc.cfd.su2_compressible_transonic`| Tính toán khí động học dòng cận âm và siêu âm máy bay phản lực (SU2)| 32 Core CPU | `capsule-su2-solver` |
| `0x4004` | `skynet://hpc.cfd.aerodynamic_drag_cd` | Tự động tính tích phân lực cản khí động học $C_d$ và lực nâng $C_l$ | 16 GB RAM | `capsule-aero-coeffs` |
| `0x4005` | `skynet://hpc.cfd.interfoam_free_surface` | Mô phỏng đập tràn xả lũ và sóng vỗ va đập mạn tàu thủy | 64 Core CPU | `capsule-vof-interfoam`|
| `0x4006` | `skynet://hpc.cfd.rho_pimple_combustion` | Mô phỏng buồng đốt nhiên liệu động cơ tuabin khí phản lực | 64 Core CPU | `capsule-combustion-cfd`|
| `0x4007` | `skynet://hpc.cfd.snappyhexmesh_gen` | Tự động chia lưới đa giác SnappyHexMesh 20 triệu ô từ file STL | 64 GB RAM | `capsule-snappy-mesh` |
| `0x4008` | `skynet://hpc.cfd.thermal_cht_multiregion`| Tính toán truyền nhiệt liên hợp chất lỏng - kim loại tản nhiệt CPU | 32 Core CPU | `capsule-cht-thermal` |
| `0x4009` | `skynet://hpc.cfd.dpm_droplet_spray` | Mô phỏng vòi phun sương làm mát và quỹ đạo phân tán hạt giọt | 32 GB RAM | `capsule-spray-dpm` |
| `0x400A` | `skynet://hpc.cfd.ventilation_hvac_indoor` | Tính toán trường nhiệt độ và luồng đối lưu thông gió phòng sạch | 16 GB RAM | `capsule-hvac-airflow` |
| `0x400B` | `skynet://hpc.fea.calculix_static_stress` | Phân tích ứng suất tĩnh Von Mises và chuyển vị khung dầm kim loại | 32 GB RAM | `capsule-calculix-fea` |
| `0x400C` | `skynet://hpc.fea.calculix_modal_frequency`| Tìm tần số dao động riêng và dạng mode chống hiện tượng cộng hưởng | 64 GB RAM | `capsule-modal-eigen` |
| `0x400D` | `skynet://hpc.fea.code_aster_nonlinear` | Phân tích biến dạng phi tuyến dẻo của bê tông cốt thép khi chịu nén | 64 GB RAM | `capsule-code-aster` |
| `0x400E` | `skynet://hpc.fea.contact_hertz_pressure` | Tính toán áp lực tiếp xúc bề mặt răng ăn khớp bánh răng hộp số | 32 GB RAM | `capsule-contact-fea` |
| `0x400F` | `skynet://hpc.fea.thermal_stress_expansion`| Mô phỏng nứt vỡ do giãn nở nhiệt không đồng đều trên piston nhôm | 32 GB RAM | `capsule-thermo-elastic`|
| `0x4010` | `skynet://hpc.fea.buckling_critical_load` | Tính tải trọng phá hủy mất ổn định uốn dọc (Buckling) cột tháp gió | 32 GB RAM | `capsule-buckling-fea` |
| `0x4011` | `skynet://hpc.fea.fatigue_sn_damage` | Ước lượng tuổi thọ mỏi kim loại dựa trên đường cong Wöhler S-N | 16 GB RAM | `capsule-fatigue-calc` |
| `0x4012` | `skynet://hpc.fea.impact_explicit_crash` | Mô phỏng va chạm trực diện xe ô tô thử nghiệm an toàn đâm va | 64 Core CPU | `capsule-open-crash` |
| `0x4013` | `skynet://hpc.fea.composite_tsai_wu` | Kiểm tra tiêu chí phá hủy vật liệu sợi carbon composite Tsai-Wu | 16 GB RAM | `capsule-composite-lam`|
| `0x4014` | `skynet://hpc.fea.weld_heat_affected_zone`| Mô phỏng chu trình nhiệt và ứng suất dư vùng ảnh hưởng nhiệt hàn HAZ | 32 GB RAM | `capsule-weld-fea` |
| `0x4015` | `skynet://hpc.opt.topology_optimization` | Tối ưu hóa cấu trúc Topo cắt giảm 40% khối lượng chi tiết cơ khí | 32 GB RAM | `capsule-topopt-simp` |
| `0x4016` | `skynet://hpc.opt.generative_bracket_cad`| Sinh thiết kế khung treo chịu lực bằng thuật toán di truyền sinh học | 32 GB RAM | `capsule-gen-bracket` |
| `0x4017` | `skynet://hpc.circuit.ngspice_transient` | Mô phỏng phân tích miền thời gian mạch nguồn xung cao tần Buck/Boost | 8 GB RAM | `capsule-ngspice` |
| `0x4018` | `skynet://hpc.circuit.ngspice_ac_noise` | Phân tích đáp ứng tần số và phổ tạp âm biên độ mạch khuếch đại | 8 GB RAM | `capsule-spice-noise` |
| `0x4019` | `skynet://hpc.circuit.xyce_parallel_sim` | Chạy song song hàng nghìn kịch bản góc phần tư mạch tích hợp Xyce | 32 Core CPU | `capsule-xyce-parallel`|
| `0x401A` | `skynet://hpc.circuit.monte_carlo_corner` | Kiểm tra dung sai linh kiện R-L-C theo phân phối ngẫu nhiên Gauss | 16 GB RAM | `capsule-spice-mc` |
| `0x401B` | `skynet://hpc.em.fdtd_antenna_radiation` | Mô phỏng trường điện từ FDTD tính đồ thị búp sóng ăng-ten 5G/WiFi | 16 GB VRAM | `capsule-openems-fdtd` |
| `0x401C` | `skynet://hpc.em.pcb_signal_integrity_s21`| Đo suy hao truyền dẫn S21 và xuyên âm tín hiệu đường mạch PCB cao tốc | 32 GB RAM | `capsule-pcb-si` |
| `0x401D` | `skynet://hpc.em.motor_pmsm_flux_map` | Tính toán bản đồ từ thông và momen quay động cơ điện đồng bộ nam châm | 32 GB RAM | `capsule-femm-motor` |
| `0x401E` | `skynet://hpc.em.transformer_eddy_loss` | Mô phỏng dòng điện xoáy Foucault gây nóng lõi thép máy biến áp | 32 GB RAM | `capsule-eddy-loss` |
| `0x401F` | `skynet://hpc.vlsi.openlane_place_route` | Tự động định tuyến vị trí hàng triệu cổng logic vi mạch ASIC số | 64 GB RAM | `capsule-openlane-asic`|
| `0x4020` | `skynet://hpc.vlsi.klayout_drc_lvs_check` | Kiểm tra quy chuẩn hình học đúc chip DRC và khớp sơ đồ nguyên lý LVS| 32 GB RAM | `capsule-klayout-drc` |
| `0x4021` | `skynet://hpc.vlsi.magic_parasitic_extract`| Trích xuất điện dung và điện trở ký sinh trên layout vi mạch | 32 GB RAM | `capsule-magic-pex` |
| `0x4022` | `skynet://hpc.vlsi.optical_proximity_opc`| Bù quang sai thấu kính quang khắc bước sóng 193nm DUV (OPC) | 64 Core CPU | `capsule-opc-litho` |
| `0x4023` | `skynet://hpc.semi.tcad_mosfet_iv_curve` | Giải phương trình trôi dạt khuếch tán bán dẫn vẽ đặc tuyến I-V | 32 GB RAM | `capsule-tcad-device` |
| `0x4024` | `skynet://hpc.semi.bandgap_quantum_espresso`| Tính toán vùng cấm năng lượng vật liệu bán dẫn GaN / SiC thế hệ mới| 64 Core CPU | `capsule-qe-dft` |
| `0x4025` | `skynet://hpc.thermal.battery_pack_cooling`| Mô phỏng quá trình sinh nhiệt và tản nhiệt chất lỏng cụm pin xe điện | 32 GB RAM | `capsule-ev-battery-thm`|
| `0x4026` | `skynet://hpc.thermal.heat_pipe_phase_change`| Tính toán chu trình bay hơi và ngưng tụ chất lỏng trong ống đồng tản nhiệt| 16 GB RAM | `capsule-heatpipe-sim` |
| `0x4027` | `skynet://hpc.acoustics.bem_engine_noise` | Mô phỏng phát xạ tiếng ồn khoang động cơ xe ra môi trường xung quanh | 32 GB RAM | `capsule-acoustics-bem`|
| `0x4028` | `skynet://hpc.acoustics.muffler_transmission_loss`| Đo suy hao truyền âm của bộ giảm thanh ống xả khí động cơ xe máy | 16 GB RAM | `capsule-muffler-tl` |
| `0x4029` | `skynet://hpc.rotor.openfast_wind_15mw` | Mô phỏng khí đàn hồi tuabin gió ngoài khơi 15MW khi bão lớn gió giật| 32 Core CPU | `capsule-openfast-wind`|
| `0x402A` | `skynet://hpc.rotor.blade_element_bem` | Thiết kế tối ưu góc xoắn cánh quạt máy bay không người lái theo BEM | 8 GB RAM | `capsule-rotor-bem` |
| `0x402B` | `skynet://hpc.hydraulic.pipe_water_hammer` | Tính toán xung áp lực va đập thủy kích (Water Hammer) vỡ đường ống | 8 GB RAM | `capsule-water-hammer` |
| `0x402C` | `skynet://hpc.hydraulic.centrifugal_pump_h_q`| Vẽ đường đặc tính lưu lượng - cột áp (H-Q) máy bơm ly tâm trục đứng | 32 Core CPU | `capsule-pump-cfd` |
| `0x402D` | `skynet://hpc.civil.bridge_seismic_pushover`| Đánh giá khả năng chống chịu động đất của cầu dây văng bằng Pushover| 64 GB RAM | `capsule-bridge-pushover`|
| `0x402E` | `skynet://hpc.civil.tunnel_ground_settlement`| Mô phỏng sụt lún nền đất khi đào hầm ngầm metro bằng máy khiên đào TBM| 32 GB RAM | `capsule-tunnel-settle`|
| `0x402F` | `skynet://hpc.civil.dam_overtopping_erosion`| Tính toán nguy cơ xói mòn chân đập thủy điện khi xả lũ khẩn cấp | 32 GB RAM | `capsule-dam-erosion` |
| `0x4030` | `skynet://hpc.mech.gear_transmission_error`| Phân tích sai số truyền động ăn khớp và rung lắc bánh răng hành tinh | 16 GB RAM | `capsule-gear-kinematics`|
| `0x4031` | `skynet://hpc.mech.bearing_elastohydrodynamic`| Mô phỏng màng dầu bôi trơn thủy động đàn hồi trong ổ lăn trục chính | 16 GB RAM | `capsule-bearing-ehd` |
| `0x4032` | `skynet://hpc.mech.cam_follower_jerk` | Tối ưu hóa đường cong cam nạp xả triệt tiêu đạo hàm gia tốc Jerk | 8 GB RAM | `capsule-cam-profile` |
| `0x4033` | `skynet://hpc.aero.flutter_aeroelasticity` | Kiểm tra hiện tượng chấn động khí đàn hồi tự kích (Flutter) cánh máy bay| 64 GB RAM | `capsule-flutter-aero` |
| `0x4034` | `skynet://hpc.aero.sonic_boom_ground_overpressure`| Tính áp lực sóng xung kích chạm đất của máy bay siêu âm thế hệ mới | 32 Core CPU | `capsule-sonic-boom` |
| `0x4035` | `skynet://hpc.space.rocket_nozzle_expansion`| Mô phỏng luồng phụt khí cháy giãn nở qua loa phụt tên lửa đẩy vũ trụ | 64 Core CPU | `capsule-rocket-cfd` |
| `0x4036` | `skynet://hpc.space.satellite_reentry_aerothermal`| Tính toán bức xạ nhiệt ma sát bầu khí quyển khi vệ tinh hạ quỹ đạo | 64 Core CPU | `capsule-reentry-aero` |
| `0x4037` | `skynet://hpc.optics.zemax_ray_tracing` | Truy vết tia quang học tính quang sai hình cầu qua hệ 5 thấu kính | 8 GB RAM | `capsule-optical-rays` |
| `0x4038` | `skynet://hpc.optics.grating_diffraction_rcwa`| Tính toán hiệu suất nhiễu xạ ánh sáng qua cách tử quang tử RCWA | 16 GB RAM | `capsule-rcwa-diffract`|
| `0x4039` | `skynet://hpc.mfg.injection_molding_flow`| Mô phỏng dòng chảy điền đầy và co ngót khuôn ép nhựa công nghiệp | 32 GB RAM | `capsule-mold-flow` |
| `0x403A` | `skynet://hpc.mfg.sheet_metal_springback` | Dự đoán độ đàn hồi bật ngược (Springback) sau khi dập tấm kim loại | 32 GB RAM | `capsule-sheet-stamp` |
| `0x403B` | `skynet://hpc.mfg.cnc_milling_chatter_stability`| Vẽ biểu đồ thùy ổn định chống rung lắc dao phay CNC tốc độ cao | 16 GB RAM | `capsule-cnc-chatter` |
| `0x403C` | `skynet://hpc.mfg.additive_laser_melt_pool`| Mô phỏng vũng chảy kim loại in 3D laser bột kim loại SLM | 24 GB VRAM | `capsule-slm-meltpool` |
| `0x403D` | `skynet://hpc.plasma.tokamak_mhd_stability`| Kiểm tra tính ổn định từ thủy động học MHD chống bung plasma Tokamak | 64 Core CPU | `capsule-tokamak-mhd` |
| `0x403E` | `skynet://hpc.plasma.etching_semiconductor_ic`| Mô phỏng ăn mòn plasma vi mô khắc rãnh silicon kích thước 3 nanomet| 32 GB RAM | `capsule-plasma-etch` |
| `0x403F` | `skynet://hpc.solar.pv_dust_shading_loss` | Tính toán tổn thất sản lượng điện mặt trời do bụi bám góc nghiêng | 8 GB RAM | `capsule-pv-shading` |
| `0x4040` | `skynet://hpc.nuclear.mcnp_neutron_transport`| Mô phỏng Monte Carlo vận chuyển neutron trong lò phản ứng phân hạch | 64 Core CPU | `capsule-neutron-mcnp`|
| `0x4041` | `skynet://hpc.h2.hydrogen_storage_diffuse` | Đánh giá độ rò rỉ khuếch tán hydro qua thành bình composite 700 bar | 16 GB RAM | `capsule-h2-diffusion` |
| `0x4042` | `skynet://hpc.h2.fuel_cell_pem_efficiency` | Mô phỏng màng trao đổi proton PEM và hiệu suất pin nhiên liệu hydro | 32 GB RAM | `capsule-pemfc-model` |
| `0x4043` | `skynet://hpc.drilling.drill_string_vibration`| Phân tích xoắn uốn rung động cần khoan dầu khí sâu 4.000m dưới biển | 16 GB RAM | `capsule-drill-vibe` |
| `0x4044` | `skynet://hpc.flow.multiphase_slug_flow` | Nhận diện chế độ dòng chảy nút khí lỏng (Slug Flow) trong đường ống | 32 GB RAM | `capsule-multiphase-cfd`|
| `0x4045` | `skynet://hpc.turbo.compressor_surge_stall`| Dự báo ranh giới giới hạn bơm phập (Surge) của máy nén khí ly tâm | 32 Core CPU | `capsule-compressor-cfd`|
| `0x4046` | `skynet://hpc.turbo.turbine_blade_cooling_jet`| Mô phỏng màng khí làm mát lỗ li ti trên cánh tuabin khí 1.400 độ C | 64 Core CPU | `capsule-film-cooling` |
| `0x4047` | `skynet://hpc.auto.abs_braking_stop_distance`| Tối ưu hóa chu kỳ nhấp nhả phanh chống bó cứng ABS trên đường ướt | 8 GB RAM | `capsule-abs-model` |
| `0x4048` | `skynet://hpc.auto.suspension_quarter_car` | Tối ưu hóa thông số giảm chấn treo chủ động tạo độ êm dịu xe hơi | 8 GB RAM | `capsule-quarter-car` |
| `0x4049` | `skynet://hpc.auto.tire_pacejka_magic_formula`| Khớp đường cong lực bám lốp xe theo công thức ma thuật Pacejka | 4 GB RAM | `capsule-pacejka-tire` |
| `0x404A` | `skynet://hpc.auto.electric_powertrain_wltp`| Tính toán mức tiêu thụ điện năng chu trình thử nghiệm xe chuẩn WLTP| 8 GB RAM | `capsule-wltp-range` |
| `0x404B` | `skynet://hpc.hvac.datacenter_pue_thermal`| Tối ưu luồng khí lạnh hành lang kín giảm chỉ số tiêu thụ điện PUE | 32 GB RAM | `capsule-dc-cooling` |
| `0x404C` | `skynet://hpc.hvac.chiller_cop_optimization`| Lập lịch phối ghép tải cụm máy làm lạnh nước Chiller tiết kiệm điện | 8 GB RAM | `capsule-chiller-cop` |
| `0x404D` | `skynet://hpc.sound.ultrasonic_ndt_flaw` | Tái tạo ảnh khuyết tật bọt khí trong mối đòn kim loại từ sóng siêu âm| 16 GB RAM | `capsule-ndt-ultrasound`|
| `0x404E` | `skynet://hpc.laser.welding_keyhole_depth`| Mô phỏng độ sâu ngấu vũng hàn công nghệ Laser công suất cao Keyhole| 32 GB RAM | `capsule-laser-weld` |
| `0x404F` | `skynet://hpc.gear.scuffing_flash_temp` | Tính nhiệt độ chớp (Flash Temperature) chống xước bề mặt răng ăn khớp| 8 GB RAM | `capsule-scuffing-calc`|
| `0x4050` | `skynet://hpc.seal.elastomer_oring_squeeze`| Mô phỏng độ biến dạng nén và áp lực tiếp xúc kín khít của gioăng O-ring| 8 GB RAM | `capsule-oring-seal` |
| `0x4051` | `skynet://hpc.fastener.bolt_preload_torque` | Tính mối liên hệ giữa momen siết cờ-lê và lực kéo căng bu-lông cường độ| 4 GB RAM | `capsule-bolt-preload` |
| `0x4052` | `skynet://hpc.press.extrusion_die_profile` | Tối ưu hình học khuôn ép đùn nhôm định hình chống cong vênh lệch tâm | 32 GB RAM | `capsule-extrusion-die`|
| `0x4053` | `skynet://hpc.forge.closed_die_flash_waste`| Giảm thiểu phoi thừa bavia trong công nghệ rèn dập nóng kim loại | 32 GB RAM | `capsule-forging-sim` |
| `0x4054` | `skynet://hpc.cast.shrinkage_porosity_feed`| Định vị vị trí đặt đậu ngót bù co ngót khi đúc gang chi tiết máy | 32 GB RAM | `capsule-casting-poro` |
| `0x4055` | `skynet://hpc.clean.cyclone_separator_dust`| Tính toán hiệu suất thu gom hạt bụi ly tâm của thiết bị Cyclone | 16 GB RAM | `capsule-cyclone-cfd` |
| `0x4056` | `skynet://hpc.mix.static_mixer_pressure_drop`| Tính toán độ đồng đều hòa trộn và sụt áp của bộ trộn tĩnh đường ống| 16 GB RAM | `capsule-mixer-cfd` |
| `0x4057` | `skynet://hpc.dryer.spray_drying_droplet` | Mô phỏng quá trình sấy phun hạt sữa bột hòa tan trong tháp sấy | 32 GB RAM | `capsule-dryer-sim` |
| `0x4058` | `skynet://hpc.ro.membrane_desalination_flux`| Tính lưu lượng thấm nước ngọt qua màng lọc thẩm thấu ngược RO | 8 GB RAM | `capsule-ro-membrane` |
| `0x4059` | `skynet://hpc.rail.wheel_rail_wear_archard`| Mô phỏng độ mài mòn bánh xe tàu hỏa ma sát với đường ray theo Archard| 16 GB RAM | `capsule-wheel-rail` |
| `0x405A` | `skynet://hpc.rail.pantograph_catenary_arc`| Tính toán độ ổn định tiếp xúc cần đón điện nóc tàu và dây điện trên cao| 16 GB RAM | `capsule-pantograph` |
| `0x405B` | `skynet://hpc.mining.ball_mill_rock_crush` | Mô phỏng va đập nghiền đá quặng bằng bi thép trong máy nghiền quay | 64 Core CPU | `capsule-dem-crusher` |
| `0x405C` | `skynet://hpc.mining.slope_stability_safety`| Tính toán hệ số an toàn chống trượt sạt lở bờ dốc mỏ khai thác lộ thiên| 16 GB RAM | `capsule-slope-safety` |
| `0x405D` | `skynet://hpc.concrete.hydration_heat_crack`| Dự báo vết nứt do nhiệt thủy hóa tỏa nhiệt của khối bê tông đổ dày | 16 GB RAM | `capsule-concrete-heat`|
| `0x405E` | `skynet://hpc.wind.tall_building_vortex_shed`| Mô phỏng lực gió giật và dao động xoáy Von Kármán của tòa nhà chọc trời| 64 Core CPU | `capsule-building-wind`|
| `0x405F` | `skynet://hpc.glass.tempering_stress_quench`| Mô phỏng luồng thổi khí làm nguội nhanh tôi nhiệt kính cường lực | 16 GB RAM | `capsule-glass-quench` |
| `0x4060` | `skynet://hpc.sensor.mems_accelerometer_freq`| Phân tích độ nhạy và dải đo cảm biến gia tốc vi cơ điện tử MEMS | 16 GB RAM | `capsule-mems-sensor` |
| `0x4061` | `skynet://hpc.nano.cnt_thermal_conductivity`| Tính toán độ dẫn nhiệt siêu cao của ống nano carbon Carbon Nanotube | 32 GB RAM | `capsule-cnt-thermal` |
| `0x4062` | `skynet://hpc.bioeng.dialysis_hollow_fiber`| Mô phỏng hiệu quả lọc độc tố máu của bó sợi rỗng quả lọc thận nhân tạo | 16 GB RAM | `capsule-dialyzer-cfd` |
| `0x4063` | `skynet://hpc.safety.gas_leak_explosion_overpressure`| Mô phỏng kịch bản rò rỉ khí gas hóa lỏng bốc cháy tạo sóng nổ phá hủy | 32 GB RAM | `capsule-blast-sim` |
| `0x4064` | `skynet://hpc.energy.flywheel_stress_burst` | Tính toán tốc độ vòng quay an toàn giới hạn nổ vỡ bánh đà tích năng | 16 GB RAM | `capsule-flywheel-burst`|

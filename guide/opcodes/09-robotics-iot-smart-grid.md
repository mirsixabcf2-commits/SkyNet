# Phân Khu 09: Robot Tự Hành, Drone & Lưới Điện Thông Minh (80 Opcodes)

Phân khu này định nghĩa 80 mã tác vụ chuyên biệt phục vụ các Kỹ sư Robot, Nhà máy tự động hóa công nghiệp (Industry 4.0), Đội bay máy bay không người lái (Drone Swarm), Xe tự hành và Nhà điều hành lưới điện thông minh (Smart Grid / Virtual Power Plant): từ Định vị và dựng bản đồ 3D LiDAR SLAM, Hoạch định quỹ đạo bay không va chạm cho 500 chiếc drone, Điều khiển xe tự lái an toàn cho đến Cân bằng phụ tải lưới điện ảo VPP.

---

## Danh Mục Chi Tiết 80 Opcodes Robot & Smart Grid (`0x9001` – `0x9050`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x9001` | `skynet://robotics.slam.fast_lio_lidar_3d` | Khớp nối đám mây điểm LiDAR 3D độ trễ dưới 10ms (Fast-LIO2) | 8 GB VRAM | `capsule-fastlio-cuda` |
| `0x9002` | `skynet://robotics.slam.cartographer_2d_submap`| Xây dựng bản đồ lưới chiếm chỗ 2D cho robot hút bụi và xe tự hành AGV| 8 GB RAM | `capsule-cartographer` |
| `0x9003` | `skynet://robotics.slam.orb_slam3_visual_inertial`| Định vị bằng camera kết hợp cảm biến gia tốc quán tính IMU (VIO) | 8 GB RAM | `capsule-orbslam3` |
| `0x9004` | `skynet://robotics.swarm.drone_collision_free_4d`| Hoạch định quỹ đạo 4D không va chạm cho bầy đàn 500 chiếc drone xếp hình| 16 GB RAM | `capsule-swarm-motion` |
| `0x9005` | `skynet://robotics.swarm.consensus_flocking_reynolds`| Mô phỏng bầy đàn tự hành Reynolds (Tụ đàn, Thẳng hàng, Tách rời) | 8 GB RAM | `capsule-flocking-sim` |
| `0x9006` | `skynet://robotics.auto.path_hybrid_a_star` | Tìm kiếm lộ trình chuyển động khả thi cho xe ô tô có bán kính quay vòng| 8 GB RAM | `capsule-hybrid-astar` |
| `0x9007` | `skynet://robotics.auto.mpc_lane_tracking` | Bộ điều khiển dự báo mô hình (MPC) bám làn đường tốc độ 120 km/h | 8 GB RAM | `capsule-mpc-tracker` |
| `0x9008` | `skynet://robotics.auto.freespace_occupancy_grid`| Dự đoán không gian trống có thể lưu thông từ cụm camera 360 quanh xe | 16 GB VRAM | `capsule-occ-network` |
| `0x9009` | `skynet://robotics.auto.hdmap_lane_graph_gen` | Tự động sinh đồ thị tâm làn đường từ đám mây điểm quét đường phố | 16 GB RAM | `capsule-hdmap-builder`|
| `0x900A` | `skynet://robotics.arm.inverse_kinematics_6dof`| Giải bài toán động học nghịch đảo tay máy công nghiệp 6 bậc tự do (<1ms)| 2 GB RAM | `capsule-ik-robotics` |
| `0x900B` | `skynet://robotics.arm.ompl_motion_planning_rrt`| Lập kế hoạch gắp thả vật phẩm tránh chướng ngại vật bằng thuật toán RRT*| 8 GB RAM | `capsule-ompl-planner` |
| `0x900C` | `skynet://robotics.arm.grasp_pose_detection_6d`| Dự đoán tọa độ và góc kẹp tối ưu để gắp đồ vật lộn xộn trong thùng | 8 GB VRAM | `capsule-gpd-grasp` |
| `0x900D` | `skynet://robotics.quad.trot_gait_mpc_wbc` | Điều khiển bước chạy nhịp nhàng cho chó robot 4 chân vượt địa hình đá | 8 GB RAM | `capsule-quadruped-ctrl`|
| `0x900E` | `skynet://robotics.humanoid.zero_moment_point_zmp`| Tính toán điểm không mô-men (ZMP) giữ thăng bằng cho robot hình người 2 chân| 16 GB RAM | `capsule-zmp-bipedal` |
| `0x900F` | `skynet://robotics.sensor.imu_preintegration_graph`| Tích phân trước số liệu IMU phục vụ tối ưu hóa đồ thị hệ số tư thế | 4 GB RAM | `capsule-imu-preint` |
| `0x9010` | `skynet://robotics.sensor.extrinsic_camera_lidar`| Hiệu chuẩn tự động ma trận quay và tịnh tiến giữa LiDAR và Camera | 8 GB RAM | `capsule-lidar-cam-cal`|
| `0x9011` | `skynet://energy.vpp.battery_dispatch_milp` | Tối ưu hóa nạp/xả pin lưu trữ gia đình cân bằng phụ tải lưới điện ảo| 8 GB RAM | `capsule-vpp-milp` |
| `0x9012` | `skynet://energy.grid.ac_optimal_power_flow` | Giải bài toán trào lưu công suất tối ưu xoay chiều AC-OPF hệ thống điện | 16 GB RAM | `capsule-ac-opf-solver`|
| `0x9013` | `skynet://energy.grid.contingency_n_minus_1` | Đánh giá an toàn lưới điện khi xảy ra sự cố đứt 1 đường dây (Tiêu chí N-1)| 32 GB RAM | `capsule-n1-stability` |
| `0x9014` | `skynet://energy.grid.transient_stability_rotor`| Mô phỏng góc quay rotor máy phát điện đồng bộ khi sét đánh ngắn mạch | 32 GB RAM | `capsule-transient-grid`|
| `0x9015` | `skynet://energy.solar.pv_short_term_forecast` | Dự báo công suất phát điện trang trại điện mặt trời trước 15 phút qua ảnh mây| 12 GB VRAM | `capsule-pv-nowcast` |
| `0x9016` | `skynet://energy.wind.wake_effect_floris` | Mô phỏng hiệu ứng bám đuôi gió (Wake effect) của các tuabin gió san sát| 16 GB RAM | `capsule-floris-wake` |
| `0x9017` | `skynet://energy.load.building_hvac_mpc` | Điều khiển nhiệt độ điều hòa tòa nhà thông minh tiết kiệm 25% tiền điện | 8 GB RAM | `capsule-bldg-hvac-mpc`|
| `0x9018` | `skynet://energy.ev.smart_charging_scheduler`| Lập lịch sạc xe điện thông minh tránh quá tải máy biến áp khu đô thị | 8 GB RAM | `capsule-ev-charging` |
| `0x9019` | `skynet://iot.dsp.bearing_vibration_envelope`| Phân tích đường bao phổ rung động chẩn đoán nứt rãnh lăn vòng bi | 4 GB RAM | `capsule-bearing-fault`|
| `0x901A` | `skynet://iot.dsp.motor_current_signature_mcsa`| Phân tích đặc trưng dòng điện động cơ MCSA phát hiện gãy thanh dẫn rotor| 4 GB RAM | `capsule-mcsa-diagnose`|
| `0x901B` | `skynet://iot.dsp.acoustic_emission_leak` | Định vị điểm rò rỉ khí gas qua tín hiệu phát xạ âm thanh cao tần AE | 8 GB RAM | `capsule-leak-detect` |
| `0x901C` | `skynet://iot.dsp.sensor_drift_compensation`| Tự động hiệu chỉnh độ trôi tín hiệu cảm biến điện hóa theo nhiệt độ | 2 GB RAM | `capsule-drift-comp` |
| `0x901D` | `skynet://iot.gateway.mqtt_topic_anomaly` | Giám sát luồng bản tin MQTT phát hiện thiết bị IoT bị nhiễm mã độc botnet| 8 GB RAM | `capsule-mqtt-guard` |
| `0x901E` | `skynet://iot.mesh.ble_mesh_route_distance` | Tối ưu hóa số bước nhảy định tuyến mạng cảm biến không dây BLE Mesh | 4 GB RAM | `capsule-ble-routing` |
| `0x901F` | `skynet://iot.lora.gateway_collision_aloha`| Mô phỏng xác suất xung đột gói tin vô tuyến LoRaWAN trong khu công nghiệp| 8 GB RAM | `capsule-lora-aloha` |
| `0x9020` | `skynet://iot.battery.soh_kalman_estimate` | Ước lượng trạng thái sức khỏe pin lithium SoH qua bộ đếm Coulomb | 2 GB RAM | `capsule-battery-soh` |
| `0x9021` | `skynet://robotics.agv.traffic_junction_deadlock`| Ngăn ngừa tắc nghẽn khóa chết Deadlock giữa 100 xe tự hành trong kho hàng| 8 GB RAM | `capsule-agv-deadlock` |
| `0x9022` | `skynet://robotics.agv.pallet_detection_pcd` | Tự động định vị lỗ xỏ càng pallet hàng từ cảm biến đám mây điểm 3D | 8 GB RAM | `capsule-pallet-detect`|
| `0x9023` | `skynet://robotics.nav.costmap_inflation_layer`| Thổi phồng lớp chi phí an toàn xung quanh chướng ngại vật cho Nav2 | 4 GB RAM | `capsule-nav2-costmap` |
| `0x9024` | `skynet://robotics.nav.teb_local_planner` | Bộ lập kế hoạch cục bộ Timed Elastic Band (TEB) tránh người đi bộ | 8 GB RAM | `capsule-teb-planner` |
| `0x9025` | `skynet://robotics.swarm.perimeter_surveillance`| Phân chia khu vực tuần tra biên giới tối ưu cho đội bay không người lái | 8 GB RAM | `capsule-patrol-swarm` |
| `0x9026` | `skynet://robotics.swarm.search_and_rescue_pso`| Thuật toán bầy đàn PSO tìm kiếm nạn nhân mất tích trong rừng núi | 8 GB RAM | `capsule-pso-rescue` |
| `0x9027` | `skynet://robotics.swarm.uwb_relative_ranging` | Định vị tương đối giữa các robot dựa trên chênh lệch thời gian bay UWB| 4 GB RAM | `capsule-uwb-swarm` |
| `0x9028` | `skynet://robotics.underwater.auv_buoyancy_ctrl`| Điều khiển độ sâu và phao nổi tàu lặn không người lái AUV biển sâu | 4 GB RAM | `capsule-auv-buoyancy` |
| `0x9029` | `skynet://robotics.underwater.sonar_mosaic_stitch`| Ghép nối dải ảnh quét sóng âm Sonar đáy biển tìm xác tàu đắm | 16 GB RAM | `capsule-sonar-mosaic` |
| `0x902A` | `skynet://robotics.pipe.pipeline_crawler_ndt` | Kiểm tra khuyết tật rỉ sét bên trong lòng đường ống dẫn dầu khí ngầm | 8 GB RAM | `capsule-pipe-crawler` |
| `0x902B` | `skynet://robotics.agri.weed_laser_targeting` | Điều khiển đầu phát laser diệt cỏ dại chính xác từng gốc rau | 8 GB VRAM | `capsule-laser-weeder` |
| `0x902C` | `skynet://robotics.agri.fruit_picking_reach` | Lập quỹ đạo tay máy nhẹ nhàng vươn tới hái chùm nho không dập nát | 8 GB RAM | `capsule-fruit-picker` |
| `0x902D` | `skynet://robotics.construct.brick_laying_plan`| Hoạch định trình tự xếp gạch xây tường phẳng tuyệt đối bằng robot | 8 GB RAM | `capsule-brick-robot` |
| `0x902E` | `skynet://robotics.construct.rebar_tying_vision`| Tự động nhận diện điểm giao nhau của lưới thép sàn để buộc kẽm | 8 GB VRAM | `capsule-rebar-tying` |
| `0x902F` | `skynet://energy.grid.harmonic_thd_calc` | Đo độ méo hài tổng điện áp THD bảo đảm tiêu chuẩn cấp điện công nghiệp| 4 GB RAM | `capsule-thd-harmonics`|
| `0x9030` | `skynet://energy.grid.fault_location_travelling_wave`| Định vị vị trí sét đánh đứt đường dây điện 500kV bằng sóng chạy | 8 GB RAM | `capsule-wave-fault` |
| `0x9031` | `skynet://energy.grid.islanding_detection_passive`| Phát hiện mất lưới điện lực để ngắt điện mặt trời tránh điện giật thợ| 2 GB RAM | `capsule-anti-islanding`|
| `0x9032` | `skynet://energy.grid.dynamic_line_rating_dlr` | Tính toán khả năng mang tải thực tế của dây dẫn theo nhiệt độ gió | 4 GB RAM | `capsule-dlr-rating` |
| `0x9033` | `skynet://energy.market.locational_marginal_price`| Tính toán giá điện biên theo nút thị trường bán buôn điện cạnh tranh | 16 GB RAM | `capsule-lmp-market` |
| `0x9034` | `skynet://energy.market.spinning_reserve_bidding`| Lập kế hoạch chào thầu dịch vụ phụ trợ dự phòng quay cho nhà máy điện | 8 GB RAM | `capsule-reserve-bid` |
| `0x9035` | `skynet://energy.thermal.cogeneration_chp_opt` | Tối ưu hóa đồng phát nhiệt - điện (CHP) cấp hơi và điện cho nhà máy giấy| 8 GB RAM | `capsule-chp-optimize` |
| `0x9036` | `skynet://energy.biomass.gasification_syngas`| Mô phỏng quá trình khí hóa trấu và mùn cưa tạo khí tổng hợp Syngas | 16 GB RAM | `capsule-biomass-gas` |
| `0x9037` | `skynet://energy.hydro.pumped_storage_dispatch`| Tối ưu hóa chu trình bơm nước lên hồ trên ban đêm và xả phát điện giờ cao điểm| 8 GB RAM | `capsule-pumped-hydro` |
| `0x9038` | `skynet://energy.microgrid.droop_control_v_f` | Mô phỏng điều khiển sụt áp Droop cân bằng tần số và điện áp lưới điện đảo| 8 GB RAM | `capsule-droop-ctrl` |
| `0x9039` | `skynet://iot.security.firmware_diff_fota` | Sinh gói nhị phân cập nhật firmware FOTA siêu nén qua đường truyền chậm| 4 GB RAM | `capsule-fota-diff` |
| `0x903A` | `skynet://iot.security.secure_boot_manifest` | Kiểm tra chuỗi niềm tin khởi động an toàn Secure Boot cho chip IoT | 2 GB RAM | `capsule-secure-boot` |
| `0x903B` | `skynet://iot.water.distribution_epanet_leak`| Mô phỏng thủy lực mạng lưới cấp nước sạch và phát hiện vỡ ống (EPANET)| 8 GB RAM | `capsule-epanet-water` |
| `0x903C` | `skynet://iot.traffic.scats_signal_adaptive` | Tự động đổi pha đèn tín hiệu giao thông ngã tư theo mật độ xe chờ | 8 GB RAM | `capsule-scats-traffic`|
| `0x903D` | `skynet://iot.waste.smart_bin_collection_route`| Lập lộ trình xe gom rác thông minh chỉ ghé các thùng rác đã đầy trên 80%| 8 GB RAM | `capsule-smart-bin-vrp`|
| `0x903E` | `skynet://iot.parking.ultrasound_space_map` | Tổng hợp trạng thái 2.000 cảm biến chỗ đỗ xe tầng hầm trung tâm thương mại| 4 GB RAM | `capsule-smart-parking`|
| `0x903F` | `skynet://iot.light.led_dimming_astronomic` | Tự động tăng giảm độ sáng đèn đường đô thị theo giờ mặt trời mọc/lặn | 2 GB RAM | `capsule-astro-dimming`|
| `0x9040` | `skynet://robotics.sensor.thermal_overheat_watch`| Giám sát nhiệt độ động cơ servo khớp tay máy tránh kẹt trục cháy cuộn dây| 2 GB RAM | `capsule-motor-thermal`|
| `0x9041` | `skynet://robotics.safety.laser_scanner_zone`| Tự động dừng khẩn cấp robot khi có công nhân bước vào vùng quét nguy hiểm| 2 GB RAM | `capsule-safety-zone` |
| `0x9042` | `skynet://robotics.haptic.teleoperation_force_feedback`| Truyền lực cản phản hồi xúc giác cho bác sĩ phẫu thuật điều khiển từ xa| 4 GB RAM | `capsule-haptic-force` |
| `0x9043` | `skynet://robotics.clean.solar_panel_cleaning_bot`| Điều khiển xe robot tự bò lau sạch bụi trên mặt kính tấm pin mặt trời | 4 GB RAM | `capsule-panel-cleaner`|
| `0x9044` | `skynet://robotics.clean.facade_window_wash` | Cân bằng cáp treo robot lau kính mặt ngoài tòa nhà cao tầng khi gió lớn | 4 GB RAM | `capsule-facade-clean` |
| `0x9045` | `skynet://energy.substation.iec61850_goose` | Giám sát và giải mã bức điện bảo vệ so lệch GOOSE trạm biến áp số | 4 GB RAM | `capsule-iec61850` |
| `0x9046` | `skynet://energy.substation.synchrophasor_pmu`| Xử lý số liệu đo góc pha đồng bộ Phasor PMU phát hiện dao động liên vùng | 16 GB RAM | `capsule-pmu-phasor` |
| `0x9047` | `skynet://energy.battery.cell_balancing_passive`| Điều khiển xả nhiệt cân bằng điện áp giữa 96 cell pin trong pack xe điện| 2 GB RAM | `capsule-cell-balance` |
| `0x9048` | `skynet://energy.battery.thermal_runaway_early_warn`| Cảnh báo sớm hiện tượng thoát nhiệt bốc cháy pin trước 5 phút bằng áp suất| 4 GB RAM | `capsule-runaway-warn` |
| `0x9049` | `skynet://energy.wind.blade_pitch_pid_tuning`| Tinh chỉnh bộ điều khiển PID góc xoay cánh quạt gió ổn định công suất phát| 4 GB RAM | `capsule-pitch-pid` |
| `0x904A` | `skynet://energy.wind.offshore_subsea_cable_loss`| Tính toán tổn thất điện năng đường cáp ngầm cao thế xoay chiều dưới biển| 8 GB RAM | `capsule-subsea-cable` |
| `0x904B` | `skynet://robotics.auto.bev_multi_camera_fusion`| Ghép 6 luồng camera thành góc nhìn từ trên cao xuống Bird's Eye View (BEV)| 16 GB VRAM | `capsule-bev-fusion` |
| `0x904C` | `skynet://robotics.auto.driver_drowsiness_monitor`| Phát hiện tài xế ngủ gật qua tần suất chớp mắt và tư thế ngả đầu | 4 GB VRAM | `capsule-dms-vision` |
| `0x904D` | `skynet://robotics.auto.platooning_v2v_distance`| Duy trì khoảng cách 5 mét giữa đoàn xe tải chạy nối đuôi nhau qua sóng V2V| 4 GB RAM | `capsule-v2v-platoon` |
| `0x904E` | `skynet://robotics.auto.traffic_light_state_ocr`| Nhận diện màu đèn giao thông và đếm ngược số giây từ khoảng cách 100m | 6 GB VRAM | `capsule-traffic-light`|
| `0x904F` | `skynet://iot.telecom.cell_tower_handover_pred`| Dự báo điểm chuyển giao trạm phát sóng di động khi tàu cao tốc chạy 300km/h| 8 GB RAM | `capsule-cell-handover`|
| `0x9050` | `skynet://iot.telecom.massive_mimo_beamforming`| Tính toán ma trận trọng số búp sóng 64 ăng-ten phát sóng tập trung vào thuê bao| 16 GB RAM | `capsule-mimo-beam` |

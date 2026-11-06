# Phân Khu 05: Trái Đất, Khí Tượng & Không Gian (80 Opcodes)

Phân khu này định nghĩa 80 mã tác vụ chuyên biệt phục vụ các Đài khí tượng thủy văn, Viện địa vật lý, Khảo sát dầu khí khoáng sản, Cơ quan vũ trụ và Trung tâm viễn thám vệ tinh: từ Dự báo thời tiết vi khí hậu phân giải cao (WRF), Tái tạo mô hình vận tốc sóng âm địa chấn (Full Waveform Inversion), Phân tích ảnh viễn thám Radar khẩu độ tổng hợp (SAR) cho đến Mô phỏng cơ học quỹ đạo thiên văn (N-Body Astrophysics).

---

## Danh Mục Chi Tiết 80 Opcodes Khí Tượng & Địa Không Gian (`0x5001` – `0x5050`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x5001` | `skynet://earth.wrf.mesoscale_rain_24h` | Dự báo vi khí hậu và lượng mưa cực đoan đô thị lưới 500m (WRF) | 64 Core CPU | `capsule-wrf-mpi` |
| `0x5002` | `skynet://earth.wrf.typhoon_track_forecast` | Dự báo quỹ đạo bão nhiệt đới và sức gió giật cấp 16 Biển Đông | 64 Core CPU | `capsule-wrf-typhoon` |
| `0x5003` | `skynet://earth.seismic.fwi_3d_velocity_model`| Tái tạo mô hình vận tốc sóng đàn hồi 3D FWI từ dữ liệu địa chấn nổ | 48 GB VRAM | `capsule-seismic-fwi` |
| `0x5004` | `skynet://earth.seismic.rtm_depth_migration` | Di chuyển độ sâu ngược thời gian RTM định vị vỉa dầu khí sâu | 32 GB VRAM | `capsule-seismic-rtm` |
| `0x5005` | `skynet://earth.seismic.earthquake_hypocenter`| Định vị chính xác tâm chấn và độ sâu đứt gãy động đất thời gian thực| 16 GB RAM | `capsule-hypo-center` |
| `0x5006` | `skynet://earth.gis.insar_ground_subsidence` | Đo độ sụt lún mặt đất từng milimet từ ảnh vệ tinh radar giao thoa InSAR| 32 GB RAM | `capsule-insar-gmtsar` |
| `0x5007` | `skynet://earth.gis.sentinel2_ndvi_crop_health`| Tính toán chỉ số thực vật NDVI theo dõi sức khỏe lúa từ Sentinel-2 | 16 GB RAM | `capsule-gis-ndvi` |
| `0x5008` | `skynet://earth.gis.dem_flood_inundation_2d` | Mô phỏng bản đồ ngập lụt 2D theo thời gian khi triều cường dâng cao | 32 GB RAM | `capsule-flood-2d` |
| `0x5009` | `skynet://earth.gis.landslide_susceptibility`| Đánh giá nguy cơ sạt lở đất đồi dốc dựa trên địa hình và lượng mưa | 16 GB RAM | `capsule-landslide-gis`|
| `0x500A` | `skynet://earth.gis.osm_road_network_topology`| Xây dựng đồ thị topo giao thông toàn quốc từ dữ liệu OpenStreetMap | 32 GB RAM | `capsule-osm-graph` |
| `0x500B` | `skynet://earth.ocean.swan_wave_coastal` | Mô phỏng sóng biển ven bờ và năng lượng sóng đánh vào đê kè (SWAN)| 32 Core CPU | `capsule-swan-wave` |
| `0x500C` | `skynet://earth.ocean.roms_circulation_tide` | Mô phỏng dòng hải lưu đại dương và thủy triều vịnh Bắc Bộ (ROMS) | 64 Core CPU | `capsule-roms-ocean` |
| `0x500D` | `skynet://earth.ocean.oil_spill_trajectory` | Dự báo hướng trôi dạt và bốc hơi của vệt dầu tràn trên biển | 16 GB RAM | `capsule-oilspill-sim` |
| `0x500E` | `skynet://earth.ocean.tsunami_propagation_shoc`| Mô phỏng sóng thần lan truyền xuyên đại dương sau động đất ngầm | 32 GB RAM | `capsule-tsunami-sim` |
| `0x500F` | `skynet://earth.space.nbody_galaxy_collision`| Mô phỏng va chạm thiên hà và tương tác vật chất tối 50 triệu hạt | 24 GB VRAM | `capsule-nbody-gpu` |
| `0x5010` | `skynet://earth.space.orbital_kepler_tle_prop`| Lan truyền quỹ đạo vệ tinh LEO dự đoán va chạm rác vũ trụ (SGP4) | 8 GB RAM | `capsule-sgp4-orbit` |
| `0x5011` | `skynet://earth.space.solar_flare_cme_storm` | Mô phỏng bão từ mặt trời CME tác động tầng điện khí quyển Trái Đất | 32 Core CPU | `capsule-solar-cme` |
| `0x5012` | `skynet://earth.space.interplanetary_transfer`| Tối ưu quỹ đạo bay chuyển tiếp Hohmann tiết kiệm nhiên liệu sao Hỏa| 8 GB RAM | `capsule-lambert-orbit`|
| `0x5013` | `skynet://earth.air.cmaq_pm25_dispersion` | Mô phỏng lan truyền bụi mịn PM2.5 và ô nhiễm không khí công nghiệp | 32 Core CPU | `capsule-cmaq-air` |
| `0x5014` | `skynet://earth.air.aerosol_optical_depth_aod`| Tính toán độ dày quang học sol khí AOD từ vệ tinh quan trắc MODIS | 16 GB RAM | `capsule-modis-aod` |
| `0x5015` | `skynet://earth.hydro.modflow6_groundwater` | Mô phỏng dòng chảy tầng chứa nước ngầm và xâm nhập mặn đồng bằng | 32 GB RAM | `capsule-modflow6` |
| `0x5016` | `skynet://earth.hydro.hec_ras_river_sediment`| Tính toán xói lở lòng sông và bồi lắng phù sa hồ chứa (HEC-RAS) | 16 GB RAM | `capsule-hec-ras` |
| `0x5017` | `skynet://earth.climate.cmip6_downscaling` | Hạ độ phân giải dữ liệu biến đổi khí hậu toàn cầu CMIP6 về cấp tỉnh | 32 GB RAM | `capsule-cmip6-down` |
| `0x5018` | `skynet://earth.climate.drought_spei_index` | Tính toán chỉ số khô hạn chuẩn hóa bốc thoát hơi nước SPEI | 8 GB RAM | `capsule-spei-drought` |
| `0x5019` | `skynet://earth.forest.canopy_height_gedi` | Đo chiều cao tán rừng và trữ lượng carbon sinh khối từ LiDAR vệ tinh| 16 GB RAM | `capsule-gedi-canopy` |
| `0x501A` | `skynet://earth.forest.wildfire_spread_farsite`| Mô phỏng tốc độ lan truyền đám cháy rừng FARSITE theo hướng gió | 16 GB RAM | `capsule-wildfire-sim` |
| `0x501B` | `skynet://earth.geochem.water_rock_phreeqc` | Mô phỏng cân bằng hóa học nước ngầm hòa tan khoáng chất (PHREEQC) | 8 GB RAM | `capsule-phreeqc` |
| `0x501C` | `skynet://earth.gravity.bouguer_anomaly_map` | Xử lý số liệu trọng lực vệ tinh lập bản đồ dị thường Bouguer | 16 GB RAM | `capsule-gravity-map` |
| `0x501D` | `skynet://earth.magnet.geomagnetic_igrf13` | Tính toán góc lệch từ trường Trái Đất chuẩn quốc tế IGRF-13 | 4 GB RAM | `capsule-igrf-magnet` |
| `0x501E` | `skynet://earth.soil.moisture_smap_radiometer`| Trích xuất độ ẩm đất bề mặt từ dữ liệu bức xạ sóng viba SMAP | 16 GB RAM | `capsule-smap-soil` |
| `0x501F` | `skynet://earth.ice.glacier_flow_velocity` | Đo tốc độ trôi trượt của sông băng Nam Cực từ cặp ảnh radar | 16 GB RAM | `capsule-glacier-flow` |
| `0x5020` | `skynet://earth.volcano.ash_plume_dispersal`| Dự báo hướng phân tán của cột tro bụi núi lửa gây gián đoạn hàng không| 16 GB RAM | `capsule-ash-plume` |
| `0x5021` | `skynet://earth.space.pulsar_dispersion_dm` | Tìm kiếm tín hiệu sao xung vô tuyến Pulsar trong dữ liệu thiên văn FAST| 32 GB RAM | `capsule-pulsar-dm` |
| `0x5022` | `skynet://earth.space.exoplanet_transit_light`| Phát hiện hành tinh ngoài hệ Mặt Trời qua đường cong ánh sáng Kepler | 16 GB RAM | `capsule-transit-fit` |
| `0x5023` | `skynet://earth.space.gravitational_wave_matched`| Lọc tín hiệu sóng hấp dẫn va chạm lỗ đen từ giao thoa kế LIGO | 16 GB VRAM | `capsule-ligo-gw` |
| `0x5024` | `skynet://earth.space.asteroid_impact_crater` | Mô phỏng kích thước hố va chạm và sóng xung kích của thiên thạch rơi | 32 GB RAM | `capsule-crater-sim` |
| `0x5025` | `skynet://earth.space.space_debris_laser_divert`| Tính toán lực đẩy chùm tia laser mặt đất làm lệch quỹ đạo rác vũ trụ| 8 GB RAM | `capsule-laser-debris` |
| `0x5026` | `skynet://earth.space.starlink_radio_interference`| Mô phỏng dải nhiễu vô tuyến của chùm vệ tinh lên đài thiên văn | 16 GB RAM | `capsule-constell-rf` |
| `0x5027` | `skynet://earth.gis.dem_watershed_delineate` | Tự động phân chia lưu vực sông và mạng lưới suối nhánh từ file DEM | 16 GB RAM | `capsule-watershed-dem`|
| `0x5028` | `skynet://earth.gis.solar_rooftop_potential` | Tính toán diện tích mái nhà đón nắng và sản lượng điện mặt trời đô thị| 16 GB RAM | `capsule-roof-solar` |
| `0x5029` | `skynet://earth.gis.urban_heat_island_landsat`| Trích xuất nhiệt độ bề mặt đất LST lập bản đồ đảo nhiệt đô thị | 16 GB RAM | `capsule-heat-island` |
| `0x502A` | `skynet://earth.gis.viewshed_line_of_sight` | Tính toán tầm nhìn khuất của trạm quan sát quân sự trên đỉnh đồi | 8 GB RAM | `capsule-viewshed-los` |
| `0x502B` | `skynet://earth.gis.coastal_erosion_transect`| Đo tốc độ xói lở đường bờ biển theo mặt cắt ngang qua các năm | 8 GB RAM | `capsule-dsas-coastal` |
| `0x502C` | `skynet://earth.gis.multispectral_rice_stage`| Phân loại các giai đoạn sinh trưởng cây lúa (cấy, đẻ nhánh, trổ bông)| 16 GB RAM | `capsule-rice-growth` |
| `0x502D` | `skynet://earth.gis.building_footprint_seg` | Tự động bóc tách ranh giới móng 50.000 ngôi nhà từ ảnh viễn thám | 12 GB VRAM | `capsule-building-seg` |
| `0x502E` | `skynet://earth.gis.bathymetry_satellite_sdb` | Đo độ sâu đáy biển ven bờ từ ảnh màu nước biển đa phổ | 8 GB RAM | `capsule-bathy-sdb` |
| `0x502F` | `skynet://earth.weather.lightning_strike_density`| Thống kê mật độ sét đánh trên km2 phân bổ trạm chống sét lưới điện | 8 GB RAM | `capsule-lightning-map`|
| `0x5030` | `skynet://earth.weather.radar_reflectivity_qpe`| Ước lượng lượng mưa tức thời chuẩn xác cao từ radar thời tiết Doppler| 16 GB RAM | `capsule-radar-qpe` |
| `0x5031` | `skynet://earth.weather.fog_aviation_predict` | Dự báo hiện tượng sương mù mây thấp làm giảm tầm nhìn sân bay | 16 GB RAM | `capsule-fog-aviation` |
| `0x5032` | `skynet://earth.weather.frost_freezing_alert` | Cảnh báo sương muối và băng giá gây hại cây trồng nông nghiệp vùng cao| 8 GB RAM | `capsule-frost-alert` |
| `0x5033` | `skynet://earth.weather.wind_gust_boundary_layer`| Tính toán gió giật lớp biên bề mặt ảnh hưởng cầu dây văng | 16 GB RAM | `capsule-wind-gust` |
| `0x5034` | `skynet://earth.weather.uv_radiation_forecast`| Tính chỉ số bức xạ cực tím UV cực đại trong ngày theo độ dày tầng Ozone| 4 GB RAM | `capsule-uv-index` |
| `0x5035` | `skynet://earth.ocean.coral_bleaching_degree` | Tính toán chỉ số nhiệt tích tụ DHW dự báo hiện tượng tẩy trắng san hô| 8 GB RAM | `capsule-coral-dhw` |
| `0x5036` | `skynet://earth.ocean.tide_harmonic_analysis` | Phân tích 68 hằng số điều hòa thủy triều dự báo lịch nước lớn ròng | 4 GB RAM | `capsule-tide-harmonics`|
| `0x5037` | `skynet://earth.ocean.salinity_intrusion_estuary`| Mô phỏng độ mặn 4 phần nghìn xâm nhập sâu vào sông Tiền, sông Hậu | 16 GB RAM | `capsule-salt-intrusion`|
| `0x5038` | `skynet://earth.ocean.acoustic_sound_channel_sofar`| Vẽ đường truyền sóng âm thanh tầm xa trong kênh âm học SOFAR biển sâu| 8 GB RAM | `capsule-sofar-channel`|
| `0x5039` | `skynet://earth.ocean.wave_energy_flux_calc` | Tính toán thông lượng mật độ năng lượng sóng biển khảo sát trạm điện | 8 GB RAM | `capsule-wave-energy` |
| `0x503A` | `skynet://earth.seismic.ambient_noise_tomography`| Khảo sát cấu trúc vỏ Trái Đất không cần động đất bằng tiếng ồn vi địa | 32 GB RAM | `capsule-noise-tomo` |
| `0x503B` | `skynet://earth.seismic.receiver_function_moho`| Xác định độ sâu ranh giới Moho ngăn cách vỏ và manti Trái Đất | 16 GB RAM | `capsule-moho-rf` |
| `0x503C` | `skynet://earth.seismic.reservoir_pore_pressure`| Dự đoán áp suất lỗ rỗng địa tầng dầu khí phòng tránh sự cố phun trào | 16 GB RAM | `capsule-pore-pressure`|
| `0x503D` | `skynet://earth.seismic.avoo_fluid_factor_calc`| Phân tích biên độ theo góc phản xạ AVO phân biệt khí gas và nước ngầm | 16 GB RAM | `capsule-avo-fluid` |
| `0x503E` | `skynet://earth.seismic.microseismic_frack_monitor`| Giám sát vị trí các vi nứt nẻ thủy lực trong giếng khoan dầu đá phiến| 32 GB RAM | `capsule-frack-monitor`|
| `0x503F` | `skynet://earth.geo.fault_stress_coulomb_transfer`| Tính toán chuyển vị ứng suất Coulomb kích hoạt động đất liên hoàn | 16 GB RAM | `capsule-coulomb-stress`|
| `0x5040` | `skynet://earth.geo.crustal_deformation_gps` | Phân tích dữ liệu mạng lưới trạm định vị GNSS đo tốc độ trôi lục địa | 8 GB RAM | `capsule-gnss-tectonic`|
| `0x5041` | `skynet://earth.geo.sinkhole_cavity_gravity` | Phát hiện hố tử thần và hang hốc ngầm caster bằng đo vi trọng lực | 8 GB RAM | `capsule-sinkhole-grav`|
| `0x5042` | `skynet://earth.mining.geostatistics_kriging` | Ước tính trữ lượng quặng kim loại quý bằng phép nội suy Kriging 3D | 32 GB RAM | `capsule-kriging-3d` |
| `0x5043` | `skynet://earth.mining.open_pit_block_model` | Tối ưu hóa thứ tự bóc đất đá mỏ lộ thiên theo thuật toán Lerchs-Grossmann| 16 GB RAM | `capsule-pit-optimizer`|
| `0x5044` | `skynet://earth.air.methane_plume_tropomi` | Định vị nguồn rò rỉ khí mê-tan toàn cầu từ ảnh vệ tinh TROPOMI | 16 GB RAM | `capsule-ch4-plume` |
| `0x5045` | `skynet://earth.air.smoke_plume_injection_height`| Đo độ cao phụt tro khói của cháy rừng vươn tới tầng bình lưu | 8 GB RAM | `capsule-smoke-height` |
| `0x5046` | `skynet://earth.air.radioactive_fallout_hysplit`| Mô phỏng phát tán bụi phóng xạ hạt nhân theo luồng gió tầng cao | 16 GB RAM | `capsule-hysplit-rad` |
| `0x5047` | `skynet://earth.space.lunar_landing_hazard_dem`| Tìm kiếm bãi đáp an toàn không có tảng đá lớn trên bề mặt Mặt Trăng | 16 GB RAM | `capsule-lunar-hazard` |
| `0x5048` | `skynet://earth.space.mars_atmosphere_density` | Tính toán mật độ khí quyển sao Hỏa điều khiển hạ cánh tàu thám hiểm | 8 GB RAM | `capsule-mars-density` |
| `0x5049` | `skynet://earth.space.telescope_psf_deconvolve`| Khử mờ tán xạ khí quyển ảnh thiên văn bằng quang học thích ứng AO | 16 GB RAM | `capsule-astro-psf` |
| `0x504A` | `skynet://earth.space.radio_interferometry_vlbi`| Tổng hợp dữ liệu giao thoa kế vô tuyến đường đáy siêu dài VLBI | 64 GB RAM | `capsule-vlbi-synth` |
| `0x504B` | `skynet://earth.space.solar_irradiance_tsi` | Đo đạc hằng số tổng bức xạ mặt trời TSI giám sát chu kỳ 11 năm | 4 GB RAM | `capsule-solar-tsi` |
| `0x504C` | `skynet://earth.gis.cadastral_parcel_boundary` | Tự động khép thửa đất và phát hiện ranh giới chồng lấn địa chính | 8 GB RAM | `capsule-parcel-check` |
| `0x504D` | `skynet://earth.gis.road_damage_pothole_yolo`| Phát hiện ổ gà, nứt nẻ mặt đường nhựa từ camera gắn trên xe bus | 4 GB VRAM | `capsule-pothole-det` |
| `0x504E` | `skynet://earth.gis.sea_level_rise_scenario` | Vẽ kịch bản ngập lụt ven biển khi nước biển dâng thêm 50cm và 100cm | 16 GB RAM | `capsule-slr-scenarios`|
| `0x504F` | `skynet://earth.gis.illegal_logging_sentinel1`| Phát hiện sớm các điểm phá rừng trái phép xuyên mây qua radar xuyên thấu| 16 GB RAM | `capsule-sar-deforest` |
| `0x5050` | `skynet://earth.space.space_weather_geomag_kp`| Dự báo chỉ số Kp bão từ Trái Đất cảnh báo sự cố lưới điện truyền tải | 8 GB RAM | `capsule-kp-geomag` |

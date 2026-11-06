# Phân Khu 02: Đồ Họa 3D, Không Gian Số & Kỹ Xảo Điện Ảnh (100 Opcodes)

Phân khu này định nghĩa 100 mã tác vụ chuyên biệt về Dựng hình 3D (Rendering), Trường bức xạ ánh sáng (Gaussian Splatting & NeRF), Mô phỏng kỹ xảo vật lý hạt/khói lửa (VFX Fluids), và Xử lý tài nguyên game (Game Assets & Photogrammetry).

---

## Danh Mục Chi Tiết 100 Opcodes Đồ Họa & Spatial Computing (`0x2001` – `0x2064`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x2001` | `skynet://graphics.render.blender_cycles_optix` | Render khung hình Blender Cycles GPU OptiX Raytracing | 8 GB VRAM | `capsule-blender-optix` |
| `0x2002` | `skynet://graphics.render.blender_cycles_metal` | Render khung hình Blender Metal trên chip Apple Silicon | 16 GB UMA | `capsule-blender-metal` |
| `0x2003` | `skynet://graphics.splat.3dgs_train` | Huấn luyện mô hình 3D Gaussian Splatting từ tập ảnh chụp | 16 GB VRAM | `capsule-diffsplat` |
| `0x2004` | `skynet://graphics.splat.3dgs_render_4k` | Render video quỹ đạo camera từ 3D Gaussian Splatting 4K | 8 GB VRAM | `capsule-splat-render` |
| `0x2005` | `skynet://graphics.nerf.instant_ngp_train` | Tái tạo trường bức xạ nơ-ron NeRF thời gian thực (Instant-NGP) | 12 GB VRAM | `capsule-instant-ngp` |
| `0x2006` | `skynet://graphics.photo.colmap_sparse_recon` | Tính toán cấu trúc từ chuyển động SfM (COLMAP Sparse) | 32 GB RAM | `capsule-colmap-sfm` |
| `0x2007` | `skynet://graphics.photo.colmap_dense_stereo`| Tái lập đám mây điểm dày đặc MVS (COLMAP Dense) | 16 GB VRAM | `capsule-colmap-mvs` |
| `0x2008` | `skynet://graphics.mesh.poisson_surface_recon`| Tái tạo bề mặt lưới đa giác kín từ đám mây điểm Poisson | 16 GB RAM | `capsule-poisson-mesh` |
| `0x2009` | `skynet://graphics.mesh.quad_remesh_autopo` | Tối ưu hóa lưới đa giác thành toàn ô tứ giác (Quad Retopology) | 8 GB RAM | `capsule-quad-remesh` |
| `0x200A` | `skynet://graphics.mesh.decimate_simplify` | Giảm số lượng đa giác 90% phục vụ tối ưu hóa game di động | 4 GB RAM | `capsule-mesh-decimate` |
| `0x200B` | `skynet://graphics.uv.opti_unwrap_pack` | Tự động trải UV và tối ưu đóng gói diện tích texture atlas | 4 GB RAM | `capsule-uv-unwrapper` |
| `0x200C` | `skynet://graphics.bake.normal_map_8k` | Nướng Normal Map từ mô hình chi tiết cao sang chi tiết thấp | 8 GB VRAM | `capsule-xnormal-bake` |
| `0x200D` | `skynet://graphics.bake.ambient_occlusion` | Nướng bản đồ đổ bóng khép kín Ambient Occlusion 8K | 8 GB VRAM | `capsule-ao-bake` |
| `0x200E` | `skynet://graphics.bake.curvature_cavity` | Nướng bản đồ độ cong cạnh góc (Curvature & Cavity Map) | 4 GB VRAM | `capsule-bake-tools` |
| `0x200F` | `skynet://graphics.bake.lightmap_photon` | Nướng Lightmap tĩnh kiến trúc bằng bản đồ photon toàn cục | 16 GB VRAM | `capsule-lightmap-bake` |
| `0x2010` | `skynet://graphics.vfx.openvdb_fire_smoke` | Nướng mô phỏng thể tích khói lửa nổ định dạng OpenVDB | 32 GB RAM | `capsule-mantaflow-vdb`|
| `0x2011` | `skynet://graphics.vfx.flip_fluids_water` | Mô phỏng chất lỏng hạt nước bắn tung tóe FLIP Fluids | 32 GB RAM | `capsule-flip-fluids` |
| `0x2012` | `skynet://graphics.vfx.sand_granular_sim` | Mô phỏng bão cát và vật liệu dạng hạt rời rạc MPM | 24 GB VRAM | `capsule-mpm-granular` |
| `0x2013` | `skynet://graphics.vfx.cloth_vellum_tear` | Mô phỏng vải vóc, cờ bay và rách xé vật lý chân thực | 16 GB RAM | `capsule-vellum-cloth` |
| `0x2014` | `skynet://graphics.vfx.rigid_body_fracture`| Mô phỏng va đập phá hủy gạch đá vỡ vụn Voronoi Fracture | 16 GB RAM | `capsule-bullet-rbd` |
| `0x2015` | `skynet://graphics.vfx.hair_fur_dynamics` | Mô phỏng động lực học hàng triệu sợi tóc và lông thú | 16 GB VRAM | `capsule-hair-fur` |
| `0x2016` | `skynet://graphics.vfx.ocean_gerstner_wave`| Mô phỏng sóng biển đại dương và bọt trắng Gerstner Spectrum | 8 GB RAM | `capsule-ocean-waves` |
| `0x2017` | `skynet://graphics.anim.auto_rig_human` | Tự động tạo hệ thống xương khớp IK/FK cho nhân vật người | 4 GB RAM | `capsule-auto-rig` |
| `0x2018` | `skynet://graphics.anim.mocap_retarget` | Khớp chuyển động bắt chuyển động (MoCap BVH) vào bộ xương 3D | 4 GB RAM | `capsule-mocap-retarget`|
| `0x2019` | `skynet://graphics.anim.blendshape_facerig`| Tạo 52 hình thái biểu cảm khuôn mặt chuẩn Apple ARKit | 8 GB RAM | `capsule-arkit-blend` |
| `0x201A` | `skynet://graphics.anim.muscle_softbody_jiggle`| Mô phỏng độ rung lắc cơ bắp nhân vật theo quán tính chạy nhảy| 16 GB RAM | `capsule-muscle-sim` |
| `0x201B` | `skynet://graphics.anim.crowd_agent_avoid` | Mô phỏng chuyển động di tản của 10.000 người tránh vật cản | 16 GB RAM | `capsule-crowd-sim` |
| `0x201C` | `skynet://graphics.render.arnold_ass_batch` | Render theo lô file mô tả cảnh Arnold (.ass) Hollywood | 32 GB RAM | `capsule-arnold-core` |
| `0x201D` | `skynet://graphics.render.vray_standalone` | Render diễn họa kiến trúc nội thất siêu thực V-Ray | 16 GB VRAM | `capsule-vray-engine` |
| `0x201E` | `skynet://graphics.render.luxcore_bidir` | Render quang sai thấu kính và tụ quang đáy hồ caustics | 16 GB VRAM | `capsule-luxcore-bidir`|
| `0x201F` | `skynet://graphics.render.pbrt_v4_spectral` | Render quang phổ vật lý chuẩn nghiên cứu hàn lâm (pbrt-v4) | 16 GB RAM | `capsule-pbrt-v4` |
| `0x2020` | `skynet://graphics.denoise.optix_ai_denoise`| Khử nhiễu hình ảnh ray-tracing bằng nhân Tensor RTX | 6 GB VRAM | `capsule-optix-denoise`|
| `0x2021` | `skynet://graphics.denoise.intel_oidn_cpu` | Khử nhiễu hình ảnh ray-tracing bằng CPU đa luồng (Intel OIDN)| 8 GB RAM | `capsule-intel-oidn` |
| `0x2022` | `skynet://graphics.color.aces_color_transform`| Chuyển đổi không gian màu ACEScg sang sRGB / Rec.709 điện ảnh | 2 GB RAM | `capsule-ocio-aces` |
| `0x2023` | `skynet://graphics.shader.osl_compile_eval` | Biên dịch và tính toán mẫu chất liệu Open Shading Language | 4 GB RAM | `capsule-osl-compiler` |
| `0x2024` | `skynet://graphics.mat.substance_sbsar_render`| Render bộ map PBR từ file vật liệu tùy biến Substance .sbsar | 8 GB RAM | `capsule-substance-sbs`|
| `0x2025` | `skynet://graphics.geo.boolean_carve_csg` | Thực hiện phép cắt đục lỗ hình học đa giác phức tạp (CSG) | 8 GB RAM | `capsule-cgal-boolean` |
| `0x2026` | `skynet://graphics.geo.sdf_voxel_convert` | Chuyển đổi mô hình đa giác sang trường khoảng cách dấu (SDF)| 16 GB RAM | `capsule-sdf-mesh` |
| `0x2027` | `skynet://graphics.cad.step_to_gltf_tess` | Rời rạc hóa file CAD cơ khí (.step, .iges) sang .gltf web 3D | 8 GB RAM | `capsule-opencascade` |
| `0x2028` | `skynet://graphics.arch.citygml_lod3_mesh` | Dựng mô hình 3D thành phố từ dữ liệu bản đồ địa lý CityGML | 16 GB RAM | `capsule-citygml` |
| `0x2029` | `skynet://graphics.plant.speedtree_leaf_bake`| Sinh ngẫu nhiên cây cối rừng rậm và nướng texture cành lá | 8 GB RAM | `capsule-proc-trees` |
| `0x202A` | `skynet://graphics.terrain.world_machine_erosion`| Mô phỏng xói mòn thủy lực tạo địa hình đồi núi thực tế | 16 GB RAM | `capsule-terrain-erosion`|
| `0x202B` | `skynet://graphics.tex.seamless_tile_patch` | Biến ảnh chụp vân gỗ, tường gạch thành texture lặp vô tận | 4 GB RAM | `capsule-seamless-tex` |
| `0x202C` | `skynet://graphics.tex.delight_specular_rm` | Tách bỏ ánh sáng gắt trong ảnh chụp để thu chất liệu Albedo | 6 GB VRAM | `capsule-delight-pbr` |
| `0x202D` | `skynet://graphics.stereo.vr180_equirect_stitch`| Ghép 2 mắt fisheye thành video thực tế ảo không gian VR180 8K | 16 GB VRAM | `capsule-vr-stitch` |
| `0x202E` | `skynet://graphics.spatial.apple_spatial_video`| Mã hóa video không gian 3D tương thích kính Apple Vision Pro | 16 GB UMA | `capsule-spatial-mvhevc`|
| `0x202F` | `skynet://graphics.hologram.lightfield_render`| Render 45 góc nhìn cho màn hình hiển thị ảnh nổi Hologram | 24 GB VRAM | `capsule-lightfield-3d`|
| `0x2030` | `skynet://graphics.optics.lens_distortion_grid`| Đo đạc và hiệu chỉnh quang sai méo góc ống kính góc rộng | 4 GB RAM | `capsule-lens-calib` |
| `0x2031` | `skynet://graphics.depth.depth_map_inpaint` | Vá các lỗ thủng trong bản đồ chiều sâu quét từ LiDAR | 4 GB RAM | `capsule-depth-inpaint`|
| `0x2032` | `skynet://graphics.scene.usd_flatten_stage` | Gom các layer phức tạp của phim thành file Universal Scene USD| 16 GB RAM | `capsule-usd-stage` |
| `0x2033` | `skynet://graphics.cloud.pointcloud_to_mesh` | Nối lưới đa giác từ dữ liệu quét LiDAR không gian 50 triệu điểm | 32 GB RAM | `capsule-pcl-mesh` |
| `0x2034` | `skynet://graphics.vfx.fire_pyro_dissipation`| Tính toán tốc độ nguội và tản khói của luồng lửa cháy âm ỉ | 16 GB RAM | `capsule-pyro-fx` |
| `0x2035` | `skynet://graphics.cloth.pattern_sewing_fit`| Ráp các rập vải may 2D thành bộ quần áo 3D hoàn chỉnh | 8 GB RAM | `capsule-pattern-cloth`|
| `0x2036` | `skynet://graphics.render.radiance_cache_bake`| Nướng bộ nhớ đệm bức xạ ánh sáng động cho game thế giới mở | 16 GB VRAM | `capsule-radiance-cache`|
| `0x2037` | `skynet://graphics.mesh.laplacian_smoothing` | Làm mịn bề mặt gồ ghề của lưới quét 3D nha khoa | 8 GB RAM | `capsule-laplacian-mesh`|
| `0x2038` | `skynet://graphics.mesh.hole_filling_watertight`| Bịt kín các lỗ thủng trên mô hình để đưa vào máy in 3D | 8 GB RAM | `capsule-mesh-repair` |
| `0x2039` | `skynet://graphics.print.slicer_gcode_gen` | Cắt lát mô hình 3D thành mã lệnh điều khiển máy in G-Code | 4 GB RAM | `capsule-cura-slicer` |
| `0x203A` | `skynet://graphics.hair.strands_to_cards` | Chuyển đổi lọn tóc sợi sang các dải polygon cards tối ưu game | 8 GB RAM | `capsule-hair-cards` |
| `0x203B` | `skynet://graphics.anim.physics_ragdoll_setup`| Gắn thông số trọng lực và va chạm vật lý hình nhân Ragdoll | 4 GB RAM | `capsule-ragdoll-gen` |
| `0x203C` | `skynet://graphics.cam.camera_tracking_3d` | Khớp chuyển động camera thật từ video để chèn kỹ xảo 3D | 8 GB VRAM | `capsule-synth-match` |
| `0x203D` | `skynet://graphics.light.hdri_env_from_pano` | Trích xuất bản đồ môi trường ánh sáng HDRI từ ảnh toàn cảnh 360| 4 GB RAM | `capsule-hdri-extract` |
| `0x203E` | `skynet://graphics.splat.splat_lod_hierarchy`| Tạo cấu trúc cây phân tầng LOD hiển thị mượt mà 3DGS | 8 GB RAM | `capsule-splat-lod` |
| `0x203F` | `skynet://graphics.vfx.dust_atmospheric_fog`| Tính toán tán xạ sương mù thể tích khí quyển Ray-marching | 12 GB VRAM | `capsule-vol-fog` |
| `0x2040` | `skynet://graphics.game.lod_group_generator` | Tự động sinh ra 4 cấp độ chi tiết LOD0 - LOD3 cho toàn bộ asset| 8 GB RAM | `capsule-auto-lod` |
| `0x2041` | `skynet://graphics.mesh.vertex_color_bake` | Nướng chi tiết bẩn, rỉ sét trực tiếp vào màu đỉnh Vertex Color | 4 GB RAM | `capsule-vertex-color` |
| `0x2042` | `skynet://graphics.tex.knald_normal_deriv` | Chuyển đổi ảnh độ cao Height Map sang pháp tuyến Normal Map | 2 GB RAM | `capsule-height-normal`|
| `0x2043` | `skynet://graphics.cloth.stress_color_map` | Tạo bản đồ nhiệt hiển thị vùng vải bị kéo căng sắp rách | 8 GB RAM | `capsule-cloth-stress` |
| `0x2044` | `skynet://graphics.mesh.voxel_boolean_merge`| Hợp nhất hàng chục part chi tiết rời thành 1 khối voxel duy nhất| 16 GB RAM | `capsule-openvdb-csg` |
| `0x2045` | `skynet://graphics.vfx.lightning_arc_path` | Mô phỏng đường đi phân nhánh gãy khúc của tia sét điện thế | 2 GB RAM | `capsule-lightning-fx` |
| `0x2046` | `skynet://graphics.vfx.snow_accumulation` | Tự động phủ một lớp tuyết rơi đọng lên các bề mặt quay lên trời | 8 GB RAM | `capsule-snow-accum` |
| `0x2047` | `skynet://graphics.shading.subsurface_profile`| Tính toán tán xạ dưới bề mặt SSS cho làn da sáp và ngọc bích | 12 GB VRAM | `capsule-sss-profile` |
| `0x2048` | `skynet://graphics.anim.root_motion_extract` | Tách vector vận tốc di chuyển gốc khỏi hoạt hình chuyển động | 2 GB RAM | `capsule-root-motion` |
| `0x2049` | `skynet://graphics.optics.bokeh_shape_kernel`| Tạo hiệu ứng xóa phông Bokeh đa giác theo số lá khẩu ống kính| 4 GB VRAM | `capsule-bokeh-synth` |
| `0x204A` | `skynet://graphics.render.render_pass_split` | Tách 1 khung hình thành các pass: Cryptomatte, Depth, Diffuse | 16 GB RAM | `capsule-exr-splitter` |
| `0x204B` | `skynet://graphics.tex.compression_bc7` | Nén texture PBR sang định dạng nén GPU BC7 siêu tốc | 4 GB RAM | `capsule-tex-compress` |
| `0x204C` | `skynet://graphics.tex.compression_astc` | Nén texture PBR sang định dạng ASTC cho thiết bị di động | 4 GB RAM | `capsule-astc-codec` |
| `0x204D` | `skynet://graphics.mesh.polycount_budget_check`| Kiểm tra vi phạm giới hạn số lượng polygon trong game pipeline | 2 GB RAM | `capsule-mesh-audit` |
| `0x204E` | `skynet://graphics.audio.spatial_audio_mesh` | Tính toán phản xạ âm thanh nảy từ các bức tường hình học 3D | 8 GB RAM | `capsule-ray-acoustics`|
| `0x204F` | `skynet://graphics.vfx.bubble_underwater` | Mô phỏng bọt khí nổi lên và vỡ ra dưới lòng đại dương | 8 GB RAM | `capsule-bubble-sim` |
| `0x2050` | `skynet://graphics.vfx.sparks_bounce_collide`| Mô phỏng các đốm tia lửa hàn rơi vung vãi và nảy trên sàn | 4 GB RAM | `capsule-spark-collide`|
| `0x2051` | `skynet://graphics.anim.facial_speech_sync` | Khớp chuyển động khẩu hình môi nhân vật theo tệp thoại audio | 4 GB RAM | `capsule-lipsync-3d` |
| `0x2052` | `skynet://graphics.cad.mesh_to_nurbs_b_rep` | Chuyển đổi ngược từ lưới đa giác sang mặt cong toán học CAD | 16 GB RAM | `capsule-nurbs-fit` |
| `0x2053` | `skynet://graphics.geo.voronoi_3d_shatter` | Cắt khối bê tông thành hàng nghìn mảnh vỡ Voronoi 3D ngẫu nhiên| 8 GB RAM | `capsule-voronoi-3d` |
| `0x2054` | `skynet://graphics.splat.color_grade_3dgs` | Cân chỉnh bảng màu và độ bão hòa trực tiếp trên đám mây 3DGS | 8 GB VRAM | `capsule-splat-grade` |
| `0x2055` | `skynet://graphics.vfx.tornado_swirl_force` | Mô phỏng cơn lốc xoáy cuốn bay lá cây và rác thải xung quanh | 16 GB RAM | `capsule-vortex-sim` |
| `0x2056` | `skynet://graphics.vfx.footstep_dust_poof` | Tự động sinh đám bụi bay lên mỗi khi chân nhân vật dẫm đất | 4 GB RAM | `capsule-dust-poof` |
| `0x2057` | `skynet://graphics.lens.chromatic_aberration`| Tái tạo quang sai sắc biến dạng viền tím chân thực của máy ảnh| 2 GB RAM | `capsule-chroma-fx` |
| `0x2058` | `skynet://graphics.cam.stereo_anaglyph_gen` | Tạo ảnh nổi 3D xem bằng kính 2 màu đỏ-xanh (Anaglyph) | 2 GB RAM | `capsule-anaglyph` |
| `0x2059` | `skynet://graphics.mesh.curved_crease_sharp` | Giữ độ sắc nét cho các đường gân xe hơi khi tăng độ mịn mesh | 4 GB RAM | `capsule-crease-mesh` |
| `0x205A` | `skynet://graphics.render.toon_outline_ink` | Render nét vẽ mực viền truyện tranh phong cách Manga/Anime | 4 GB VRAM | `capsule-toon-shader` |
| `0x205B` | `skynet://graphics.light.ies_photometric_profile`| Tạo nguồn sáng đèn chiếu góc rộng theo thông số thực của hãng IES| 2 GB RAM | `capsule-ies-light` |
| `0x205C` | `skynet://graphics.anim.ik_reach_solver` | Giải bài toán nghịch đảo tay với tới vị trí tay nắm cửa | 2 GB RAM | `capsule-ik-solver` |
| `0x205D` | `skynet://graphics.anim.gait_walk_cycle` | Tự động sinh chu kỳ bước đi cân bằng theo địa hình dốc núi | 4 GB RAM | `capsule-gait-gen` |
| `0x205E` | `skynet://graphics.mesh.manifold_repair` | Vá lỗi tự cắt nhau và đa giác cụt không liền khối Non-manifold| 8 GB RAM | `capsule-manifold-fix` |
| `0x205F` | `skynet://graphics.vfx.ink_in_water_drop` | Mô phỏng giọt mực đen lan tỏa uốn lượn huyền ảo trong nước | 16 GB RAM | `capsule-ink-drop-sim` |
| `0x2060` | `skynet://graphics.render.deep_shadow_map` | Tạo bản đồ đổ bóng sâu Deep Shadow cho sợi tóc và làn khói mỏng| 8 GB VRAM | `capsule-deep-shadow` |
| `0x2061` | `skynet://graphics.splat.segment_3d_object` | Bóc tách riêng một đồ vật cụ thể ra khỏi trường ảnh 3DGS | 12 GB VRAM | `capsule-splat-seg` |
| `0x2062` | `skynet://graphics.terrain.road_spline_carve`| Tự động san phẳng địa hình men theo đường cong lộ trình cao tốc| 8 GB RAM | `capsule-road-carve` |
| `0x2063` | `skynet://graphics.mesh.ambient_occlusion_vertex`| Nướng trực tiếp độ tối bóng râm vào từng đỉnh của thảm cỏ game| 4 GB RAM | `capsule-vert-ao` |
| `0x2064` | `skynet://graphics.arch.sun_study_timelapse`| Mô phỏng góc chiếu nắng đổ bóng của tòa nhà từ sáng tới chiều | 8 GB RAM | `capsule-sun-study` |

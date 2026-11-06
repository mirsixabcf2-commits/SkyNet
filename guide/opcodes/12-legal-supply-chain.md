# Phân Khu 12: Pháp Lý, Thuế & Chuỗi Cung Ứng Logistics (40 Opcodes)

Phân khu này định nghĩa 40 mã tác vụ chuyên biệt phục vụ các Công ty luật, Văn phòng công chứng, Hãng kiểm toán xây dựng, Doanh nghiệp thương mại điện tử, Tập đoàn vận tải đa phương thức và Quản lý chuỗi cung ứng: từ Tối ưu hóa lộ trình đội xe giao hàng chặng cuối (Vehicle Routing Problem - VRP), Xếp hàng tối ưu thể tích container 3D (3D Bin Packing), Rà soát đối chiếu chéo hợp đồng kinh tế cho đến Kiểm toán hồ sơ quyết toán vốn đầu tư công.

---

## Danh Mục Chi Tiết 40 Opcodes Pháp Lý & Logistics (`0xC001` – `0xC028`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0xC001` | `skynet://logistics.vrp.fleet_routing_tw` | Tối ưu hóa lộ trình 500 xe tải giao hàng có ràng buộc khung giờ (VRP-TW)| 16 GB RAM | `capsule-or-tools-vrp` |
| `0xC002` | `skynet://logistics.pack.container_3d_knapsack`| Xếp 10.000 kiện hàng đa kích thước vào container 40 feet không lãng phí chỗ| 16 GB RAM | `capsule-bin-pack-3d` |
| `0xC003` | `skynet://logistics.wh.pallet_slotting_fast_mover`| Sắp xếp vị trí để hàng trong kho logistics theo tần suất xuất kho ABC | 8 GB RAM | `capsule-slotting-opt` |
| `0xC004` | `skynet://logistics.wh.forklift_path_congestion`| Tối ưu đường đi của 30 xe nâng tránh đâm nhau và ùn tắc trong lối đi hẹp | 8 GB RAM | `capsule-forklift-nav` |
| `0xC005` | `skynet://logistics.lastmile.crowdsourced_courier`| Ghép đơn hàng giao đồ ăn cho 5.000 tài xế xe ôm công nghệ thời gian thực | 16 GB RAM | `capsule-courier-match`|
| `0xC006` | `skynet://logistics.cold.temperature_logger_audit`| Quét dữ liệu cảm biến nhiệt độ thùng lạnh phát hiện đứt gãy chuỗi cung ứng| 4 GB RAM | `capsule-coldchain-chk`|
| `0xC007` | `skynet://logistics.customs.hs_code_classification`| Tự động áp mã số hải quan HS Code 8 chữ số cho hàng hóa xuất nhập khẩu | 8 GB RAM | `capsule-hscode-ai` |
| `0xC008` | `skynet://logistics.maritime.berth_allocation_port`| Lập lịch cập bến cầu cảng tối ưu cho các tàu mẹ container siêu trường | 16 GB RAM | `capsule-port-berth-opt`|
| `0xC009` | `skynet://logistics.maritime.stowage_plan_stability`| Lập sơ đồ xếp hàng lên tàu biển đảm bảo trọng tâm và độ ổn định mạn tàu | 16 GB RAM | `capsule-stowage-plan` |
| `0xC00A` | `skynet://logistics.air.belly_cargo_weight_balance`| Phân bổ tải trọng hàng hóa dưới bụng máy bay chở khách an toàn bay | 8 GB RAM | `capsule-air-cargo-bal`|
| `0xC00B` | `skynet://logistics.inventory.safety_stock_demand`| Tính mức tồn kho an toàn và điểm đặt hàng lại ROP theo biến động nhu cầu | 4 GB RAM | `capsule-safety-stock` |
| `0xC00C` | `skynet://logistics.inventory.bullwhip_effect_damp`| Giảm thiểu hiệu ứng chiếc roi da (Bullwhip effect) trong chuỗi cung ứng | 8 GB RAM | `capsule-bullwhip-filter`|
| `0xC00D` | `skynet://legal.contract.clause_diff_redline` | So sánh và đánh dấu đỏ (Redline) mọi điểm sửa đổi giữa 2 bản hợp đồng | 4 GB RAM | `capsule-contract-diff`|
| `0xC00E` | `skynet://legal.contract.indemnity_risk_scanner`| Quét các điều khoản bồi thường vô hạn và điều kiện thanh toán bất lợi | 8 GB RAM | `capsule-indemnity-chk`|
| `0xC00F` | `skynet://legal.contract.cross_ref_inconsistency`| Kiểm tra lỗi đứt gãy tham chiếu chéo giữa các điều khoản và phụ lục | 4 GB RAM | `capsule-crossref-audit`|
| `0xC010` | `skynet://legal.ediscovery.document_relevance` | Sàng lọc 500.000 email nội bộ tìm chứng cứ liên quan vụ kiện thương mại | 32 GB RAM | `capsule-ediscovery-ai`|
| `0xC011` | `skynet://legal.compliance.gdpr_right_to_forget`| Tìm kiếm và xóa triệt để mọi dữ liệu cá nhân trên toàn bộ các hệ thống | 16 GB RAM | `capsule-gdpr-eraser` |
| `0xC012` | `skynet://legal.audit.construction_settlement_210`| Kiểm toán hồ sơ quyết toán công trình theo Nghị định 210/2026/NĐ-CP | 8 GB RAM | `capsule-dinhson-audit`|
| `0xC013` | `skynet://legal.audit.construction_cross_check`| Đối chiếu chéo 3 bên: Bản vẽ hoàn công - Nhật ký thi công - Bảng thanh toán| 8 GB RAM | `capsule-draw-log-bill`|
| `0xC014` | `skynet://legal.audit.construction_material_split`| Bóc tách khối lượng vật tư từ dự toán gửi nhà cung cấp theo 9 nhóm | 4 GB RAM | `capsule-material-takeoff`|
| `0xC015` | `skynet://legal.ip.trademark_image_similarity` | So sánh độ tương đồng hình ảnh nhãn hiệu tránh vi phạm nhái thương hiệu | 8 GB VRAM | `capsule-trademark-ai` |
| `0xC016` | `skynet://legal.ip.patent_prior_art_semantic` | Tìm kiếm các bằng sáng chế đã có từ trước (Prior Art) ngữ nghĩa toàn cầu | 16 GB RAM | `capsule-patent-search`|
| `0xC017` | `skynet://legal.tax.vat_invoice_qr_verify` | Kiểm tra tính hợp lệ và chữ ký số hóa đơn điện tử GTGT từ Tổng cục Thuế | 4 GB RAM | `capsule-vat-verifier` |
| `0xC018` | `skynet://legal.tax.transfer_pricing_arm_length`| Kiểm tra nguyên tắc giá thị trường giao dịch liên kết chuyển giá | 8 GB RAM | `capsule-transfer-price`|
| `0xC019` | `skynet://legal.notary.id_card_fake_detect` | Phát hiện phôi CCCD/Hộ chiếu giả mạo qua dấu nổi và viền hoa văn vi mô | 8 GB VRAM | `capsule-id-fraud-check`|
| `0xC01A` | `skynet://legal.labor.work_hours_overtime_audit`| Kiểm tra vi phạm giới hạn làm thêm giờ của công nhân theo luật lao động | 4 GB RAM | `capsule-overtime-audit`|
| `0xC01B` | `skynet://supply.rfq.supplier_bid_rank_score` | Chấm điểm tự động các hồ sơ chào thầu theo tiêu chí kỹ thuật và giá cả | 4 GB RAM | `capsule-rfq-scorer` |
| `0xC01C` | `skynet://supply.risk.single_source_disruption`| Cảnh báo rủi ro đứt gãy nguồn cung khi phụ thuộc vào 1 nhà cung ứng duy nhất| 8 GB RAM | `capsule-supply-risk` |
| `0xC01D` | `skynet://supply.trace.carbon_footprint_scope3`| Tính toán lượng phát thải carbon phạm vi 3 dọc theo toàn bộ chuỗi cung ứng| 16 GB RAM | `capsule-scope3-carbon`|
| `0xC01E` | `skynet://supply.trace.conflict_minerals_smelter`| Truy vết nguồn gốc quặng thiếc, vonfram không đến từ vùng xung đột | 8 GB RAM | `capsule-conflict-min` |
| `0xC01F` | `skynet://logistics.empty.container_repositioning`| Tối ưu việc chuyển vỏ container rỗng từ cảng nhập siêu sang cảng xuất siêu| 16 GB RAM | `capsule-empty-repo` |
| `0xC020` | `skynet://logistics.bunker.fuel_speed_optimization`| Tính toán tốc độ chạy tàu biển kinh tế nhất theo giá dầu nhiên liệu | 4 GB RAM | `capsule-bunker-speed` |
| `0xC021` | `skynet://legal.realestate.title_deed_chain` | Truy vết lịch sử sang tên sổ đỏ qua 50 năm phát hiện tranh chấp | 8 GB RAM | `capsule-title-deed` |
| `0xC022` | `skynet://legal.antitrust.market_share_hhi` | Tính toán chỉ số tập trung thị trường HHI đánh giá nguy cơ độc quyền sáp nhập| 4 GB RAM | `capsule-hhi-antitrust`|
| `0xC023` | `skynet://legal.reg.basel_capital_adequacy_ratio`| Tính tỷ lệ an toàn vốn tối thiểu CAR theo chuẩn quy định ngân hàng | 8 GB RAM | `capsule-car-ratio` |
| `0xC024` | `skynet://legal.audit.payroll_ghost_employee_find`| Quét dữ liệu chấm công và mã số thuế phát hiện nhân viên ma nhận lương | 4 GB RAM | `capsule-ghost-employee`|
| `0xC025` | `skynet://legal.audit.procurement_split_po_alert`| Phát hiện thủ thuật chia nhỏ đơn hàng né hạn mức đấu thầu rộng rãi | 4 GB RAM | `capsule-split-po-chk` |
| `0xC026` | `skynet://logistics.crossdock.pallet_flow_match`| Lập kế hoạch trung chuyển trực tiếp Cross-docking không lưu kho | 8 GB RAM | `capsule-cross-docking`|
| `0xC027` | `skynet://logistics.reverse.return_routing_inspect`| Tối ưu quy trình thu hồi và phân loại hàng đổi trả thương mại điện tử | 8 GB RAM | `capsule-reverse-logist`|
| `0xC028` | `skynet://logistics.toll.electronic_toll_tollbooth`| Đối soát doanh thu trạm thu phí không dừng ETC và lưu lượng xe qua trạm | 8 GB RAM | `capsule-etc-toll-recon`|

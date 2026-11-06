# Phân Khu 08: Tài Chính, Định Lượng & DeFi (80 Opcodes)

Phân khu này định nghĩa 80 mã tác vụ chuyên biệt phục vụ các Quỹ đầu tư định lượng (Quant Funds), Ngân hàng thương mại, Công ty bảo hiểm, Cổng thanh toán và Sàn giao dịch phi tập trung (DeFi): từ Định giá quyền chọn phức tạp Monte Carlo 100M Path, Tối ưu hóa danh mục đầu tư Markowitz, Khớp lệnh lô phi tập trung chống MEV (CoW-Swap Solvers), Giám sát giao dịch chống rửa tiền (AML Graphs) cho đến Đối soát sao kê tài chính tự động.

---

## Danh Mục Chi Tiết 80 Opcodes Tài Chính & DeFi (`0x8001` – `0x8050`)

| Opcode | Định Danh MCP Tool URI | Tác Vụ Cụ Thể | Phần Cứng Tối Thiểu | Capsule Runtime |
| :---: | :--- | :--- | :---: | :--- |
| `0x8001` | `skynet://quant.option.monte_carlo_exotics_100m`| Định giá quyền chọn rào cản phức tạp qua 100 triệu kịch bản giả lập | 16 GB VRAM | `capsule-mc-options` |
| `0x8002` | `skynet://quant.option.black_scholes_pde` | Giải phương trình đạo hàm riêng Black-Scholes tính mặt cong biến động | 8 GB RAM | `capsule-black-scholes`|
| `0x8003` | `skynet://quant.risk.greeks_delta_gamma_vega` | Tính toán ma trận độ nhạy rủi ro Greeks (Delta, Gamma, Vega, Theta) | 8 GB RAM | `capsule-greeks-calc` |
| `0x8004` | `skynet://quant.risk.var_cvar_historical` | Tính toán Giá trị chịu rủi ro VaR và rủi ro đuôi CVaR mức tin cậy 99% | 16 GB RAM | `capsule-var-cvar` |
| `0x8005` | `skynet://quant.port.markowitz_efficient_frontier`| Tối ưu hóa tỷ trọng phân bổ tài sản đường biên hiệu quả Markowitz | 8 GB RAM | `capsule-markowitz-opt`|
| `0x8006` | `skynet://quant.port.black_litterman_views` | Kết hợp quan điểm chủ quan của chuyên gia vào phân bổ Black-Litterman| 8 GB RAM | `capsule-black-litter` |
| `0x8007` | `skynet://quant.port.hierarchical_risk_parity` | Phân bổ rủi ro phân cấp HRP dựa trên cấu trúc liên kết đồ thị cụm | 16 GB RAM | `capsule-hrp-portfolio`|
| `0x8008` | `skynet://quant.factor.fama_french_5factor` | Ước lượng hệ số hồi quy mô hình 5 nhân tố Fama-French cho 5.000 cổ phiếu | 16 GB RAM | `capsule-fama-french` |
| `0x8009` | `skynet://quant.calib.heston_stochastic_vol`| Hiệu chỉnh tham số mô hình biến động ngẫu nhiên Heston từ giá thị trường | 16 GB RAM | `capsule-heston-calib` |
| `0x800A` | `skynet://quant.calib.sabr_volatility_smile`| Khớp nụ cười biến động SABR cho thị trường lãi suất hoán đổi Swaptions | 8 GB RAM | `capsule-sabr-smile` |
| `0x800B` | `skynet://quant.rate.hull_white_yield_curve` | Mô phỏng đường cong lợi suất trái phiếu 1 nhân tố Hull-White | 8 GB RAM | `capsule-hull-white` |
| `0x800C` | `skynet://quant.credit.merton_structural_default`| Dự báo xác suất vỡ nợ doanh nghiệp dựa trên mô hình cấu trúc Merton | 8 GB RAM | `capsule-merton-default`|
| `0x800D` | `skynet://quant.credit.vasicek_portfolio_loss`| Mô phỏng phân phối tổn thất tín dụng của danh mục 100.000 khoản vay | 16 GB RAM | `capsule-vasicek-loss` |
| `0x800E` | `skynet://defi.batch.cow_swap_solver` | Tìm kiếm nghiệm hoán đổi vòng kín (Coincidence of Wants) chống MEV | 32 Core CPU | `capsule-cow-mip-solver`|
| `0x800F` | `skynet://defi.arb.cross_dex_bellman_ford` | Tìm chu trình sinh lời chênh lệch giá Arbitrage đa chuỗi thời gian thực | 16 GB RAM | `capsule-dex-arb-graph`|
| `0x8010` | `skynet://defi.amm.dynamic_fee_reinforcement`| Tự động tối ưu biểu phí giao dịch pool thanh khoản bằng học tăng cường | 12 GB VRAM | `capsule-amm-rl-fee` |
| `0x8011` | `skynet://defi.liq.lending_protocol_scan` | Quét hệ số an toàn Health Factor của 50.000 tài khoản vay tìm điểm thanh lý| 16 GB RAM | `capsule-liquidation-ai`|
| `0x8012` | `skynet://defi.opt.concentrated_liquidity_range`| Tối ưu dải giá cung cấp thanh khoản Uniswap v3 tối đa hóa phí thu được | 8 GB RAM | `capsule-uni-v3-opt` |
| `0x8013` | `skynet://defi.mev.sandwich_attack_simulator`| Giả lập kịch bản tấn công kẹp bánh mì để bảo vệ người dùng trượt giá | 16 GB RAM | `capsule-mev-simulator`|
| `0x8014` | `skynet://defi.oracle.twap_outlier_rejection` | Tính giá trung bình gia quyền thời gian TWAP loại bỏ râu nến thao túng | 4 GB RAM | `capsule-twap-oracle` |
| `0x8015` | `skynet://fin.aml.transaction_graph_peeling` | Bóc tách mạng lưới rửa tiền bóc tách chuỗi giao dịch Peeling Chain | 32 GB RAM | `capsule-aml-graph` |
| `0x8016` | `skynet://fin.aml.sanction_list_fuzzy_match` | Quét tên khách hàng đối chiếu danh sách trừng phạt OFAC/Interpol | 8 GB RAM | `capsule-ofac-screener`|
| `0x8017` | `skynet://fin.fraud.credit_card_xgb_infer` | Phát hiện giao dịch cà thẻ tín dụng gian lận bằng XGBoost (<10ms) | 8 GB RAM | `capsule-fraud-xgb` |
| `0x8018` | `skynet://fin.score.credit_score_woe_iv` | Tính điểm rủi ro tín dụng khách hàng cá nhân bằng kỹ thuật WoE & IV | 8 GB RAM | `capsule-woe-credit` |
| `0x8019` | `skynet://fin.recon.bank_statement_double_entry`| Tự động đối soát 1 triệu giao dịch sao kê ngân hàng với sổ cái kế toán | 16 GB RAM | `capsule-recon-fin` |
| `0x801A` | `skynet://fin.fx.cross_currency_triangular_arb`| Dò tìm chênh lệch tỷ giá tam giác giữa 30 cặp ngoại tệ thế giới | 8 GB RAM | `capsule-triangular-fx`|
| `0x801B` | `skynet://fin.hft.orderbook_lob_imbalance` | Tính toán chỉ số mất cân bằng sổ lệnh LOB dự báo hướng giá micro | 8 GB RAM | `capsule-lob-imbalance`|
| `0x801C` | `skynet://fin.hft.volume_synchronized_vpin` | Đo xác suất giao dịch nội gián VPIN phát hiện biến động ngầm thị trường | 8 GB RAM | `capsule-vpin-flow` |
| `0x801D` | `skynet://fin.insur.glm_frequency_severity` | Định phí bảo hiểm xe cơ giới bằng mô hình tuyến tính tổng quát GLM | 16 GB RAM | `capsule-actuary-glm` |
| `0x801E` | `skynet://fin.insur.chain_ladder_reserving`| Trích lập quỹ dự phòng bồi thường bảo hiểm theo phương pháp Chain Ladder| 8 GB RAM | `capsule-chain-ladder` |
| `0x801F` | `skynet://fin.tax.crypto_fifo_capital_gain`| Tính toán thuế thu nhập thặng dư vốn giao dịch crypto theo nguyên tắc FIFO| 16 GB RAM | `capsule-tax-fifo` |
| `0x8020` | `skynet://fin.bond.macaulay_convexity_duration`| Tính thời lượng Macaulay và độ lồi của danh mục trái phiếu chính phủ | 4 GB RAM | `capsule-bond-duration`|
| `0x8021` | `skynet://quant.stress.basel3_liquidity_lcr` | Kiểm tra sức chịu đựng tỷ lệ đảm bảo thanh khoản LCR chuẩn Basel III | 16 GB RAM | `capsule-basel3-stress`|
| `0x8022` | `skynet://quant.stress.reverse_stress_testing` | Tìm kịch bản kinh tế vĩ mô tồi tệ nhất có thể làm vỡ nợ tổ chức | 16 GB RAM | `capsule-reverse-stress`|
| `0x8023` | `skynet://quant.corr.denoised_marchenko_pastur`| Khử nhiễu ma trận tương quan tài sản bằng định lý Marchenko-Pastur | 8 GB RAM | `capsule-rmt-corr` |
| `0x8024` | `skynet://quant.cointegration.johansen_pairs`| Kiểm định đồng tích hợp Johansen tìm cặp cổ phiếu giao dịch Pairs Trading| 8 GB RAM | `capsule-johansen-pair`|
| `0x8025` | `skynet://quant.ml.reinforcement_trading_agent`| Chạy mô hình học tăng cường PPO tự động đặt lệnh mua bán chứng khoán | 12 GB VRAM | `capsule-rl-trader` |
| `0x8026` | `skynet://quant.micro.kyle_lambda_liquidity`| Đo lường độ sâu thanh khoản thị trường và tác động giá Kyle's Lambda | 8 GB RAM | `capsule-kyle-lambda` |
| `0x8027` | `skynet://defi.staking.slashing_risk_simulator`| Mô phỏng rủi ro bị phạt mất tiền Slashing khi tham gia Validator PoS | 8 GB RAM | `capsule-slash-sim` |
| `0x8028` | `skynet://defi.bridge.liquidity_rebalance_flow`| Tính toán luồng tái cân bằng tiền mặt giữa các đầu cầu nối thanh toán | 8 GB RAM | `capsule-bridge-rebal` |
| `0x8029` | `skynet://defi.perpetual.funding_rate_optimizer`| Điều chỉnh tỷ lệ lãi suất tài trợ Funding Rate cân bằng vị thế Long/Short| 8 GB RAM | `capsule-funding-rate` |
| `0x802A` | `skynet://defi.vault.auto_compound_yield_apr`| Tối ưu hóa chu kỳ gặt hái lãi kép Auto-Compound tối đa hóa APR thực tế| 4 GB RAM | `capsule-vault-compound`|
| `0x802B` | `skynet://fin.pos.merchant_chargeback_predict`| Dự báo khả năng người mua khiếu nại đòi tiền bồi hoàn Chargeback | 8 GB RAM | `capsule-chargeback-ai`|
| `0x802C` | `skynet://fin.loan.peer_to_peer_match_order`| Ghép nối tối ưu người vay và người cho vay P2P theo khẩu vị rủi ro | 8 GB RAM | `capsule-p2p-lending` |
| `0x802D` | `skynet://fin.invoice.factoring_risk_discount`| Định giá chiết khấu mua lại khoản phải thu hóa đơn doanh nghiệp | 4 GB RAM | `capsule-factoring-risk`|
| `0x802E` | `skynet://fin.wealth.robo_advisor_glidepath`| Tự động dịch chuyển tỷ trọng cổ phiếu sang trái phiếu theo tuổi nghỉ hưu| 4 GB RAM | `capsule-robo-advisor` |
| `0x802F` | `skynet://fin.macro.dsge_economic_equilibrium`| Mô phỏng cân bằng tổng thể ngẫu nhiên động DSGE dự báo lạm phát và lãi suất| 32 GB RAM | `capsule-dsge-macro` |
| `0x8030` | `skynet://fin.macro.yield_curve_nelson_siegel`| Khớp đường cong cấu trúc kỳ hạn lãi suất theo mô hình Nelson-Siegel | 4 GB RAM | `capsule-nelson-siegel`|
| `0x8031` | `skynet://fin.commod.storage_convenience_yield`| Tính toán chi phí lưu kho và lợi suất tiện ích thị trường hàng hóa phái sinh| 4 GB RAM | `capsule-commodity-yield`|
| `0x8032` | `skynet://fin.commod.crack_spread_refinery`| Tính toán biên lợi nhuận lọc dầu Crack Spread giữa dầu thô và xăng/dầu | 4 GB RAM | `capsule-crack-spread` |
| `0x8033` | `skynet://fin.carbon.emission_credit_pricing`| Định giá tín chỉ carbon bù trừ phát thải khí nhà kính doanh nghiệp | 8 GB RAM | `capsule-carbon-pricing`|
| `0x8034` | `skynet://fin.audit.benford_law_fraud_check`| Kiểm tra quy luật Benford phát hiện số liệu sổ sách kế toán làm giả | 4 GB RAM | `capsule-benford-audit` |
| `0x8035` | `skynet://fin.audit.audit_sampling_mus` | Rút mẫu kiểm toán theo đơn vị tiền tệ MUS đảm bảo tính đại diện | 4 GB RAM | `capsule-mus-sampling` |
| `0x8036` | `skynet://fin.audit.variance_budget_actual` | Tự động phân tích nguyên nhân chênh lệch giữa dự toán và thực tế chi tiêu| 4 GB RAM | `capsule-budget-var` |
| `0x8037` | `skynet://fin.esg.esg_score_corporate_nlp` | Chấm điểm trách nhiệm xã hội và môi trường ESG từ báo cáo thường niên | 12 GB RAM | `capsule-esg-scorer` |
| `0x8038` | `skynet://fin.fund.nav_per_share_calc` | Tính toán giá trị tài sản ròng trên mỗi chứng chỉ quỹ NAV/Share cuối ngày| 8 GB RAM | `capsule-fund-nav` |
| `0x8039` | `skynet://fin.fund.attribution_brinson_hood` | Phân tích đóng góp hiệu quả đầu tư Brinson: Chọn ngành vs Chọn cổ phiếu | 8 GB RAM | `capsule-brinson-attrib`|
| `0x803A` | `skynet://fin.fund.tracking_error_etf` | Đo độ lệch bám sát (Tracking Error) của quỹ hoán đổi danh mục ETF | 4 GB RAM | `capsule-etf-tracking` |
| `0x803B` | `skynet://fin.mna.dcf_valuation_sensitivity` | Định giá doanh nghiệp theo chiết khấu dòng tiền DCF và phân tích độ nhạy| 8 GB RAM | `capsule-dcf-valuation` |
| `0x803C` | `skynet://fin.mna.lbo_debt_waterfall_repay` | Lập mô hình hoàn nợ phân tầng Waterfall cho thương vụ mua lại bằng vốn vay| 8 GB RAM | `capsule-lbo-model` |
| `0x803D` | `skynet://fin.realestate.hedonic_price_index`| Xây dựng chỉ số giá bất động sản theo phương pháp đặc tính Hedonic | 8 GB RAM | `capsule-hedonic-prop` |
| `0x803E` | `skynet://fin.realestate.cap_rate_noi_calc` | Tính toán tỷ suất sinh lời Cap Rate và thu nhập hoạt động thuần NOI | 4 GB RAM | `capsule-realestate-noi`|
| `0x803F` | `skynet://fin.crypto.utxo_dust_consolidation`| Lập kế hoạch gom các mảnh UTXO nhỏ lẻ tiết kiệm phí chuyển tiền tương lai| 8 GB RAM | `capsule-dust-combine` |
| `0x8040` | `skynet://fin.crypto.mpc_wallet_policy_eval`| Kiểm tra chính sách hạn mức chi tiêu ví đa chữ ký trước khi giải ngân | 4 GB RAM | `capsule-mpc-policy` |
| `0x8041` | `skynet://fin.crypto.impermanent_loss_hedge`| Tính toán quyền chọn bảo hiểm tổn thất tạm thời cho nhà cung cấp thanh khoản| 8 GB RAM | `capsule-il-hedge` |
| `0x8042` | `skynet://fin.crypto.staking_apr_inflation_adj`| Tính toán lợi suất Staking thực tế sau khi đã trừ tỷ lệ lạm phát token| 2 GB RAM | `capsule-real-apr` |
| `0x8043` | `skynet://fin.crypto.cross_chain_atomic_swap` | Xác thực khóa thời gian và băm bí mật cho giao dịch hoán đổi nguyên tử | 4 GB RAM | `capsule-atomic-swap` |
| `0x8044` | `skynet://fin.pay.interchange_fee_optimization`| Tự động chọn cổng thanh toán có mức phí trung gian Interchange rẻ nhất | 4 GB RAM | `capsule-pay-router` |
| `0x8045` | `skynet://fin.pay.qr_emvco_payload_generate`| Sinh mã QR thanh toán chuẩn quốc tế EMVCo động tích hợp số tiền | 2 GB RAM | `capsule-emvco-qr` |
| `0x8046` | `skynet://fin.settle.swift_mt103_iso20022_map`| Chuyển đổi điện báo ngân hàng SWIFT MT103 sang chuẩn mới ISO 20022 XML | 4 GB RAM | `capsule-iso20022` |
| `0x8047` | `skynet://fin.settle.rtgs_liquidity_netting` | Bù trừ đa phương ròng theo lô tiết kiệm vốn thanh toán liên ngân hàng | 16 GB RAM | `capsule-rtgs-netting` |
| `0x8048` | `skynet://fin.fx.cross_border_fx_forward_hedge`| Tính toán hợp đồng kỳ hạn ngoại hối phòng hộ rủi ro biến động tỷ giá | 4 GB RAM | `capsule-fx-forward` |
| `0x8049` | `skynet://fin.macro.inflation_nowcasting_groceries`| Dự báo lạm phát tức thì từ dữ liệu giá cả giỏ hàng siêu thị trực tuyến | 8 GB RAM | `capsule-cpi-nowcast` |
| `0x804A` | `skynet://fin.macro.taylor_rule_policy_rate`| Ước lượng lãi suất mục tiêu của Ngân hàng Trung ương theo quy tắc Taylor | 4 GB RAM | `capsule-taylor-rule` |
| `0x804B` | `skynet://fin.macro.sovereign_debt_sustainability`| Đánh giá tính bền vững của nợ công quốc gia theo kịch bản tăng trưởng GDP| 8 GB RAM | `capsule-debt-sustain` |
| `0x804C` | `skynet://quant.sentiment.news_volume_spike`| Phát hiện đột biến số lượng tin tức truyền thông cảnh báo thiên nga đen | 8 GB RAM | `capsule-news-spike` |
| `0x804D` | `skynet://quant.sentiment.social_cashtag_buzz`| Đo lường mức độ bàn luận mã chứng khoán (Cashtags) trên mạng xã hội | 8 GB RAM | `capsule-cashtag-buzz` |
| `0x804E` | `skynet://quant.ml.feature_importance_shapley`| Giải thích đóng góp của từng chỉ số vào quyết định đầu tư bằng giá trị SHAP| 16 GB RAM | `capsule-shap-finance` |
| `0x804F` | `skynet://quant.ml.fractional_differentiation`| Lấy vi phân bậc phân số chuỗi giá cổ phiếu giữ nguyên trí nhớ dài hạn | 8 GB RAM | `capsule-frac-diff` |
| `0x8050` | `skynet://quant.stat.hurst_exponent_mean_revert`| Tính toán số mũ Hurst phân loại thị trường có xu hướng hay đảo chiều hồi quy| 4 GB RAM | `capsule-hurst-calc` |

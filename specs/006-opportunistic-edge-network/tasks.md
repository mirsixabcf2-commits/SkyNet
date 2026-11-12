# Tasks: Mạng Lưới AI Cận Biên Tận Dụng GPU/CPU Nhàn Rỗi, Huấn Luyện Vi Mô Phân Tán & Kinh Tế P2P Thời Gian Thực

**Input**: Tài liệu thiết kế từ `specs/006-opportunistic-edge-network/` ([spec.md](./spec.md), [plan.md](./plan.md))

## Format: `- [ ] T### [P?] [US#?] Mô tả hành động cụ thể kèm đường dẫn tệp chính xác`

- **`[P]`**: Chạy song song an toàn (100% khác tệp nguồn, không phụ thuộc vào bất kỳ tác vụ chưa hoàn thành nào trong cùng Phase)
- **`[US#]`**: Ánh xạ trực tiếp tới User Story trong `spec.md` (`[US1]` – `[US5]`)
- **Ràng buộc Bắt buộc**: `0` crate ngoài mới trong `Cargo.toml`; mọi tệp `.rs` chạm tới `<= 250` dòng mục tiêu (`350` dòng trần cứng); mọi hàm `<= 40` dòng, `<= 4` tham số, độ lồng `<= 3`; `Result<T, E>` (`thiserror`), cấm `.unwrap()` runtime, cấm tạo `utils.rs`/`helpers.rs`.

---

## Phase 1: Setup & Prerequisite Modular Refactoring (Zero Functional Change)

**Goal**: Tách toàn bộ 11 tệp hiện hữu `>= 200` dòng cần chạm tới thành các submodule đơn trách nhiệm `<= 230` dòng/tệp mà không thay đổi bất kỳ API công khai hay hành vi nào (`70 passed, 0 failed` giữ nguyên 100%).

- [x] T001 [P] Tách `crates/skynet-dna/src/lib.rs` (481 dòng) thành `crates/skynet-dna/src/genes.rs`, `crates/skynet-dna/src/cell_dna.rs`, `crates/skynet-dna/src/dna_tests.rs` và giữ re-export tương thích ngược tại `crates/skynet-dna/src/lib.rs` (`<= 210` dòng/tệp).
- [x] T002 [P] Tách `crates/skynet-cell/src/lib.rs` (414 dòng) thành `crates/skynet-cell/src/hardware.rs`, `crates/skynet-cell/src/micro_cell.rs`, `crates/skynet-cell/src/cell_tests.rs` và re-export tại `crates/skynet-cell/src/lib.rs` (`<= 190` dòng/tệp).
- [x] T003 [P] Tách `crates/skynet-ledger/src/lib.rs` (1639 dòng) và `crates/skynet-ledger/src/compute_escrow.rs` (324 dòng, mở `pub(crate) records` trên `ComputeEscrowVault`) thành `crates/skynet-ledger/src/{nano_skn.rs,halving_vault.rs,custody.rs,voucher.rs,utreexo.rs,dag_ledger.rs,ledger_core_tests.rs,ledger_custody_tests.rs,compute_escrow.rs,compute_escrow_refund.rs}` cùng re-export tại `crates/skynet-ledger/src/lib.rs` (`<= 230` dòng/tệp).
- [x] T004 [P] Tách `crates/skynet-wire/src/lib.rs` (381 dòng), `crates/skynet-wire/src/opcode_registry.rs` (233 dòng) và `crates/skynet-wire/src/mcp_capsule.rs` (209 dòng) thành `crates/skynet-wire/src/{packet.rs,wire_tests.rs,opcode_domains.rs,opcode_registry.rs,mcp_capsule.rs,mcp_capsule_tests.rs}` và re-export tại `crates/skynet-wire/src/lib.rs` (`<= 195` dòng/tệp).
- [x] T005 [P] Tách `crates/skynet-agent/src/lib.rs` (1105 dòng) thành `crates/skynet-agent/src/{snowball.rs,organism_state.rs,datagram_router.rs,agent_core_tests.rs,agent_network_tests.rs}` và re-export tại `crates/skynet-agent/src/lib.rs` (`<= 225` dòng/tệp).
- [x] T006 [P] Tách `crates/skynet-node/src/bin/skynet.rs` (224 dòng) thành `crates/skynet-node/src/cli/{mod.rs,wallet_opcode_cmd.rs}` (re-export tại `crates/skynet-node/src/lib.rs`) và bộ điều phối mỏng tại `crates/skynet-node/src/bin/skynet.rs` (`<= 160` dòng/tệp, bảo toàn 100% chuỗi CLI hiện hữu).
- [x] T007 [P] Tách `crates/skynet-sim/src/main.rs` (407 dòng) thành `crates/skynet-sim/src/{classic_sim.rs,sim_metrics.rs,main.rs}` (`<= 220` dòng/tệp).
- [x] T008 [P] Tách `crates/skynet-dashboard/src/main.rs` (941 dòng) thành `crates/skynet-dashboard/src/{lib.rs,dto.rs,live_world.rs,world_actions.rs,http_router.rs,main.rs}` (giữ `preprocess_api.rs` được re-export qua `crates/skynet-dashboard/src/lib.rs` để dùng chung cho cả binary và integration test in-process, `<= 220` dòng/tệp).
- [x] T009 Chạy kiểm chứng hồi quy sau phân rã `cargo test --workspace` trên gốc workspace `Cargo.toml` để xác nhận toàn bộ `70` tests gốc pass 100% và mọi tệp vừa tách `<= 230` dòng (`SC-008`).

**Checkpoint**: Nền tảng mã nguồn sạch, 100% tệp cần mở rộng đã dưới ngưỡng `230` dòng, sẵn sàng phát triển tính năng song song.

---

## Phase 2: Foundational (Shared Blocking Prerequisites)

**Goal**: Xây dựng các nguyên thủy dùng chung xuyên suốt các User Story (tín hiệu ngắt lát cắt nguyên tử, vi điểm dừng, nén gradient `f16` và đăng ký miền mã lệnh `0x13`).

- [x] T010 [P] Hiện thực `PreemptSignal` (`AtomicU8`: `Run`, `Throttle50`, `PreemptImmediate`), `PreemptGuard`, `SliceLatencyGuard` (ngắt khi `actual_micros > 18 * ema_micros / 10` trong `< 20ms`) và `MicroTrainCheckpoint` kèm khối `#[cfg(test)]` nội bộ tại `crates/skynet-cell/src/preempt_guard.rs` (`FR-005`, `SC-001`).
- [x] T011 [P] Hiện thực bộ nén chênh lệch trọng số `half::f16` (`encode_dense_f16`, `decode_dense_f16`), `SparseTopKEntry` và hàm cắt chuẩn L2 `clip_l2_norm` kèm khối `#[cfg(test)]` nội bộ tại `crates/skynet-cell/src/gradient_codec.rs` (`FR-006`).
- [x] T012 [P] Bổ sung `OpcodeDomain::MicroTrainEvolution = 0x13` và đăng ký miền `0x1300` (`0x1301` `micro_train.mlp_step`, `0x1302` `micro_train.lora_adapter`, `0x1303` `evolution.plasmid_guided`) kèm unit tests tại `crates/skynet-wire/src/mcp_capsule.rs`, `crates/skynet-wire/src/opcode_domains.rs` và `crates/skynet-wire/src/opcode_registry.rs` (`FR-007`).
- [x] T013 Khai báo `pub mod preempt_guard; pub mod gradient_codec;` cùng re-export tại `crates/skynet-cell/src/lib.rs`, chạy `cargo test -p skynet-cell -p skynet-wire` xác nhận toàn bộ unit tests của Phase 2 biên dịch và pass 100% (`FR-005`, `FR-006`, `FR-007`).

**Checkpoint**: Các nguyên thủy cấp thấp tại `skynet-cell` và `skynet-wire` đã sẵn sàng cho các Phase nghiệp vụ `US1`–`US5`.

---

## Phase 3: User Story 1 - Khởi Nguyên Sổ Cái `GENESIS_HASH` & Kết Nối P2P Xuyên NAT Đa Thiết Bị (Priority: P1 — MVP)

**Goal**: Hoàn tất trọn vẹn nghi thức khởi nguyên `genesis.json` (`21T SKN`: `85%` Vault + `15%` Lưu thông), màng lọc tế bào Da `< 5µs` không cấp phát heap, khám phá LAN `< 1s`, đục lỗ NAT UDP đồng thời qua Seed và chuyển tiếp dự phòng `GangliaRelay`.  
**Independent Test**: Chạy `cargo test -p skynet-ledger genesis`, `cargo test -p skynet-wire rendezvous`, và `cargo test -p skynet-node --test genesis_cli_test --test p2p_rendezvous_hole_punch_test`.

- [x] T014 [P] [US1] Hiện thực `GenesisConfig`, `GenesisAllocation`, `GenesisUtxoEntry`, `default_mainnet_genesis`, `compute_genesis_hash` (BLAKE3 32-byte `GENESIS_HASH`) và `validate_genesis_invariants` (`85%` `17.85T SKN` Vault + `15%` `3.15T SKN` Circulating == `21,000,000,000,000 SKN` chính xác tới `1 nanoSKN`) kèm unit tests tại `crates/skynet-ledger/src/genesis.rs` và re-export tại `crates/skynet-ledger/src/lib.rs` (`FR-001`, `SC-005`).
- [x] T015 [P] [US1] Hiện thực `EpidermalHeader` và `EpidermalFilter::verify_stack_header` kiểm tra `network_id == GENESIS_HASH` và chống phát lại `±30 giây` (`MAX_TIMESTAMP_DRIFT_SEC = 30`) trong `< 5µs` với `0` cấp phát heap cho gói lỗi kèm khối `#[cfg(test)]` nội bộ tại `crates/skynet-wire/src/epidermal_filter.rs` (`FR-002`, `SC-002`).
- [x] T016 [P] [US1] Hiện thực `RendezvousMessage` (`PheromoneHello`, `PunchRendezvous` `<= 300B`, `PunchProbe`, `LanBeacon`, `GangliaRelayFrame` `<= 1200B`) kèm ký/xác thực Ed25519 và khối `#[cfg(test)]` kích thước MTU tại `crates/skynet-wire/src/rendezvous_wire.rs` (`FR-003`, `SC-002`).
- [x] T017 [US1] Gắn biến thể `ImpulseBody::Rendezvous(RendezvousMessage)` vào `crates/skynet-wire/src/packet.rs` và re-export `epidermal_filter`, `rendezvous_wire` tại `crates/skynet-wire/src/lib.rs` để kích hoạt biên dịch và pass unit tests `skynet-wire` (`FR-002`, `FR-003`).
- [x] T018 [US1] Hiện thực `RendezvousRegistry` (`RENDEZVOUS_TTL_SEC = 120`), ghép cặp hai nút sau NAT sinh `PunchRendezvous`, sinh loạt `4` xung `PunchProbe` đồng thời, xử lý `LanBeacon` và định tuyến `GangliaRelayFrame` kèm unit tests tại `crates/skynet-agent/src/rendezvous_broker.rs` và re-export tại `crates/skynet-agent/src/lib.rs` (`FR-003`, `SC-006`).
- [x] T019 [US1] Hiện thực đọc/ghi JSON chuẩn hóa `genesis.json` (`load_genesis_file`, `write_genesis_file`) tại `crates/skynet-node/src/genesis_io.rs` và bổ sung cờ `--genesis <PATH>` vào `NodeConfig` tại `crates/skynet-node/src/config.rs` (`FR-001`, `FR-011`).
- [x] T020 [US1] Hiện thực tác vụ mạng P2P thực tế (lọc `EpidermalFilter`, ping `PheromoneHello` mỗi `25s` tới `seed_peers`, bắn xung đục lỗ khi nhận `PunchRendezvous`, phát `LanBeacon` `< 1s`, nạp UTXO gốc từ `genesis.json`) tại `crates/skynet-node/src/runtime_p2p.rs` và tích hợp vào `crates/skynet-node/src/runtime.rs` + `crates/skynet-node/src/lib.rs` (`FR-002`, `FR-003`, `SC-006`).
- [x] T021 [US1] Hiện thực nhánh lệnh `skynet genesis generate --output <PATH> [--network <NAME>]`, `skynet genesis verify --path <PATH>` và nâng cấp `skynet airdrop claim` chạy `PopeChallenge` + `MarketingBootstrapVault` thật tại `crates/skynet-node/src/cli/genesis_airdrop_cmd.rs`, nối vào `crates/skynet-node/src/cli/mod.rs` và `crates/skynet-node/src/bin/skynet.rs` (`FR-001`, `FR-011`).
- [x] T022 [US1] Viết integration test kiểm chứng trọn vẹn vòng đời CLI `genesis generate` -> `genesis verify` -> từ chối tệp lệch `1 nanoSKN` tại `crates/skynet-node/tests/genesis_cli_test.rs` và mô phỏng 1 Seed Node + 2 NAT Worker Nodes thực hiện `PheromoneHello` -> `PunchRendezvous` -> đục lỗ UDP `< 3s`, LAN `< 1s`, `GangliaRelay` & chặn gói sai `GENESIS_HASH` tại `crates/skynet-node/tests/p2p_rendezvous_hole_punch_test.rs` (`FR-001`, `FR-002`, `FR-003`, `SC-002`, `SC-005`, `SC-006`).

**Checkpoint**: User Story 1 hoàn tất độc lập — nhiều thiết bị thật có thể chung sổ cái `GENESIS_HASH` và kết nối P2P xuyên NAT qua UDP.

---

## Phase 4: User Story 2 - Bộ Điều Tốc Cơ Hội Không Độ Trễ Cho Game Thủ & Chủ Máy (Priority: P1 — MVP)

**Goal**: Giám sát phần cứng đa nền tảng không cần quyền `root`, nhường GPU/CPU cho Game thủ trong `< 100ms` SLA (`< 25ms` mục tiêu), ghi nguyên tử `Micro-Checkpoint` `< 5ms` và chống nhấp nháy bằng dải trễ nhiệt `8°C` + cửa sổ nhàn rỗi `30s`.  
**Independent Test**: Chạy `cargo test -p skynet-agent governor_fsm` và `cargo test -p skynet-node --test opportunistic_governor_test`.

- [x] T023 [US2] Hiện thực trước kiểu tiền đề `GovernorConfig` (`temp_limit_c = 78`, `hysteresis_cool_c = 8`, `idle_hold_secs = 30`, `max_vram_pct = 50`, `battery_guard = true`, `schedule_window`), `SystemTelemetrySnapshot` và máy trạng thái `GovernorState` (`DeepIdle`, `LightIdle`, `Throttled`, `PausedGaming`, `PausedThermal`, `PausedBattery`, `PausedVram`, `PausedSchedule`) kèm unit tests tại `crates/skynet-agent/src/governor_fsm.rs` và re-export tại `crates/skynet-agent/src/lib.rs` (`FR-005`, `SC-001`).
- [x] T024 [P] [US2] Hiện thực bộ thu thập cảm biến macOS không cần `root` (`ioreg -r -d 1 -w 0 -c IOAccelerator`, `pmset -g batt`, `pmset -g therm`, `sysctl`, `lsappinfo front`/`ps` đối chiếu danh sách Game, xuất `SystemTelemetrySnapshot`) tại `crates/skynet-node/src/governor/probe_macos.rs` (`FR-004`).
- [x] T025 [P] [US2] Hiện thực bộ thu thập cảm biến Linux tốc độ cao (`/proc/stat`, `/proc/meminfo`, `/sys/class/thermal`, `/sys/class/power_supply`, `/sys/class/drm`, `nvidia-smi` tùy chọn, `/proc/[pid]/comm` đối chiếu danh sách Game, xuất `SystemTelemetrySnapshot`) tại `crates/skynet-node/src/governor/probe_linux.rs` (`FR-004`).
- [x] T026 [P] [US2] Hiện thực bộ thu thập cảm biến Windows (`nvidia-smi` CSV, `tasklist /FO CSV /NH` đối chiếu danh sách Game, trạng thái pin) và `SimulatedTelemetryProbe` dự phòng đa nền tảng tại `crates/skynet-node/src/governor/probe_windows.rs` (`FR-004`).
- [x] T027 [P] [US2] Hiện thực kho lưu trữ nguyên tử `.tmp -> rename` (`< 5ms`) cho `MicroTrainCheckpoint` và xuất bản chụp `worker_studio_state.json` tại `crates/skynet-node/src/governor/checkpoint_store.rs` (`FR-005`, `SC-001`).
- [x] T028 [US2] Hiện thực bộ điều phối giám sát thích nghi `OsTelemetryGovernor` (`250ms` khi hoạt động / `1500ms` khi tạm dừng qua `tokio::task::spawn_blocking`, kết nối các probe `T024`–`T027` và kích hoạt `PreemptGuard` tức thì) tại `crates/skynet-node/src/governor/mod.rs` và re-export tại `crates/skynet-node/src/lib.rs` (`FR-004`, `FR-005`, `SC-001`).
- [x] T029 [US2] Viết integration test đo thực nghiệm độ trễ ngắt `< 100ms` khi bật Game/quá nhiệt `78°C`/rút sạc, ghi nguyên tử `MicroTrainCheckpoint` `< 5ms`, chống bật lại ở `75°C` hoặc `< 30s` và nối lại chính xác khi `<= 70°C` + `>= 30s` tại `crates/skynet-node/tests/opportunistic_governor_test.rs` (`FR-004`, `FR-005`, `SC-001`).

**Checkpoint**: User Story 2 hoàn tất độc lập — bộ điều tốc bảo vệ tuyệt đối trải nghiệm Game thủ (`< 100ms`) và bảo toàn trạng thái tính toán.

---

## Phase 5: User Story 3 - Huấn Luyện Vi Mô Phân Tán, Tiến Hóa Gen & Kiểm Chứng PoUW Cắt Cọc (Priority: P1)

**Goal**: Huấn luyện nơ-ron nhiều bước trên `candle-core` có hỗ trợ tạm dừng/nối lại, lai ghép `PlasmidPatch`, truyền phân mảnh qua `FountainDroplet` (`<= 1200B`, chịu mất gói `40%`), kiểm chứng bẫy `Canary Spot-Check` `O(1)` kèm cắt cọc gian lận và tổng hợp `TrimmedMeanAggregator`.  
**Independent Test**: Chạy `cargo test -p skynet-dna`, `cargo test -p skynet-cell`, `cargo test -p skynet-ledger slash`, và `cargo test -p skynet-wire micro_train`.

- [x] T030 [P] [US3] Hiện thực `PlasmidPatch`, kiểm chứng mã băm `BLAKE3` `"SKYNET_PLASMID_PATCH_V1"` và hàm lai ghép tiến hóa có hướng `apply_plasmid_patch` kèm unit tests tại `crates/skynet-dna/src/plasmid_patch.rs`, re-export tại `crates/skynet-dna/src/lib.rs` (`FR-006`).
- [x] T031 [P] [US3] Hiện thực `MicroTrainConfig`, `MicroTrainBatch`, `MicroTrainResult` và động cơ `execute_micro_train_slices` dùng `candle_core::{Var, Tensor, Device}` + `.backward()` chạy `inner_steps` (kiểm tra `PreemptGuard` giữa mỗi bước để trả về `Preempted(MicroTrainCheckpoint)` hoặc `Completed(MicroTrainResult)` kèm `compute_canonical_digest`) và unit tests tại `crates/skynet-cell/src/micro_train.rs`, khai báo `pub mod micro_train;` tại `crates/skynet-cell/src/lib.rs` (`FR-005`, `FR-006`, `SC-001`, `SC-003`).
- [x] T032 [P] [US3] Bổ sung `EscrowStatus::Slashed` và mã lỗi bằng chứng gian lận vào `crates/skynet-ledger/src/compute_escrow_types.rs` (dùng kiểu `TaskId` chuẩn); hiện thực `FraudProofClaim`, `WorkerReputation` (`pouw_score_bps`) và `ComputeEscrowVault::slash_on_fraud_proof` (cắt `100%` `bond`: chia `50%` thưởng Auditor + hoàn `100%` bounty và `50%` bond cho Client, hạ `pouw_score_bps = 0`) kèm unit tests bảo toàn `NanoSkn` tại `crates/skynet-ledger/src/escrow_slash.rs` và re-export tại `crates/skynet-ledger/src/lib.rs` (`FR-008`, `SC-004`, `SC-005`).
- [x] T033 [P] [US3] Hiện thực các bản tin `MicroTrainWireMessage` (`MicroBatchManifest`, `WeightDeltaClaim`, `CanaryFraudReport`, `ChunkVoucherDelivery` `<= 1200B`, phân mảnh shard `<= 128 KB` qua `FountainDroplet`) kèm unit tests khôi phục `100%` dưới mức rớt gói `40%` tại `crates/skynet-wire/src/micro_train_wire.rs`, gắn `ImpulseBody::MicroTrain` vào `crates/skynet-wire/src/packet.rs` và re-export tại `crates/skynet-wire/src/lib.rs` (`FR-007`, `SC-002`, `SC-003`).
- [x] T034 [US3] Hiện thực bộ sinh bẫy tất định `generate_canary_batch` (`BLAKE3` XOF `"SKYNET_POUW_CANARY_V1"`, mặc định `canary_rate_bps = 1000`) và bộ thẩm định `verify_canary_spot_check` (`!NaN/Inf`, giảm loss, chuẩn L2, `Cosine Similarity >= 0.985` trên `MicroTrainResult` từ `T031`) kèm unit tests tại `crates/skynet-cell/src/canary_pouw.rs`, khai báo `pub mod canary_pouw;` tại `crates/skynet-cell/src/lib.rs` (`FR-008`, `SC-004`).
- [x] T035 [US3] Hiện thực `TrimmedMeanAggregator` cắt bỏ `10%` cực trị hai đầu trên từng tọa độ (`trim_ratio_bps = 1000`), nhân trọng số `pouw_score_bps` và xuất `PlasmidPatch` (`T030`) kèm unit tests chống `30%` nút Byzantine tại `crates/skynet-cell/src/federated_agg.rs`, hoàn thiện `pub mod federated_agg;` và re-export đầy đủ tại `crates/skynet-cell/src/lib.rs` (`FR-006`, `FR-008`, `SC-003`).

**Checkpoint**: User Story 3 hoàn tất ở tầng cốt lõi — huấn luyện vi mô, truyền tải phân mảnh chịu mất gói 40%, bẫy PoUW Canary và cắt cọc hoạt động đầy đủ.

---

## Phase 6: User Story 4 - Đa Hợp Tác Vụ 3 Nấc, Thanh Toán Streaming Từng Lát & Tự Động Quét Lãi Về Ví Lạnh (Priority: P2)

**Goal**: Khai thác tối đa thời gian nhàn rỗi theo 3 nấc (`DeepIdle` / `LightIdle` / `Paused*`), kiểm tra lợi nhuận điện năng, giải ngân từng lát hoàn thành qua `SynapticVoucher` lũy kế và tự động quét lãi ròng `Yield Sweep` về Ví Lạnh chủ sở hữu.  
**Independent Test**: Chạy `cargo test -p skynet-ledger streaming_escrow yield_sweep`, `cargo test -p skynet-agent`, và `cargo run -p skynet-sim`.

- [x] T036 [P] [US4] Mở rộng kiểu lỗi/trạng thái cột mốc lát cắt trong `crates/skynet-ledger/src/compute_escrow_types.rs` và hiện thực `ChunkEscrowSchedule` (theo `TaskId`), `ChunkReleaseReceipt`, `ComputeEscrowVault::release_chunk_milestone` đồng bộ với `SynapticVoucher` lũy kế (hỗ trợ tất toán đơn phương khi khách hàng ngắt kết nối) kèm khối `#[cfg(test)]` tại `crates/skynet-ledger/src/streaming_escrow.rs` (`FR-010`, `SC-005`).
- [x] T037 [P] [US4] Hiện thực `YieldSweepConfig` (`auto_sweep_skn = 50 SKN` mặc định), `build_yield_sweep_tx` chuyển lãi ròng sang `TxOutput::direct(human_owner, swept_yield)` (`0%` hao phí trao đổi chất), giữ nguyên `operating_reserve` trong `CustodyPolicy::AgentBond` và bảo toàn bất biến `HumanPrincipalDiversionBlocked` + `Sovereign Reclaim` kèm khối `#[cfg(test)]` tại `crates/skynet-ledger/src/yield_sweep.rs` (`FR-010`, `SC-005`).
- [x] T038 [P] [US4] Hiện thực bộ đa hợp 3 nấc `WorkloadMultiplexer` (`DeepIdle`: đan xen `MicroTrainingShard` + `FastInferenceSub10ms`; `LightIdle`: chỉ chạy `FastInferenceSub10ms` + `CanaryVerification`; `Paused*`: ngừng 100%) và `PowerProfitabilityGuard` (từ chối thầu dưới giá điện ước tính) kèm khối `#[cfg(test)]` tại `crates/skynet-agent/src/workload_mux.rs` (`FR-009`).
- [x] T039 [US4] Khai báo và re-export `streaming_escrow`, `yield_sweep` tại `crates/skynet-ledger/src/lib.rs`, chạy `cargo test -p skynet-ledger` xác nhận toàn bộ unit tests giải ngân từng lát và quét lãi pass 100% (`FR-010`, `SC-005`).
- [x] T040 [US4] Hiện thực `OpportunisticWorkerEngine` điều phối gom mảnh `FountainDecoder`, chạy `execute_micro_train_slices` / suy luận `< 10ms`, thẩm định Canary, nhận `ChunkVoucherDelivery` và kích hoạt `build_yield_sweep_tx` tại `crates/skynet-agent/src/opportunistic_worker.rs`; xóa bỏ stub `Ok(None)` trong `crates/skynet-agent/src/datagram_router.rs` để nối thật `ImpulseBody::{Compute,Rendezvous,MicroTrain}` và re-export `workload_mux`, `opportunistic_worker` tại `crates/skynet-agent/src/lib.rs` (`FR-006`..`FR-010`).
- [x] T041 [US4] Viết integration test đầu-cuối kiểm chứng luồng 10 lát huấn luyện vi mô + bẫy Canary bắt giữ nút gian lận cắt cọc + tất toán đơn phương `SynapticVoucher` khi Client rớt mạng ở lát thứ 6 + `Yield Sweep` tự động về Ví Lạnh bảo toàn `21T SKN` tại `crates/skynet-agent/tests/micro_train_canary_settlement_test.rs` (`FR-008`, `FR-009`, `FR-010`, `SC-004`, `SC-005`).
- [x] T042 [US4] Hiện thực kịch bản mô phỏng hỗn loạn đa tác tử dưới mức rớt gói UDP `40%`, bật Game ngắt `< 100ms` + nối lại checkpoint, `30%` nút Byzantine và `Yield Sweep` tại `crates/skynet-sim/src/opportunistic_sim.rs` và nối vào `crates/skynet-sim/src/main.rs` (`SC-001`, `SC-003`, `SC-004`, `SC-005`).

**Checkpoint**: User Story 4 hoàn tất — toàn bộ chu trình kinh tế đa tác vụ, giải ngân từng lát và quét lãi tự động vận hành khép kín trên sổ cái.

---

## Phase 7: User Story 5 - CLI Turnkey 1 Lệnh Cho Game Thủ/Khách Hàng & Trạm Giám Sát Thu Nhập 60fps (Priority: P2)

**Goal**: Cung cấp trải nghiệm 1 dòng lệnh thực chiến cho Game thủ (`skynet-agent --opportunistic ...`) và khách hàng (`skynet compute dispatch|train`), xóa sạch mã giả lập CLI cũ, và nâng cấp `skynet-dashboard` lên WebSocket RFC 6455 `60fps` kèm bảng `Worker Earnings & Opportunistic Studio`.  
**Independent Test**: Chạy `cargo test -p skynet-node` và `cargo test -p skynet-dashboard`.

- [x] T043 [P] [US5] Hiện thực `crates/skynet-node/src/cli/compute_cmd.rs` cung cấp `skynet compute presets`, nâng cấp `skynet compute dispatch` bắn gói UDP thực tế và bổ sung `skynet compute train --input <PATH> [--epochs <N>] [--bounty <SKN>] [--target <ADDR>]` phát luồng shard `FountainDroplet`, nối vào `crates/skynet-node/src/cli/mod.rs` và `crates/skynet-node/src/bin/skynet.rs` (`FR-007`, `FR-011`, `SC-006`).
- [x] T044 [P] [US5] Hiện thực bộ phân tích cấu hình Turnkey Worker (`--opportunistic`, `--owner`, `--seed`, `--genesis`, `--schedule`, `--auto-sweep-skn`, `--game-watchlist`, `--max-vram-pct`, `--idle-hold-secs`, bảo toàn 100% cờ cũ `--init-worker`, `--auto-bid`) tại `crates/skynet-node/src/worker_cli.rs` (re-export tại `crates/skynet-node/src/lib.rs`) và nâng cấp `crates/skynet-node/src/bin/skynet_agent.rs` chạy vòng lặp `UdpSocket` + `OsTelemetryGovernor` + `OpportunisticWorkerEngine` thật (xóa bỏ hoàn toàn timer cộng giả `+15 SKN`) (`FR-011`, `SC-006`).
- [x] T045 [P] [US5] Hiện thực bộ bắt tay nâng cấp WebSocket RFC 6455 thuần Rust (SHA-1 + Base64 tiêu chuẩn, `0` crate ngoài) và codec/luồng đẩy khung văn bản độc lập kiểu dữ liệu (nhận chuỗi JSON `&str` ở chu kỳ `16ms`/`60fps`, không phụ thuộc `dto.rs`) kèm khối `#[cfg(test)]` tại `crates/skynet-dashboard/src/ws_rfc6455.rs` (`FR-012`, `SC-007`).
- [x] T046 [P] [US5] Hiện thực `WorkerStudioTelemetryDto`, endpoint `GET /api/worker-studio` và các hành động kịch bản tương tác (`sim_game_launch`, `sim_thermal_spike`, `sim_canary_fraud`, `sim_yield_sweep`) tại `crates/skynet-dashboard/src/worker_studio_api.rs` + mở rộng DTO tại `crates/skynet-dashboard/src/dto.rs` (`FR-012`, `SC-007`).
- [x] T047 [US5] Hiện thực script client mô-đun `crates/skynet-dashboard/assets/worker_studio.js` (`<= 220` dòng), gắn khung hiển thị **Worker Earnings & Opportunistic Studio** vào `crates/skynet-dashboard/assets/dashboard.html`, và nối `ws_rfc6455`, `worker_studio_api`, `/ws/telemetry`, `/api/worker-studio`, `/assets/worker_studio.js` vào `crates/skynet-dashboard/src/{lib.rs,http_router.rs,main.rs}` (`FR-012`, `SC-007`).
- [x] T048 [US5] Viết integration test in-process (khởi chạy máy chủ từ `crates/skynet-dashboard/src/lib.rs` trên cổng động `127.0.0.1:0`) kiểm chứng bắt tay WebSocket RFC 6455, nhận luồng khung hình `60fps` và thực thi các nút mô phỏng Studio (`< 100ms` phản hồi trạng thái) tại `crates/skynet-dashboard/tests/ws_worker_studio_test.rs` (`FR-012`, `SC-007`).

**Checkpoint**: Toàn bộ 5 User Stories (`US1`–`US5`) hoàn tất đầu-cuối từ tầng mật mã, tế bào nơ-ron, mạng P2P đến CLI và Dashboard 60fps.

---

## Phase 8: Polish & Cross-Cutting Verification

**Goal**: Kiểm định hội tụ toàn diện trên toàn bộ Cargo Workspace theo đúng quy chuẩn SkyNet Constitution.

- [x] T049 Chạy kiểm tra kiểu tĩnh toàn workspace `cargo check --workspace` đối với `Cargo.toml` gốc đạt `0` lỗi (`SC-008`).
- [x] T050 Chạy linter nghiêm ngặt `cargo clippy --workspace --all-targets -- -D warnings` trên toàn bộ `crates/*` đạt `0` cảnh báo (`SC-008`).
- [x] T051 Chạy toàn bộ bộ kiểm thử tự động `cargo test --workspace` và chạy xác thực mô phỏng `cargo run -p skynet-sim` trên `crates/skynet-sim/src/main.rs` xác nhận `100%` tests pass (`SC-001`..`SC-008`).
- [x] T052 Kiểm toán vật lý toàn bộ tệp nguồn trong `Cargo.toml` và `crates/{skynet-dna,skynet-cell,skynet-ledger,skynet-wire,skynet-agent,skynet-node,skynet-sim,skynet-dashboard}/src/*`: xác nhận `0` crate mới, `0` tệp `utils.rs`/`helpers.rs`, `100%` tệp chạm tới `<= 250` dòng mục tiêu (trần cứng `350`), hàm `<= 40` dòng, tham số `<= 4`, độ lồng `<= 3` và không dùng `.unwrap()` trên dữ liệu runtime (`SC-008`).

---

## Ma Trận Truy Vết Đầy Đủ (100% Cross-Artifact Traceability)

| Hạng Mục Đặc Tả (`spec.md`) | Ánh Xạ Tác Vụ (`tasks.md`) |
| :--- | :--- |
| **`US1`** (Genesis & P2P NAT Rendezvous) | `T014`–`T022` |
| **`US2`** (Zero-Impact Opportunistic Governor) | `T010`, `T013`, `T023`–`T029` |
| **`US3`** (Micro-Training, Plasmid & PoUW Canary Slashing) | `T011`, `T012`, `T013`, `T030`–`T035` |
| **`US4`** (Multi-Workload Mux, Streaming Escrow & Yield Sweep) | `T036`–`T042` |
| **`US5`** (Turnkey CLI & 60fps Worker Earnings Studio) | `T043`–`T048` |
| **`FR-001`** (Genesis `21T SKN` 85%/15% & `GENESIS_HASH`) | `T014`, `T019`, `T021`, `T022` |
| **`FR-002`** (`EpidermalFilter` `< 5µs` zero-alloc & `±30s`) | `T015`, `T017`, `T020`, `T022` |
| **`FR-003`** (LAN `< 1s`, `PheromoneHello`/`PunchRendezvous` & `GangliaRelay`) | `T016`, `T017`, `T018`, `T020`, `T022` |
| **`FR-004`** (Rootless OS Probes macOS/Linux/Windows `250ms`/`1500ms`) | `T024`, `T025`, `T026`, `T028`, `T029` |
| **`FR-005`** (`< 100ms` Preemption, `< 5ms` Checkpoint, `8°C` & `30s` Hysteresis) | `T010`, `T013`, `T023`, `T027`, `T028`, `T029`, `T031` |
| **`FR-006`** (`Candle` Multi-Step Micro-Training, `Plasmid` & `f16` Codec) | `T011`, `T013`, `T030`, `T031`, `T035`, `T040` |
| **`FR-007`** (`<= 128 KB` Fountain Shards `<= 1200B` MTU & Opcode Domain `0x13`) | `T012`, `T033`, `T043` |
| **`FR-008`** (`Canary Spot-Check` `10%`, Fraud Slashing & `TrimmedMean`) | `T032`, `T034`, `T035`, `T040`, `T041` |
| **`FR-009`** (3-Tier `WorkloadMultiplexer` & `PowerProfitabilityGuard`) | `T038`, `T040`, `T041` |
| **`FR-010`** (`Chunk-Streaming Settlement` & `Soul-Body Yield Sweep`) | `T036`, `T037`, `T039`, `T040`, `T041`, `T042` |
| **`FR-011`** (Turnkey CLI `skynet-agent --opportunistic`, `skynet` real UDP) | `T006`, `T019`, `T021`, `T043`, `T044` |
| **`FR-012`** (WebSocket RFC 6455 `60fps` & `Worker Earnings Studio`) | `T008`, `T045`, `T046`, `T047`, `T048` |
| **`SC-001`** (`< 100ms` Preemption SLA & `< 5ms` Atomic Checkpoint) | `T010`, `T023`, `T027`, `T028`, `T029`, `T031`, `T042` |
| **`SC-002`** (`<= 1200B` UDP MTU, `<= 300B` Rendezvous & `< 5µs` Zero-Alloc Filter) | `T015`, `T016`, `T022`, `T033` |
| **`SC-003`** (`100%` Recovery at `40%` UDP Loss & `30%` Byzantine Convergence) | `T031`, `T033`, `T035`, `T042` |
| **`SC-004`** (`> 85%` Compute Savings via `10%` Canary & `100%` Fraud Slashing) | `T032`, `T034`, `T041`, `T042` |
| **`SC-005`** (`21T SKN` Exact Supply & `Soul-Body Custody` Invariants) | `T014`, `T022`, `T032`, `T036`, `T037`, `T039`, `T041`, `T042` |
| **`SC-006`** (`< 1s` LAN, `< 3s` NAT Hole Punch & Real Multi-Node Execution) | `T018`, `T020`, `T022`, `T043`, `T044` |
| **`SC-007`** (`60fps` WebSocket Stream & `< 100ms` Studio UI Reflection) | `T045`, `T046`, `T047`, `T048` |
| **`SC-008`** (`<= 250` Line Target, `0` New Crates & `100%` Check/Clippy/Test) | `T001`–`T009`, `T049`–`T052` |

## Dependencies & Execution Strategy

- **Execution Order**: `Phase 1 (Splits)` → `Phase 2 (Foundational)` → `Phases 3, 4, 5 (P1 Stories)` → `Phases 6, 7 (P2 Stories)` → `Phase 8 (Polish & Verification)`.
- **Wave-2 Parallel Swarm Slices (100% Khác Crate/Module, Không Xung Đột Tệp — khớp 1-1 với `plan.md` §7)**:
  1. **Author `dna-cell` (`skynet-dna` + `skynet-cell`)**: `T001`, `T002`, `T010`, `T011`, `T013`, `T030`, `T031`, `T034`, `T035`
  2. **Author `ledger` (`skynet-ledger`)**: `T003`, `T014`, `T032`, `T036`, `T037`, `T039`
  3. **Author `wire` (`skynet-wire`)**: `T004`, `T012`, `T015`, `T016`, `T017`, `T033`
  4. **Author `agent` (`skynet-agent`)**: `T005`, `T018`, `T023`, `T038`, `T040`, `T041`
  5. **Author `node-cli` (`skynet-node`)**: `T006`, `T019`–`T022`, `T024`–`T029`, `T043`, `T044`
  6. **Author `sim-dashboard` (`skynet-sim` + `skynet-dashboard`)**: `T007`, `T008`, `T042`, `T045`–`T048`

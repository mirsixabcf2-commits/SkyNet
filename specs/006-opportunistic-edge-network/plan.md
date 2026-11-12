# Implementation Plan: Mạng Lưới AI Cận Biên Tận Dụng GPU/CPU Nhàn Rỗi, Huấn Luyện Vi Mô Phân Tán & Kinh Tế P2P Thời Gian Thực

**Feature Directory**: `specs/006-opportunistic-edge-network` | **Date**: 2026-11-12 | **Spec**: [spec.md](./spec.md)

---

## 1. Summary

Bản kế hoạch này chuyển hóa đặc tả [spec.md](./spec.md) (`US1`–`US5`, `FR-001`–`FR-012`, `SC-001`–`SC-008`) thành kiến trúc kỹ thuật Production trên Cargo Workspace 8 crates của SkyNet, giải quyết đồng thời hai mục tiêu:
1. **Khép kín toàn bộ khoảng trống kết nối P2P đa thiết bị thực tế (`005` + Runtime Wiring):** Hiện thực hóa `GenesisConfig` / `GENESIS_HASH` (`21T SKN`: `85%` Vault + `15%` Circulating), màng lọc tế bào Da không cấp phát bộ nhớ động (`EpidermalFilter` `< 5µs`, chống phát lại `±30s`), tự động khám phá LAN (`< 1s`), đục lỗ NAT UDP đồng thời qua Seed (`PheromoneHello` / `PunchRendezvous`) kèm chuyển tiếp dự phòng `GangliaRelay`, và thay thế toàn bộ các đoạn mã giả lập ở CLI/Daemon bằng luồng xử lý UDP & sổ cái thật.
2. **Triển khai Mạng AI Cơ hội "Nhanh hơn & Rộng hơn":** Bộ điều tốc phần cứng đa nền tảng không cần quyền `root` kết hợp ngắt lát cắt nội tại `< 100ms` SLA (`< 25ms` trung vị) + lưu nguyên tử `MicroTrainCheckpoint` (`< 5ms`), động cơ huấn luyện vi mô `candle-core` (`Var`, `Tensor`, `.backward()`) nén chênh lệch trọng số `half::f16` truyền qua `FountainDroplet` (`<= 1200B` MTU, chịu mất gói `40%`), kiểm chứng bẫy `Canary Spot-Check` `O(1)` kèm cắt cọc bảo lãnh (`slash_on_fraud_proof`) & tổng hợp `TrimmedMeanAggregator`, giải ngân từng lát `ChunkSettlement` + tự động `Yield Sweep` về Ví Lạnh theo chuẩn **Soul–Body Custody**, và nâng cấp `skynet-dashboard` lên WebSocket RFC 6455 thuần Rust `60fps` kèm bảng `Worker Earnings & Opportunistic Studio`.

---

## 2. Technical Context

- **Language / Runtime**: Rust Edition `2021` (`rust-version = "1.83.0"`), Async Runtime `tokio = "=1.41.1"` (`rt-multi-thread, net, time, sync, macros, io-util, signal`).
- **Primary Dependencies (ZERO New External Crates)**: Sử dụng **duy nhất** các crates đã khai báo trong `[workspace.dependencies]` gốc (`blake3 = "=1.5.5"`, `bincode = "1.3.3"`, `candle-core = "=0.8.0"`, `ed25519-dalek = "=2.1.1"`, `half = "=2.4.1"`, `rand = "0.8.5"`, `rand_chacha = "0.3.1"`, `serde = "1.0"`, `serde_json = "1.0"`, `thiserror = "1.0"`, `tokio = "=1.41.1"`). Không thêm `sysinfo`, `nvml`, `libc`, `tokio-tungstenite`, `sha1`, hay `base64`:
  - **OS Telemetry:** Đọc trực tiếp `std::fs` (`/proc`, `/sys` trên Linux) và gọi `std::process::Command` (`ioreg`, `pmset`, `sysctl`, `lsappinfo`, `ps` trên macOS; `nvidia-smi`, `tasklist` trên Windows/Linux) bên trong `tokio::task::spawn_blocking`, cập nhật nguyên tử qua `AtomicU8` / `RwLock`.
  - **Autograd Micro-Training:** Dùng trực tiếp `candle_core::{Var, Tensor, Device, backprop::GradStore}` và nén `half::f16`.
  - **WebSocket 60fps:** Tự hiện thực bộ bắt tay nâng cấp RFC 6455 (SHA-1 + Base64 tiêu chuẩn ~120 dòng thuần Rust) và đóng khung văn bản trên `tokio::net::TcpStream`.
- **Crate Dependency DAG (1 Chiều Tuyệt Đối)**:
  ```text
  skynet-dna
      ├──► skynet-cell ──┐
      └──► skynet-ledger ┼──► skynet-wire ──► skynet-agent ──► skynet-node ──┬──► skynet-sim
                         │                                                   └──► skynet-dashboard
                         └──────────────────────────────────────────────────────► (re-used upward)
  ```
- **Storage**: Lưu trữ tệp nhị phân/JSON nguyên tử qua quy trình ghi tệp tạm `.tmp` rồi `std::fs::rename` (`genesis.json`, `MicroTrainCheckpoint`, `FullNodeSnapshot`, `worker_studio_state.json`).
- **Testing**: `cargo check --workspace`, `cargo clippy --workspace --all-targets`, `cargo test --workspace` (kế thừa baseline `70 passed, 0 failed` và mở rộng unit + integration tests trên toàn bộ 8 crates).
- **Constraints & Performance Targets**:
  - UDP MTU cứng `<= 1200 Bytes` (`MAX_UDP_DATAGRAM_BYTES`); bản tin bắt tay Rendezvous `<= 300 Bytes`; màng lọc Da `< 5µs` với `0` cấp phát heap khi gói lỗi.
  - Nhường máy `< 100ms` SLA (`< 25ms` qua `SliceLatencyGuard`); ghi checkpoint nguyên tử `< 5ms`.
  - Bảo toàn tuyệt đối `GENESIS_TOTAL_NANO_SKN = 21,000,000,000,000 * 10^9 nanoSKN` và bất biến lưu ký `HumanPrincipalDiversionBlocked` + `Sovereign Reclaim`.
  - Giới hạn vật lý mã nguồn: Mục tiêu `<= 250 dòng/file` (trần cứng `350 dòng`), hàm `<= 40 dòng`, tối đa `4 tham số/hàm` (gom vào Struct Config/Options), độ sâu lồng nhau `<= 3`, cấm `utils.rs`/`helpers.rs`.

---

## 3. Constitution Check

*GATE: Phải vượt qua 100% trước khi triển khai mã nguồn.*

| Nguyên Tắc Constitution | Giải Pháp Thiết Kế Cụ Thể | Trạng Thái |
| :--- | :--- | :---: |
| **I.1 Giới hạn File `<= 250` dòng (Trần `350`)** | Lập **Phase 1: Prerequisite Modular Splits** tách toàn bộ 11 tệp hiện hữu `>= 200` dòng cần chạm tới thành các submodule chuyên biệt `<= 230` dòng trước khi thêm tính năng; mọi tệp mới đều thiết kế trong ngân sách `120–220` dòng. | **PASS** |
| **I.2 Hàm `<= 40` dòng, Tham số `<= 4`, Nesting `<= 3`** | Dùng Guard Clauses (`let ... else { return ... }`, `?`), cấm `else` dư sau `return`; gom tham số vào các DTO định kiểu (`GovernorConfig`, `MicroTrainConfig`, `CanaryConfig`, `ChunkSettlementReq`, `YieldSweepConfig`). | **PASS** |
| **I.3 Phân tầng Kiến trúc & Cấm `utils.rs`/`helpers.rs`** | Cô lập rõ tầng Wire (`skynet-wire`), tầng Domain (`skynet-dna`, `skynet-cell`, `skynet-ledger`, `skynet-agent`), tầng OS Adapter/CLI (`skynet-node`) và tầng UI (`skynet-dashboard`). `0` tệp mang tên `utils` hoặc `helpers`. | **PASS** |
| **II.1 Xử lý lỗi `Result<T, E>` (`thiserror`), Cấm `.unwrap()`** | Định nghĩa kiểu lỗi miền rõ ràng (`GenesisError`, `GovernorError`, `MicroTrainError`, `LedgerError`, `WireError`, `AgentError`, `RuntimeError`); cấm `.unwrap()`/`.expect()` trên dữ liệu mạng/OS/đĩa runtime. | **PASS** |
| **II.2 Kiểm chứng Biên & Không Hardcode Magic Numbers** | Kiểm tra `EpidermalFilter` ngay tại biên socket trước khi giải mã sâu; tập trung toàn bộ ngưỡng (`78°C`, `30s`, `1200B`, `±30s`, `1000 bps`) vào hằng số/cấu hình định danh. | **PASS** |
| **IV. Blast Radius & Zero External Crates** | Giữ nguyên `Cargo.toml` không thêm bất kỳ crate ngoài nào; giữ nguyên vẹn các tệp ổn định không cần sửa (`hwid_airdrop.rs`, `pope_benchmark.rs`, `compute_candle.rs`, `storage.rs`, `fountain.rs`, `tabular_csv.rs`); bảo toàn 100% giao diện CLI/API cũ. | **PASS** |

---

## 4. Ma Trận Tách Submodule Tiền Điều Kiện (Prerequisite Splits cho Tệp `>= 200` Dòng)

Dựa trên kiểm toán toàn bộ 41 tệp `.rs` hiện có, bảng dưới đây quy định xử lý bắt buộc đối với mọi tệp `>= 200` dòng:

| Tệp Hiện Hữu | Số Dòng Hiện Tại | Tác Động | Kế Hoạch Xử Lý Bắt Buộc Trong Phase 1 (Trước Khi Thêm Tính Năng) |
| :--- | :---: | :---: | :--- |
| `crates/skynet-ledger/src/lib.rs` | `1639` | Chạm tới | **Tách thành 8 tệp `<= 230` dòng:** `nano_skn.rs`, `halving_vault.rs`, `custody.rs`, `voucher.rs`, `utreexo.rs`, `dag_ledger.rs`, `ledger_core_tests.rs`, `ledger_custody_tests.rs` + `lib.rs` (`~110` dòng re-export). |
| `crates/skynet-agent/src/lib.rs` | `1105` | Chạm tới | **Tách thành 6 tệp `<= 225` dòng:** `snowball.rs`, `organism_state.rs`, `datagram_router.rs`, `agent_core_tests.rs`, `agent_network_tests.rs` + `lib.rs` (`~95` dòng re-export). |
| `crates/skynet-dashboard/src/main.rs` | `941` | Chạm tới | **Tách thành 6 tệp `<= 220` dòng:** `lib.rs` (`~55` dòng re-export cho cả binary và integration test), `dto.rs`, `live_world.rs`, `world_actions.rs`, `http_router.rs` + `main.rs` (`~65` dòng). |
| `crates/skynet-dna/src/lib.rs` | `481` | Chạm tới | **Tách thành 4 tệp `<= 210` dòng:** `genes.rs`, `cell_dna.rs`, `dna_tests.rs` + `lib.rs` (`~55` dòng re-export). |
| `crates/skynet-ledger/src/hwid_airdrop.rs` | `446` | Không sửa | **Giữ nguyên (Blast Radius):** API `MarketingBootstrapVault` đã công khai đầy đủ để CLI gọi trực tiếp. |
| `crates/skynet-cell/src/lib.rs` | `414` | Chạm tới | **Tách thành 4 tệp `<= 190` dòng:** `hardware.rs`, `micro_cell.rs`, `cell_tests.rs` + `lib.rs` (`~70` dòng re-export). |
| `crates/skynet-sim/src/main.rs` | `407` | Chạm tới | **Tách thành 3 tệp `<= 220` dòng:** `classic_sim.rs`, `sim_metrics.rs` + `main.rs` (`~90` dòng) trước khi thêm `opportunistic_sim.rs`. |
| `crates/skynet-wire/src/lib.rs` | `381` | Chạm tới | **Tách thành 3 tệp `<= 195` dòng:** `packet.rs`, `wire_tests.rs` + `lib.rs` (`~80` dòng re-export). |
| `crates/skynet-ledger/src/compute_escrow.rs` | `324` | Chạm tới | **Tách thành 2 tệp `<= 200` dòng:** `compute_escrow.rs` (mở `pub(crate) records`, giữ lock/complete/auditor) + `compute_escrow_refund.rs` (timeout refund). |
| `crates/skynet-cell/src/pope_benchmark.rs` | `274` | Không sửa | **Giữ nguyên (Blast Radius):** API PoPE đã hoàn chỉnh cho `skynet airdrop claim`. |
| `crates/skynet-agent/src/compute_flow.rs` | `238` | Không sửa | **Giữ nguyên:** Logic điều phối cơ hội mới đặt độc lập tại `opportunistic_worker.rs`. |
| `crates/skynet-agent/src/auditor_flow.rs` | `235` | Không sửa | **Giữ nguyên:** Logic kiểm toán Canary đặt tại `skynet-cell/src/canary_pouw.rs` & `opportunistic_worker.rs`. |
| `crates/skynet-wire/src/opcode_registry.rs` | `233` | Chạm tới | **Tách thành 2 tệp `<= 160` dòng:** `opcode_domains.rs` (danh mục miền) + `opcode_registry.rs` (tra cứu descriptor) trước khi thêm miền `0x13`. |
| `crates/skynet-node/src/storage.rs` | `231` | Không sửa | **Giữ nguyên:** Lưu trữ checkpoint tách độc lập tại `governor/checkpoint_store.rs`. |
| `crates/skynet-cell/src/compute_candle.rs` | `225` | Không sửa | **Giữ nguyên:** Tái sử dụng `pub fn compute_canonical_digest`; huấn luyện vi mô đặt tại `micro_train.rs`. |
| `crates/skynet-node/src/bin/skynet.rs` | `224` | Chạm tới | **Tách thành `src/cli/{mod.rs, wallet_opcode_cmd.rs}` + `bin/skynet.rs` (`~65` dòng)** trước khi thêm `genesis_airdrop_cmd.rs` & `compute_cmd.rs`. |
| `crates/skynet-wire/src/mcp_capsule.rs` | `209` | Chạm tới | **Tách unit tests sang `mcp_capsule_tests.rs` (`~70` dòng)** để `mcp_capsule.rs` còn `~150` dòng trước khi thêm `OpcodeDomain::MicroTrainEvolution = 0x13`. |
| `crates/skynet-node/src/preprocessor/tabular_csv.rs` | `207` | Không sửa | **Giữ nguyên.** |
| `crates/skynet-wire/src/fountain.rs` | `206` | Không sửa | **Giữ nguyên:** Tái sử dụng trực tiếp `FountainEncoder` / `FountainDecoder` cho shard `<= 128 KB`. |

---

## 5. Thiết Kế Kiến Trúc & Hợp Đồng Dữ Liệu Theo Tầng Crate

### 5.1. Tầng Di Truyền (`crates/skynet-dna`)
- **`plasmid_patch.rs`** (`FR-006`, `US3`):
  - `PlasmidPatch`: `{ pub patch_id: [u8; 32], pub parent_dna_hash: [u8; 32], pub fitness_gain_bps: u16, pub quantized_deltas_i16: Vec<i16>, pub scale_milli: u32, pub canonical_digest: [u8; 32] }`.
  - `apply_plasmid_patch(dna: &CellDna, patch: &PlasmidPatch) -> Result<CellDna, DnaError>`: Kiểm chứng `canonical_digest` (`BLAKE3` domain `"SKYNET_PLASMID_PATCH_V1"`), giải lượng tử hóa và trộn thích nghi có kiểm soát biên độ vào `AdaptiveGenes`, trả về `CellDna` con đã cập nhật mã băm toàn vẹn.

### 5.2. Tầng Tế Bào Nơ-ron & PoUW (`crates/skynet-cell`)
- **`preempt_guard.rs`** (`FR-005`, `SC-001`, `US2`):
  - `PreemptSignal` (`#[repr(u8)]`: `Run = 0`, `Throttle50 = 1`, `PreemptImmediate = 2`) bọc trong `PreemptGuard` (`Arc<AtomicU8>`).
  - `SliceLatencyGuard`: Theo dõi trung bình động hàm mũ `ema_slice_micros: u64`; nếu một vi lát cắt chạy lâu hơn `ema_slice_micros * 18 / 10` (sau giai đoạn làm nóng 3 lát đầu), tự động kích hoạt `PreemptImmediate` trong `< 20ms` mà không tốn lời gọi hệ thống.
  - `MicroTrainCheckpoint`: `{ pub task_id: [u8; 32], pub shard_id: u16, pub completed_steps: u16, pub total_steps: u16, pub weights_f16_bits: Vec<u16>, pub last_loss_micro: u64, pub checkpoint_digest: [u8; 32] }`.
- **`gradient_codec.rs`** (`FR-006`, `US3`):
  - Nén vector `f32` sang `half::f16` (`encode_dense_f16(deltas: &[f32]) -> Vec<u16>`, `decode_dense_f16(bits: &[u16]) -> Vec<f32>`), nén thưa `SparseTopKEntry { index: u16, f16_bits: u16 }` cho tinh chỉnh nhẹ, tính chuẩn L2 và cắt chuẩn `clip_l2_norm`.
- **`micro_train.rs`** (`FR-005`, `FR-006`, `US2`, `US3`):
  - `MicroTrainConfig`: `{ pub dim_in: usize, pub dim_hidden: usize, pub dim_out: usize, pub inner_steps: u16, pub learning_rate_micro: u32, pub max_grad_norm_milli: u32 }`.
  - `MicroTrainBatch`: `{ pub task_id: [u8; 32], pub shard_id: u16, pub base_weights_f16: Vec<u16>, pub samples_x: Vec<f32>, pub targets_y: Vec<f32> }`.
  - `MicroTrainOutcome`: `Completed(MicroTrainResult)` hoặc `Preempted(MicroTrainCheckpoint)`.
  - `execute_micro_train_slices(cfg: &MicroTrainConfig, batch: &MicroTrainBatch, resume: Option<&MicroTrainCheckpoint>, guard: &PreemptGuard) -> Result<MicroTrainOutcome, MicroTrainError>`: Khởi tạo `candle_core::Var` trên thiết bị phần cứng (`HardwareProfile::select_compute_device()`), thực thi từng bước MSE forward + `.backward()` + cập nhật SGD có kiểm tra `guard` sau mỗi bước (`<= 15ms`/slice). Khi hoàn tất, xuất `delta_f16`, `initial_loss`, `final_loss` và `canonical_digest = compute_canonical_digest(&delta_f32)`.
- **`canary_pouw.rs`** (`FR-008`, `SC-004`, `US3`):
  - `CanaryConfig`: `{ pub canary_rate_bps: u16 /* default 1000 */, pub min_cosine_milli: u16 /* default 985 */, pub max_grad_norm_milli: u32 }`.
  - `generate_canary_batch(seed: &[u8; 32], epoch: u64, shard_id: u16, cfg: &MicroTrainConfig) -> MicroTrainBatch`: Sinh dữ liệu bẫy tất định qua `BLAKE3` XOF (`"SKYNET_POUW_CANARY_V1"`).
  - `verify_canary_spot_check(worker_res: &MicroTrainResult, reference_res: &MicroTrainResult, canary_cfg: &CanaryConfig) -> CanaryVerdict`: Kiểm tra không chứa `NaN`/`Inf`, `final_loss <= initial_loss`, chuẩn L2 hợp lệ và `cosine_similarity >= 0.985` (trả về `Valid` hoặc `FraudDetected { reason, measured_cosine_milli }`).
- **`federated_agg.rs`** (`FR-008`, `SC-003`, `US3`):
  - `TrimmedMeanAggregator`: Hợp nhất danh sách `(WeightDelta, pouw_score_bps)` bằng cách loại bỏ `trim_ratio_bps` (`10%` cao nhất và `10%` thấp nhất trên từng tọa độ khi có `>= 4` mẫu), tính trung bình có trọng số uy tín và tùy chọn đóng gói thành `PlasmidPatch`.

### 5.3. Tầng Sổ Cái, Genesis, Cắt Cọc & Kinh Tế Streaming (`crates/skynet-ledger`)
- **`genesis.rs`** (`FR-001`, `SC-005`, `US1`):
  - `GenesisConfig`: `{ pub network_name: String, pub total_supply: NanoSkn, pub vault_allocation: NanoSkn, pub circulating_allocation: NanoSkn, pub initial_utxos: Vec<GenesisUtxoEntry>, pub annual_halving_epochs: u64 }`.
  - `validate_genesis_invariants(&GenesisConfig) -> Result<(), LedgerError>`: Cưỡng chế `total_supply == GENESIS_TOTAL_NANO_SKN` (`21T SKN`), `vault_allocation == 85%` (`17.85T SKN`), `circulating_allocation == 15%` (`3.15T SKN`) và `vault_allocation + sum(initial_utxos) == total_supply` chính xác tới `1 nanoSKN`.
  - `compute_genesis_hash(&GenesisConfig) -> [u8; 32]`: Băm tất định theo thứ tự trường chuẩn hóa qua `BLAKE3` (`"SKYNET_GENESIS_CONFIG_V1"`), hoàn toàn độc lập với định dạng khoảng trắng của tệp JSON.
- **`compute_escrow_types.rs` & `escrow_slash.rs`** (`FR-008`, `SC-004`, `US3`):
  - Mở rộng `compute_escrow_types.rs` (`<= 155` dòng): bổ sung biến thể `EscrowStatus::Slashed` cùng các mã lỗi `EscrowError` cho bằng chứng gian lận và giải ngân theo lát; dùng nhất quán kiểu `TaskId` (`compute_escrow_types::TaskId`, re-export thành `ComputeTaskId` tại `skynet-ledger/src/lib.rs`).
  - `FraudProofClaim`: `{ pub task_id: TaskId, pub faulty_worker: [u8; 32], pub challenger_auditor: [u8; 32], pub measured_cosine_milli: i16, pub evidence_digest: [u8; 32] }`.
  - `WorkerReputation`: `{ pub pouw_score_bps: u16 /* 0..=10_000, default 5_000 */, pub verified_chunks: u64, pub slashed_count: u32 }`.
  - `ComputeEscrowVault::slash_on_fraud_proof(...) -> Result<SlashOutcome, EscrowError>`: Thu giữ 100% `bond` của Worker gian lận (chia `50%` thưởng cho `challenger_auditor`, hoàn trả `100%` `bounty + 50% bond` cho Client), đánh dấu `EscrowStatus::Slashed`, và đặt `pouw_score_bps = 0`.
- **`streaming_escrow.rs`** (`FR-010`, `SC-005`, `US4`):
  - `ChunkEscrowSchedule`: Đăng ký tổng số lát `total_chunks: u16` cho một `TaskId`.
  - `ComputeEscrowVault::release_chunk_milestone(task_id, chunk_idx, worker_pubkey) -> Result<ChunkReleaseReceipt, EscrowError>`: Giải ngân tuyến tính phần thưởng lát `chunk_idx` và đối chiếu đồng bộ với `SynapticVoucher` lũy kế (`sequence = chunk_idx as u64`), bảo đảm tổng giải ngân các lát cộng phần dư cuối cùng khớp chính xác `100%` số `NanoSkn` đã khóa.
- **`yield_sweep.rs`** (`FR-010`, `SC-005`, `US4`):
  - `YieldSweepConfig`: `{ pub human_owner: [u8; 32], pub operating_reserve: NanoSkn, pub auto_sweep_threshold: NanoSkn }`.
  - `build_yield_sweep_tx(agent_signing_key: &SigningKey, bond_utxos: &[(UtxoId, TxOutput)], cfg: &YieldSweepConfig, tx_nonce: u64) -> Result<Option<SknTransaction>, LedgerError>`: Nếu tổng số dư `AgentBond` vượt `operating_reserve + auto_sweep_threshold`, tạo giao dịch hợp lệ chuyển phần lãi thặng dư sang `TxOutput::direct(cfg.human_owner, swept_yield)` (`0%` phí đốt trao đổi chất) và giữ nguyên `operating_reserve` trong đầu ra đổi tiền thừa `CustodyPolicy::AgentBond`, tuân thủ tuyệt đối kiểm chứng `HumanPrincipalDiversionBlocked` của `UtxoDagLedger`.

### 5.4. Tầng Khung Tin UDP, Màng Lọc Da & Mã Lệnh (`crates/skynet-wire`)
- **`epidermal_filter.rs`** (`FR-002`, `SC-002`, `US1`):
  - `MAX_TIMESTAMP_DRIFT_SEC: u64 = 30`.
  - `EpidermalHeader`: Header cố định `44 Bytes` đặt ở đầu khung tin có gắn mạng (`magic: [u8; 2]`, `version: u8`, `hop_ttl: u8`, `network_id: [u8; 32]`, `timestamp_sec: u64`) kèm phương thức `EpidermalFilter::verify_stack_header(raw: &[u8], expected_genesis_hash: &[u8; 32], local_now_sec: u64) -> Result<&[u8], WireError>` chạy `100%` trên lát bộ nhớ ngăn xếp (`0` cấp phát heap, `< 5µs`), tương thích ngược với `ImpulsePacket` chuẩn.
- **`rendezvous_wire.rs`** (`FR-003`, `SC-002`, `US1`):
  - `RendezvousMessage` (`<= 300B` cho các gói điều khiển, `<= 1200B` cho gói relay):
    - `PheromoneHello { sender_pubkey: [u8; 32], internal_addr: SocketAddr, node_tier: u8, timestamp_sec: u64, nonce: u64, signature: Vec<u8> }`
    - `PunchRendezvous { target_peer_pubkey: [u8; 32], peer_public_addr: SocketAddr, rendezvous_token: u64, expiry_timestamp: u64, signature: Vec<u8> }`
    - `PunchProbe { sender_pubkey: [u8; 32], rendezvous_token: u64 }`
    - `LanBeacon { sender_pubkey: [u8; 32], listen_port: u16, genesis_hash: [u8; 32] }`
    - `GangliaRelayFrame { target_pubkey: [u8; 32], sender_pubkey: [u8; 32], inner_datagram: Vec<u8> }`
  - Gắn vào `ImpulseBody::Rendezvous(RendezvousMessage)`.
- **`micro_train_wire.rs` & Miền Opcode `0x13`** (`FR-007`, `SC-002`, `US3`):
  - Bổ sung `OpcodeDomain::MicroTrainEvolution = 0x13` trong `mcp_capsule.rs` và đăng ký miền `0x1300` (`0x1301`: `micro_train.mlp_step`, `0x1302`: `micro_train.lora_adapter`, `0x1303`: `evolution.plasmid_guided`) trong `opcode_domains.rs` / `opcode_registry.rs`.
  - `MicroTrainWireMessage`: `MicroBatchManifest`, `WeightDeltaClaim`, `CanaryFraudReport`, `ChunkVoucherDelivery` (`<= 1200B` trên đường truyền; payload shard `<= 128 KB` truyền qua `ImpulseBody::FountainDroplet`), gắn vào `ImpulseBody::MicroTrain(MicroTrainWireMessage)`.

### 5.5. Tầng Tác Tử P2P, Điều Phối Cơ Hội & Đa Hợp Tác Vụ (`crates/skynet-agent`)
- **`rendezvous_broker.rs`** (`FR-003`, `SC-006`, `US1`):
  - `RENDEZVOUS_TTL_SEC: u64 = 120`, `HOLE_PUNCH_BURST_COUNT: usize = 4`.
  - `RendezvousRegistry`: Lưu ánh xạ `pubkey -> PeerRendezvousEntry`, tự động dọn mục quá hạn `> 120s`, ghép cặp hai nút sau NAT sinh cặp gói `PunchRendezvous`, sinh loạt xung `PunchProbe`, và định tuyến chuyển tiếp `GangliaRelayFrame` khi đối phương chưa mở thông kết nối trực tiếp.
- **`governor_fsm.rs`** (`FR-005`, `SC-001`, `US2`):
  - `GovernorConfig`: `{ pub temp_limit_c: u8 /* 78 */, pub hysteresis_cool_c: u8 /* 8 */, pub idle_hold_secs: u64 /* 30 */, pub max_vram_pct: u8 /* 50 */, pub battery_guard: bool /* true */, pub schedule_window: Option<(u16, u16)> }`.
  - `SystemTelemetrySnapshot`: `{ pub gpu_util_pct: u8, pub cpu_util_pct: u8, pub vram_used_pct: u8, pub temp_c: u8, pub on_battery: bool, pub active_game_detected: bool, pub observed_at_sec: u64 }`.
  - `GovernorState`: `DeepIdle`, `LightIdle`, `Throttled`, `PausedGaming`, `PausedThermal`, `PausedBattery`, `PausedVram`, `PausedSchedule`.
  - Máy trạng thái thuần hàm cập nhật tín hiệu `PreemptGuard` và kiểm chứng cửa sổ giữ `30s` + dải trễ nhiệt `8°C`.
- **`workload_mux.rs`** (`FR-009`, `US4`):
  - `WorkloadClass`: `MicroTrainingShard`, `PlasmidEvolution`, `FastInferenceSub10ms`, `CanaryVerification`.
  - `WorkloadMultiplexer` & `PowerProfitabilityGuard`: Quyết định nhận/tạm hoãn tác vụ theo `GovernorState` (`DeepIdle` cho phép mọi lớp và chèn `FastInferenceSub10ms` giữa các lát; `LightIdle` chỉ cho phép `FastInferenceSub10ms` + `CanaryVerification`; `Paused*` từ chối toàn bộ) và kiểm tra `offered_nano_per_sec >= estimated_power_cost_nano_per_sec`.
- **`opportunistic_worker.rs` & Nối Thật `datagram_router.rs`** (`FR-006`..`FR-010`, `US3`, `US4`):
  - Xóa bỏ stub `ImpulseBody::Compute(_) => Ok(None)` trong `AgentNode`; nối trực tiếp `ImpulseBody::Compute`, `ImpulseBody::Rendezvous` và `ImpulseBody::MicroTrain` vào `ComputeFlowCoordinator`, `RendezvousRegistry` và `OpportunisticWorkerEngine` (tích hợp gom mảnh `FountainDecoder`, huấn luyện vi mô có checkpoint, kiểm chứng Canary, nhận `SynapticVoucher` từng lát và kích hoạt `build_yield_sweep_tx`).

### 5.6. Tầng Cảm Biến Hệ Điều Hành, Daemon & Bộ Lệnh CLI (`crates/skynet-node`)
- **`governor/` (`mod.rs`, `probe_macos.rs`, `probe_linux.rs`, `probe_windows.rs`, `checkpoint_store.rs`)** (`FR-004`, `FR-005`, `SC-001`, `US2`):
  - Vòng lặp giám sát nền thích nghi (`250ms` khi hoạt động / `1500ms` khi tạm dừng) qua `tokio::task::spawn_blocking`, tự động chọn probe theo `cfg!(target_os)` không cần quyền `root` và đối chiếu danh sách tiến trình Game mặc định (`cs2`, `valorant`, `dota2`, `leagueclient`, `cyberpunk2077`, `eldenring`, `genshinimpact`, `baldur`, `unreal`, `blender`) cộng danh sách `--game-watchlist` của người dùng.
  - `checkpoint_store.rs`: Lưu/khôi phục nguyên tử `MicroTrainCheckpoint` (`.tmp -> rename`, `< 5ms`) và xuất tệp trạng thái `worker_studio_state.json` cho Dashboard đọc trực tiếp.
- **`genesis_io.rs` & `runtime_p2p.rs` (`skynet-node`)** (`FR-001`..`FR-003`, `FR-011`, `US1`):
  - Mở rộng `NodeConfig` thêm `genesis_path: Option<PathBuf>`.
  - `runtime_p2p.rs`: Nạp `GENESIS_HASH` & UTXO khởi nguyên từ `genesis.json` (giữ chế độ khởi tạo mặc định khi chạy đơn lẻ không truyền `--genesis` để tương thích ngược 100% với các test cũ), kích hoạt `EpidermalFilter`, gửi `PheromoneHello` mỗi `25s` tới `seed_peers`, phát/nhận `LanBeacon` và bắn loạt `PunchProbe` đục lỗ NAT khi nhận `PunchRendezvous`.
- **Bộ CLI `skynet` (`cli/*`) & `skynet-agent` (`worker_cli.rs`, `bin/skynet_agent.rs`)** (`FR-011`, `US1`..`US5`):
  - Bảo toàn 100% chuỗi đầu ra và cờ hiện hữu được kiểm tra bởi `skynet_cli_test.rs` & `skynet_agent_cli_test.rs`.
  - Bổ sung trên `skynet`: `genesis generate --output <PATH> [--network <NAME>]`, `genesis verify --path <PATH>`, `compute train --input <PATH> [--epochs <N>] [--bounty <SKN>] [--target <ADDR>]`, nâng cấp `compute dispatch` bắn gói UDP thực tế và nâng cấp `airdrop claim` tính toán `PopeChallenge` + xác nhận trên `MarketingBootstrapVault`.
  - Bổ sung trên `skynet-agent`: `--opportunistic`, `--owner <HEX_PUBKEY>`, `--seed <ADDR>`, `--genesis <PATH>`, `--schedule <HH:MM-HH:MM>`, `--auto-sweep-skn <u64>`, `--game-watchlist <csv>`, `--max-vram-pct <u8>`, `--idle-hold-secs <u64>`, chạy vòng lặp `tokio::select!` thật trên `UdpSocket` + `OsTelemetryGovernor` + `OpportunisticWorkerEngine` (xóa bỏ hoàn toàn bộ đếm cộng tiền giả `+15 SKN`).

### 5.7. Tầng Mô Phỏng Hỗn Loạn (`crates/skynet-sim`) & Trạm Giám Sát 60fps (`crates/skynet-dashboard`)
- **`crates/skynet-sim/src/opportunistic_sim.rs`** (`SC-001`, `SC-003`, `SC-004`, `SC-005`):
  - Kịch bản mô phỏng tích hợp kiểm chứng đồng thời: rớt gói UDP `40%` trên `FountainDroplet`, ngắt Game thủ `< 100ms` + nối lại từ `MicroTrainCheckpoint`, `30%` nút Byzantine bị `CanarySpotCheck` cắt cọc + `TrimmedMeanAggregator` hội tụ loss, và thanh toán từng lát + `Yield Sweep` bảo toàn chính xác `21T SKN`.
- **`crates/skynet-dashboard` (`lib.rs`, `ws_rfc6455.rs`, `worker_studio_api.rs`, `assets/dashboard.html`, `assets/worker_studio.js`)** (`FR-012`, `SC-007`, `US5`):
  - `lib.rs`: Re-export công khai các module router, `LiveWorld`, WebSocket và Studio API để cả `src/main.rs` lẫn integration test in-process (`127.0.0.1:0`, không cần `tokio::process`) dùng chung trực tiếp.
  - `ws_rfc6455.rs`: Bộ mã hóa SHA-1 + Base64 chuẩn RFC 6455 thuần Rust và codec đóng khung văn bản độc lập kiểu dữ liệu (nhận chuỗi JSON `&str`, không phụ thuộc `dto.rs`), phục vụ nâng cấp `GET /ws/telemetry` ở chu kỳ `16ms` (`~60fps`).
  - `worker_studio_api.rs`: Phục vụ `GET /api/worker-studio` và các hành động điều khiển mới trên `POST /api/control?action=sim_game_launch|sim_thermal_spike|sim_canary_fraud|sim_yield_sweep`, kết hợp đọc trạng thái thực từ `worker_studio_state.json` trên đĩa nếu có tiến trình `skynet-agent` đang chạy thật.
  - `assets/dashboard.html` & `assets/worker_studio.js`: Tích hợp kết nối WebSocket `60fps` tự động chuyển dự phòng sang HTTP polling và hiển thị khu vực **Worker Earnings & Opportunistic Studio** trực quan.

---

## 6. Project Structure

### 6.1. Feature Artifacts

```text
specs/006-opportunistic-edge-network/
├── spec.md                      # Đặc tả yêu cầu nghiệp vụ & tiêu chí nghiệm thu (Vietnamese)
├── checklists/
│   └── requirements.md          # Checklist kiểm định chất lượng đặc tả Spec-Kit
├── plan.md                      # Kiến trúc kỹ thuật & kiểm toán Constitution (tệp này)
└── tasks.md                     # Danh sách tác vụ phân pha song song cho Wave-2 Swarm
```

### 6.2. Target Source Code Layout (Mọi tệp `.rs` chạm tới đều `<= 230` dòng)

```text
crates/
├── skynet-dna/src/
│   ├── lib.rs                   # [SPLIT] ~55 dòng: re-exports
│   ├── genes.rs                 # [SPLIT] ~160 dòng: CellRole, Conserved/AdaptiveGenes, EpigeneticMask
│   ├── cell_dna.rs              # [SPLIT] ~210 dòng: CellDna lifecycle & crossover
│   ├── dna_tests.rs             # [SPLIT] ~110 dòng: unit tests gốc
│   └── plasmid_patch.rs         # [NEW]   ~165 dòng: PlasmidPatch & guided evolution
├── skynet-cell/src/
│   ├── lib.rs                   # [SPLIT] ~70 dòng: re-exports
│   ├── hardware.rs              # [SPLIT] ~130 dòng: HardwareTier, HardwareProfile, CellError
│   ├── micro_cell.rs            # [SPLIT] ~180 dòng: MicroCell & TissueBatch
│   ├── cell_tests.rs            # [SPLIT] ~100 dòng: unit tests gốc
│   ├── preempt_guard.rs         # [NEW]   ~165 dòng: PreemptGuard, SliceLatencyGuard, MicroTrainCheckpoint
│   ├── gradient_codec.rs        # [NEW]   ~155 dòng: half::f16 & Sparse Top-K gradient codec + L2 norm
│   ├── micro_train.rs           # [NEW]   ~215 dòng: Candle Var/GradStore micro-batch trainer
│   ├── canary_pouw.rs           # [NEW]   ~185 dòng: BLAKE3 XOF Canary generator & Cosine verifier
│   └── federated_agg.rs         # [NEW]   ~170 dòng: Byzantine TrimmedMeanAggregator
├── skynet-ledger/src/
│   ├── lib.rs                   # [SPLIT] ~110 dòng: LedgerError & re-exports
│   ├── nano_skn.rs              # [SPLIT] ~160 dòng: NanoSkn & 21T supply math
│   ├── halving_vault.rs         # [SPLIT] ~160 dòng: GenesisHalvingVault
│   ├── custody.rs               # [SPLIT] ~220 dòng: CustodyPolicy, TxOutput, SknTransaction, HumanWallet
│   ├── voucher.rs               # [SPLIT] ~210 dòng: SynapticVoucher & VoucherChannelTracker
│   ├── utreexo.rs               # [SPLIT] ~210 dòng: StatelessUtreexoRoots
│   ├── dag_ledger.rs            # [SPLIT] ~225 dòng: UtxoDagLedger & AgentMetabolicBank
│   ├── ledger_core_tests.rs     # [SPLIT] ~220 dòng: unit tests cung tiền & Utreexo
│   ├── ledger_custody_tests.rs  # [SPLIT] ~220 dòng: unit tests lưu ký & voucher
│   ├── compute_escrow_types.rs  # [MOD]   ~155 dòng: TaskId, + EscrowStatus::Slashed & chunk milestone types/errors
│   ├── compute_escrow.rs        # [SPLIT] ~195 dòng: ComputeEscrowVault core (pub(crate) records)
│   ├── compute_escrow_refund.rs # [SPLIT] ~140 dòng: timeout refund queries/actions
│   ├── genesis.rs               # [NEW]   ~190 dòng: GenesisConfig, GENESIS_HASH, 85%/15% invariant
│   ├── escrow_slash.rs          # [NEW]   ~185 dòng: PoUW fraud slashing & WorkerReputation
│   ├── streaming_escrow.rs      # [NEW]   ~190 dòng: per-chunk milestone settlement
│   └── yield_sweep.rs           # [NEW]   ~180 dòng: Soul-Body automatic cold-wallet Yield Sweep
├── skynet-wire/src/
│   ├── lib.rs                   # [SPLIT] ~80 dòng: re-exports
│   ├── packet.rs                # [SPLIT] ~190 dòng: ImpulseBody, ImpulsePacket, WireError
│   ├── wire_tests.rs            # [SPLIT] ~180 dòng: unit tests gốc
│   ├── mcp_capsule.rs           # [SPLIT] ~160 dòng: OpcodeDomain (+ 0x13 MicroTrainEvolution)
│   ├── mcp_capsule_tests.rs     # [SPLIT] ~75 dòng: capsule unit tests
│   ├── opcode_domains.rs        # [SPLIT] ~165 dòng: DOMAIN_CATEGORIES (+ 0x1300 catalog)
│   ├── opcode_registry.rs       # [SPLIT] ~130 dòng: descriptor lookup (+ 0x1301..0x1303)
│   ├── epidermal_filter.rs      # [NEW]   ~175 dòng: <5µs zero-alloc GENESIS_HASH & ±30s filter
│   ├── rendezvous_wire.rs       # [NEW]   ~195 dòng: PheromoneHello, PunchRendezvous, LanBeacon, GangliaRelay
│   └── micro_train_wire.rs      # [NEW]   ~190 dòng: MicroBatchManifest, WeightDeltaClaim, ChunkVoucherDelivery
├── skynet-agent/
│   ├── src/
│   │   ├── lib.rs               # [SPLIT] ~95 dòng: AgentError & re-exports
│   │   ├── snowball.rs          # [SPLIT] ~180 dòng: SnowballEngine
│   │   ├── organism_state.rs    # [SPLIT] ~220 dòng: AgentNode cellular lifecycle
│   │   ├── datagram_router.rs   # [SPLIT] ~215 dòng: UDP datagram routing (wired to Compute/Rendezvous/MicroTrain)
│   │   ├── agent_core_tests.rs  # [SPLIT] ~210 dòng: unit tests gốc (phần 1)
│   │   ├── agent_network_tests.rs # [SPLIT] ~210 dòng: unit tests gốc (phần 2)
│   │   ├── rendezvous_broker.rs # [NEW]   ~195 dòng: RendezvousRegistry, NAT hole-punch burst & relay
│   │   ├── governor_fsm.rs      # [NEW]   ~190 dòng: GovernorState FSM, 8°C hysteresis & 30s hold
│   │   ├── workload_mux.rs      # [NEW]   ~180 dòng: 3-tier WorkloadMultiplexer & PowerProfitabilityGuard
│   │   └── opportunistic_worker.rs # [NEW] ~215 dòng: OpportunisticWorkerEngine end-to-end coordinator
│   └── tests/
│       └── micro_train_canary_settlement_test.rs # [NEW] ~180 dòng: integration test PoUW + streaming + sweep
├── skynet-node/
│   ├── src/
│   │   ├── lib.rs               # [MOD]   ~45 dòng: re-exports governor, genesis_io, worker_cli
│   │   ├── config.rs            # [MOD]   ~185 dòng: + --genesis <PATH>
│   │   ├── genesis_io.rs        # [NEW]   ~135 dòng: canonical JSON read/write & verification
│   │   ├── runtime.rs           # [MOD]   ~175 dòng: NodeRuntime event loop delegated to runtime_p2p
│   │   ├── runtime_p2p.rs       # [NEW]   ~185 dòng: EpidermalFilter, 25s seed ping, NAT punch & LAN beacon
│   │   ├── worker_cli.rs        # [NEW]   ~185 dòng: Opportunistic Worker CLI config & argument parser
│   │   ├── governor/
│   │   │   ├── mod.rs           # [NEW]   ~165 dòng: adaptive 250ms/1500ms OsTelemetryGovernor
│   │   │   ├── probe_macos.rs   # [NEW]   ~165 dòng: ioreg/pmset/sysctl/lsappinfo rootless probe
│   │   │   ├── probe_linux.rs   # [NEW]   ~165 dòng: /proc, /sys, drm & nvidia-smi probe
│   │   │   ├── probe_windows.rs # [NEW]   ~150 dòng: tasklist/nvidia-smi & SimulatedTelemetryProbe
│   │   │   └── checkpoint_store.rs # [NEW] ~155 dòng: atomic .tmp->rename checkpoint & studio state writer
│   │   ├── cli/
│   │   │   ├── mod.rs           # [SPLIT] ~95 dòng: flag parser & hex codec
│   │   │   ├── wallet_opcode_cmd.rs # [SPLIT] ~145 dòng: wallet & opcode commands
│   │   │   ├── genesis_airdrop_cmd.rs # [NEW] ~165 dòng: genesis generate/verify & real PoPE airdrop
│   │   │   └── compute_cmd.rs   # [NEW]   ~195 dòng: presets, real UDP dispatch & compute train
│   │   └── bin/
│   │       ├── skynet.rs        # [SPLIT] ~65 dòng: thin CLI dispatcher
│   │       └── skynet_agent.rs  # [MOD]   ~175 dòng: real UDP opportunistic worker daemon
│   └── tests/
│       ├── genesis_cli_test.rs  # [NEW]   ~120 dòng: CLI genesis generate/verify roundtrip
│       ├── p2p_rendezvous_hole_punch_test.rs # [NEW] ~150 dòng: Seed + 2 NAT peers UDP hole punch
│       └── opportunistic_governor_test.rs # [NEW] ~150 dòng: <100ms preemption & atomic checkpoint resume
├── skynet-sim/src/
│   ├── main.rs                  # [SPLIT] ~95 dòng: simulation runner entrypoint
│   ├── classic_sim.rs           # [SPLIT] ~215 dòng: existing multi-agent scenarios
│   ├── sim_metrics.rs           # [SPLIT] ~150 dòng: reporting & invariant assertions
│   └── opportunistic_sim.rs     # [NEW]   ~210 dòng: 40% loss + preemption + 30% Byzantine + Yield Sweep sim
└── skynet-dashboard/
    ├── assets/
    │   ├── dashboard.html       # [MOD]   WebSocket 60fps hook + Worker Studio mount container
    │   └── worker_studio.js     # [NEW]   ~195 dòng: modular 60fps WS client & Worker Earnings Studio UI
    ├── src/
    │   ├── lib.rs               # [NEW]   ~55 dòng: re-exports server/router/world cho binary & integration tests
    │   ├── main.rs              # [SPLIT] ~65 dòng: thin TCP/WS listener bootstrap
    │   ├── dto.rs               # [SPLIT] ~175 dòng: dashboard & WorkerStudioTelemetryDto
    │   ├── live_world.rs        # [SPLIT] ~215 dòng: 8-node live cluster tick
    │   ├── world_actions.rs     # [SPLIT] ~205 dòng: interactive simulation & live disk state loader
    │   ├── http_router.rs       # [SPLIT] ~190 dòng: HTTP/1.1 & WS upgrade router
    │   ├── ws_rfc6455.rs        # [NEW]   ~175 dòng: pure-Rust RFC 6455 SHA-1/Base64 + generic &str frame stream
    │   ├── worker_studio_api.rs # [NEW]   ~190 dòng: /api/worker-studio & gamer/thermal/canary triggers
    │   └── preprocess_api.rs    # [KEEP]  64 dòng: giữ nguyên
    └── tests/
        └── ws_worker_studio_test.rs # [NEW] ~140 dòng: in-process 127.0.0.1:0 RFC 6455 & studio API test
```

---

## 7. Chiến Lược Phân Lát Song Song Cho Wave-2 Swarm (Zero File Collision)

Thiết kế trên ánh xạ 1-1 với **6 lát cắt tác tử độc lập theo Crate** trong `tasks.md` để các tác tử thực thi (Wave-2 Rust Authors) chạy song song mà **không bao giờ chạm chung một tệp `.rs`**:

1. **Đợt 1 — Phase 1 Prerequisite Splits (Tối đa 6 tác tử chạy song song cùng lúc trên 8 crates độc lập):**
   - `Author dna-cell` (`skynet-dna`, `skynet-cell`), `Author ledger` (`skynet-ledger`), `Author wire` (`skynet-wire`), `Author agent` (`skynet-agent`), `Author node-cli` (`skynet-node`), `Author sim-dashboard` (`skynet-sim`, `skynet-dashboard`).
2. **Đợt 2 — Lower-Tier Crates (3 tác tử song song 100% khác crate):**
   - **`Author dna-cell`**: `PreemptGuard`, `gradient_codec`, `PlasmidPatch`, `micro_train` -> `canary_pouw` -> `federated_agg`.
   - **`Author ledger`**: `genesis.rs`, `compute_escrow_types.rs` + `escrow_slash.rs`, `streaming_escrow.rs`, `yield_sweep.rs`.
   - **`Author wire`**: Miền Opcode `0x13`, `epidermal_filter.rs`, `rendezvous_wire.rs`, `micro_train_wire.rs`.
3. **Đợt 3 — Mid-Tier Mesh & Coordinator (`skynet-agent`):**
   - **`Author agent`**: `rendezvous_broker.rs`, `governor_fsm.rs` (xuất `SystemTelemetrySnapshot` & `GovernorState` làm tiền đề cho các OS probe của `skynet-node`), `workload_mux.rs`, `opportunistic_worker.rs` & nối thật `datagram_router.rs`.
4. **Đợt 4 — Application Leaf Crates (2 nhánh song song 100% khác crate sau khi `skynet-agent` sẵn sàng):**
   - **`Author node-cli` (`skynet-node`)**: `governor/probe_{macos,linux,windows}.rs`, `checkpoint_store.rs`, `governor/mod.rs`, `genesis_io.rs`, `runtime_p2p.rs`, `cli/{genesis_airdrop_cmd,compute_cmd}.rs`, `worker_cli.rs`, `bin/skynet_agent.rs` & bộ integration tests P2P/Governor/Genesis.
   - **`Author sim-dashboard` (`skynet-sim` + `skynet-dashboard`)**: Kịch bản hỗn loạn `opportunistic_sim.rs` cùng `ws_rfc6455.rs`, `worker_studio_api.rs`, `assets/worker_studio.js`, `assets/dashboard.html` & integration test WebSocket `60fps`.

# Tasks: Remote Compute & Candle Execution Protocol

**Input**: Design documents from `specs/001-remote-compute-candle/` (`spec.md`, `plan.md`)

## Format: `- [ ] [TaskID] [P?] [Story?] Description with file path`

- **`[P]`**: Parallelizable (different files, no blocking dependency within phase)
- **`[US#]`**: Maps task to User Story from `spec.md` (`[US1]`, `[US2]`, `[US3]`)

---

## Phase 1: Setup

- [x] T001 Initialize remote compute feature module scaffolding across workspace crates.

---

## Phase 2: Foundational (Blocking Prerequisites)

- [x] T002 [P] Implement `crates/skynet-wire/src/compute_wire.rs` defining `ComputeTaskManifest`, `ComputeMessage`, `ComputeBid`, `ComputeCompletionClaim`, and `WireError` variants. (`FR-001`, `FR-002`)
- [x] T003 Expose `compute_wire` and integrate `ImpulseBody::Compute` in `crates/skynet-wire/src/lib.rs` while verifying UDP packet size limits <= 1200 bytes. (`FR-002`)
- [x] T004 [P] Implement `crates/skynet-ledger/src/compute_escrow.rs` defining `ComputeEscrowVault`, `EscrowRecord`, dual-staking lock (`bounty` + `bond`), completion payout, and slashing refund logic. (`FR-004`, `FR-005`)
- [x] T005 Expose `compute_escrow` in `crates/skynet-ledger/src/lib.rs`. (`FR-004`)

**Checkpoint**: Core wire protocols and ledger escrow foundations ready for User Stories.

---

## Phase 3: User Story 1 - Candle Micro-Neural Compute (Priority: P1 — MVP)

**Goal**: Enable Worker Node to execute Candle neural computation on CUDA / Metal / CPU and return authenticated result tensor.  
**Independent Test**: Unit test in `crates/skynet-cell/src/compute_candle.rs` executing neural forward pass, calculating BLAKE3 digest and signing result.

- [x] T006 [P] [US1] Implement `crates/skynet-cell/src/compute_candle.rs` defining `CandleComputeExecutor`, `CandleTaskInput`, `CandleTaskOutput`, hardware device selection (CUDA/Metal/CPU), and output hashing. (`FR-006`, `FR-007`)
- [x] T007 [US1] Expose `compute_candle` in `crates/skynet-cell/src/lib.rs` and add unit test verifying execution correctness. (`FR-006`, `FR-007`)
- [x] T008 [US1] Implement `crates/skynet-agent/src/compute_flow.rs` orchestrating end-to-end task broadcast, bid matching, Candle execution, and result return. (`FR-008`)
- [x] T009 [US1] Expose `compute_flow` in `crates/skynet-agent/src/lib.rs` and add integration test verifying client-worker computation over simulated UDP datagrams. (`FR-008`, `SC-001`, `SC-002`)

**Checkpoint**: User Story 1 (MVP) fully functional and testable.

---

## Phase 4: User Story 2 - Dual-Staking Escrow Bounty Settlement (Priority: P2)

**Goal**: Protect economic incentives by locking Client bounty and Worker bond in ledger escrow, executing automatic payout on valid result or slashing on timeout.  
**Independent Test**: Unit tests in `crates/skynet-ledger/src/compute_escrow.rs` verifying escrow state transitions (`Locked` -> `Completed` / `Refunded`) and exact `NanoSkn` conservation.

- [x] T010 [US2] Add unit tests in `crates/skynet-ledger/src/compute_escrow.rs` for happy path (bounty + bond payout to worker) and failure path (timeout refund + slashing burn). (`FR-004`, `FR-005`, `SC-003`)
- [x] T011 [US2] Integrate escrow settlement into `crates/skynet-agent/src/compute_flow.rs` ensuring client co-signs release or claims refund upon timeout. (`FR-005`, `FR-008`)
- [x] T018 [US2] Implement `AuditorDeposit`, `EscrowSplitConfig`, and `complete_task_with_auditors` in `crates/skynet-ledger/src/compute_escrow.rs` and `compute_escrow_types.rs`. (`FR-009`)
- [x] T019 [US2] Implement `AuditorCoordinator` in `crates/skynet-agent/src/auditor_flow.rs` orchestrating auditor bidding, attestation verification, and quorum settlement. (`FR-010`)

**Checkpoint**: User Story 2 fully functional with verified economic invariants.

---

## Phase 5: User Story 3 - Multi-droplet Fountain Code Streaming (Priority: P3)

**Goal**: Support transferring large compute inputs/outputs exceeding 1200 bytes using GF(2) Fountain erasure droplets over lossy UDP.  
**Independent Test**: Unit test in `crates/skynet-wire/src/compute_wire.rs` demonstrating chunking of 8KB payload into droplets, random 30% drop, and 100% reconstruction.

- [x] T012 [US3] Implement Fountain droplet segmentation and reassembly helpers for compute payloads in `crates/skynet-wire/src/compute_wire.rs`. (`FR-003`)
- [x] T013 [US3] Add unit test in `crates/skynet-wire/src/compute_wire.rs` verifying loss-tolerant reconstruction of compute tensors under packet drop. (`FR-003`, `SC-001`)

---

## Phase 6: Polish & Verification

- [x] T014 Run `cargo check --workspace` to verify zero compiler errors.
- [x] T015 Run `cargo clippy --workspace --all-targets` and fix any warnings.
- [x] T016 Run `cargo test --workspace` and confirm 100% test pass across all 7 crates.
- [x] T017 Audit file length across all new files to ensure strict compliance with the <= 350 lines limit. (`SC-004`)

---

## Dependencies & Execution Strategy

- **Execution Order**: Phase 1 -> Phase 2 -> Phase 3 (US1 MVP) -> Phase 4 (US2) -> Phase 5 (US3) -> Phase 6 (Polish & Verification).
- **MVP Scope**: Phase 1 through Phase 3 delivers a working, testable Candle remote compute engine.

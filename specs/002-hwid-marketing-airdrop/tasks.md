# Task Checklist: Cơ chế Marketing Airdrop theo HWID chống Gian lận & Lạm phát

## Phase 1: Setup & Project Documentation
- [x] T001: Cập nhật tài liệu kiến trúc kỹ thuật `docs/MARKETING_AIRDROP_SPEC.md` và liên kết trong `docs/ARCHITECTURE.md`.
- [x] T002: Đăng ký feature active trong `.specify/feature.json`.

## Phase 2: Foundational & Ledger Module (`skynet-ledger`)
- [x] T003: Xây dựng module `crates/skynet-ledger/src/hwid_airdrop.rs` với các struct `MarketingBootstrapVault`, `HwidClaimRecord` và enum `MarketingError`.
- [x] T004: Hiện thực logic tính toán đường cong halving và phương thức `claim_reward` đảm bảo bất biến bảo toàn `NanoSkn`.
- [x] T005: Xuất bản và re-export module trong `crates/skynet-ledger/src/lib.rs`.
- [x] T006: Viết bộ unit tests cho `MarketingBootstrapVault` (double claim, halving, reserve exhaustion).

## Phase 3: Hardware Attestation & Candle PoPE (`skynet-cell`)
- [x] T007: Xây dựng module `crates/skynet-cell/src/pope_benchmark.rs` thực thi Candle Tensor memory-hard PoPE.
- [x] T008: Hiện thực hàm `verify_pope_proof` xác thực kết quả và kiểm tra độ trễ thực thi.
- [x] T009: Xuất bản và re-export module trong `crates/skynet-cell/src/lib.rs`.
- [x] T010: Viết unit tests cho PoPE benchmark và bộ kiểm chứng.

## Phase 4: Polish & Workspace Verification
- [x] T011: Chạy kiểm tra kiểu `cargo check --workspace`.
- [x] T012: Chạy linter `cargo clippy --workspace --all-targets`.
- [x] T013: Chạy toàn bộ test suites `cargo test --workspace`.
- [x] T014: Kiểm toán lại giới hạn vật lý (File <= 250 dòng, hàm <= 40 dòng) và hoàn tất checklist.

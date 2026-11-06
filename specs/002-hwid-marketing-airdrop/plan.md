# Implementation Plan: Cơ chế Marketing Airdrop theo HWID chống Gian lận & Lạm phát

## 1. Kiến trúc Kỹ thuật & Bố cục Module

Cơ chế được chia nhỏ thành 2 crate độc lập theo nguyên tắc Separation of Concerns:
1. **`skynet-ledger` (`crates/skynet-ledger/src/hwid_airdrop.rs`)**:
   - Chịu trách nhiệm quản lý quỹ dự trữ `MarketingBootstrapVault`, lưu vết các thiết bị đã nhận thưởng `HwidClaimRecord`, tính toán đường cong halving, tạo `TxOutput` với chính sách `CustodyPolicy::AgentBond`, và đảm bảo bất biến bảo toàn token.
2. **`skynet-cell` (`crates/skynet-cell/src/pope_benchmark.rs`)**:
   - Chịu trách nhiệm tạo và kiểm chứng Bằng chứng Thực thi Vật lý (`Proof of Physical Execution - PoPE`).
   - Sử dụng thư viện `candle-core` để thực thi tính toán tensor nơ-ron memory-hard, bão hòa bộ đệm và tính BLAKE3 digest xác thực kèm đo lường thời gian thực thi.

## 2. Thiết kế Cấu trúc Dữ liệu & API

### `skynet-ledger::hwid_airdrop`
- `HwidDigest = [u8; 32]`
- `ProofDigest = [u8; 32]`
- `HwidClaimRecord`:
  - `hwid_digest: HwidDigest`
  - `claimed_epoch: u64`
  - `allocated_bond: NanoSkn`
  - `utxo_id: UtxoId`
  - `proof_digest: ProofDigest`
- `MarketingBootstrapVault`:
  - `total_allocated: NanoSkn`
  - `remaining_reserve: NanoSkn`
  - `total_devices_registered: u64`
  - `base_reward: NanoSkn`
  - `halving_step_devices: u64`
  - `claimed_devices: HashMap<HwidDigest, HwidClaimRecord>`
  - API: `new(reserve: NanoSkn, base_reward: NanoSkn, halving_step: u64) -> Self`
  - API: `calculate_current_reward(&self) -> NanoSkn`
  - API: `claim_reward(&mut self, hwid: HwidDigest, human_owner: [u8; 32], agent_pubkey: [u8; 32], epoch: u64, proof_digest: ProofDigest, tx_hash: [u8; 32], vout: u16) -> Result<(NanoSkn, TxOutput), MarketingError>`

### `skynet-cell::pope_benchmark`
- `PopeProof`:
  - `result_digest: [u8; 32]`
  - `duration_micros: u64`
- API: `compute_pope_challenge(device: &Device, hwid: &[u8; 32], challenge_nonce: &[u8; 32], dim: usize) -> Result<PopeProof, ComputeCandleError>`
- API: `verify_pope_proof(hwid: &[u8; 32], challenge_nonce: &[u8; 32], proof: &PopeProof, max_allowed_micros: u64, dim: usize) -> bool`

## 3. Kế hoạch Kiểm tra Giới hạn Vật lý (Constitution Alignment)
- `hwid_airdrop.rs`: ~200 dòng (dưới giới hạn 250 dòng).
- `pope_benchmark.rs`: ~180 dòng (dưới giới hạn 250 dòng).
- Mỗi hàm <= 40 dòng, sử dụng guard clauses và `?` toán tử xử lý lỗi.
- Đầy đủ unit tests cho các trường hợp biên:
  - Khởi tạo quỹ và nhận thưởng lần đầu.
  - Từ chối nhận thưởng trùng HWID.
  - Hạ thưởng theo chu kỳ halving.
  - Cạn kiệt quỹ dự trữ (`ReserveExhausted`).
  - Kiểm tra PoPE đúng digest và trượt khi digest sai / timeout.

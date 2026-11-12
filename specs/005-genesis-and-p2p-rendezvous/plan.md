# Technical Implementation Plan: Nghi Thức Khởi Nguyên Genesis & Đục Lỗ NAT UDP P2P Rendezvous

## 1. Kiến Trúc & Thiết Kế Tổng Thể (System Architecture)

Tính năng này kết nối chặt chẽ giữa 4 crates (`skynet-ledger` -> `skynet-wire` -> `skynet-agent` -> `skynet-node`) nhằm hoàn thiện tầng giao thức mạng thực tế và định danh khởi nguyên bất biến:

```
┌─────────────────────────────────────────────────────────────┐
│                    skynet-ledger (Genesis)                  │
│  - genesis.rs : GenesisConfig, GenesisAllocation            │
│                 Blake3 canonical hashing -> GENESIS_HASH    │
│                 Invariant verification (21T SKN: 85% / 15%) │
└──────────────────────────────┬──────────────────────────────┘
                               │ GENESIS_HASH (32B)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    skynet-wire (Wire Protocol)              │
│  - rendezvous_wire.rs : PheromoneHello, PunchRendezvous     │
│  - EpidermalFilter    : Stack zero-alloc network_id check   │
│                         Anti-replay (timestamp +-30s)       │
└──────────────────────────────┬──────────────────────────────┘
                               │ Wire Packets (<= 1200B)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    skynet-agent (P2P Mesh)                  │
│  - rendezvous_broker.rs : In-memory RendezvousRegistry      │
│                           Peer matching & Hole-punch broker │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    skynet-node (CLI & Daemon)               │
│  - bin/skynet.rs      : CLI `genesis generate/verify`       │
│  - config.rs          : `--seed <ADDR>`, `--genesis <PATH>` │
│  - runtime.rs         : Seed Pheromone ping & NAT puncher   │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Thiết Kế Chi Tiết Từng Module

### 2.1. Phân Hệ Sổ Cái Genesis (`crates/skynet-ledger/src/genesis.rs`)
- **`GenesisConfig` & `GenesisAllocation`**:
  - `network_name: String` (mặc định: `"skynet-mainnet-v1"` hoặc `"skynet-testnet-alpha"`).
  - `total_supply: NanoSkn` (bắt buộc đúng $21{,}000{,}000{,}000{,}000\text{ SKN}$).
  - `vault_allocation: NanoSkn` ($85\%$ - $17.85\text{T SKN}$).
  - `circulating_allocation: NanoSkn` ($15\%$ - $3.15\text{T SKN}$).
  - `initial_utxos: Vec<GenesisUtxoEntry>`.
  - `annual_halving_epochs: u64` (mặc định `525_600`).
- **`compute_genesis_hash(config: &GenesisConfig) -> [u8; 32]`**:
  - Tuần tự hóa JSON chuẩn hóa (canonical formatting với key có thứ tự).
  - Băm bằng `blake3::hash` trả về mảng 32 bytes `GENESIS_HASH`.
- **`validate_genesis_invariants(config: &GenesisConfig) -> Result<(), LedgerError>`**:
  - Kiểm tra toán học: $\text{vault\_allocation} + \sum \text{initial\_utxos} \equiv \text{total\_supply}$.
  - Kiểm tra $100\%$ không tràn số và không thiếu hụt dù chỉ $1\text{ nanoSKN}$.

### 2.2. Phân Hệ Khung Tin & Màng Lọc Tế Bào Da (`crates/skynet-wire/src/rendezvous_wire.rs`)
- **`PheromoneHello`**:
  - `sender_pubkey: [u8; 32]`
  - `internal_addr: SocketAddr`
  - `node_tier: u8` (0: Tiny, 1: Standard, 2: Colossal)
  - `timestamp_sec: u64`
  - `nonce: u64`
  - `signature: [u8; 64]`
- **`PunchRendezvous`**:
  - `target_peer_pubkey: [u8; 32]`
  - `peer_public_addr: SocketAddr`
  - `rendezvous_token: u64`
  - `expiry_timestamp: u64`
  - `signature: [u8; 64]`
- **`EpidermalFilter`**:
  - `verify_network_and_timestamp(raw_network_id: &[u8; 32], expected_network_id: &[u8; 32], packet_ts: u64, local_ts: u64) -> Result<(), WireError>`
  - Thực thi hoàn toàn trên stack, $< 5\mu\text{s}$, không cấp phát heap memory.

### 2.3. Phân Hệ Điều Phối Điểm Hẹn (`crates/skynet-agent/src/rendezvous_broker.rs`)
- **`RendezvousRegistry`**:
  - Quản lý bảng tra cứu đồng thời `Arc<RwLock<HashMap<[u8; 32], PeerRendezvousEntry>>>`.
  - `PeerRendezvousEntry`: `public_addr: SocketAddr`, `internal_addr: SocketAddr`, `node_tier: u8`, `last_seen_sec: u64`.
  - `register_peer(entry)`: Tự động dọn rác các peer quá hạn ($> 120\text{s}$).
  - `pair_peers(peer_a, peer_b)`: Sinh mã ngẫu nhiên `rendezvous_token` và tạo 2 gói `PunchRendezvous`.

### 2.4. Phân Hệ Ứng Dụng CLI & Runtime Daemon (`crates/skynet-node`)
- **`src/bin/skynet.rs`**:
  - Thêm nhánh lệnh `genesis`:
    - `skynet genesis generate --output <PATH> [--network <NAME>]`
    - `skynet genesis verify --path <PATH>`
- **`src/config.rs`**:
  - Thêm trường `seeds: Vec<String>` (phân giải từ `--seed <ADDR>` hoặc biến môi trường `SKYNET_SEEDS`).
  - Thêm trường `genesis_path: Option<PathBuf>`.
- **`src/runtime.rs`**:
  - Khởi tạo `seed_ping_task`: Khi có cấu hình `seeds`, gửi định kỳ `PheromoneHello` mỗi 25s để giữ cổng NAT.
  - Xử lý khi nhận `PunchRendezvous`: Bắn xung UDP trực tiếp 3 lần tới `peer_public_addr` để đục lỗ NAT.

---

## 3. Constitution Compliance Check

| Tiêu Chuẩn Constitution | Hiện Trạng Thiết Kế | Trạng Thái |
| :--- | :--- | :---: |
| **Giới hạn File $\le 250$ dòng** | Mọi file mới (`genesis.rs`, `rendezvous_wire.rs`, `rendezvous_broker.rs`) đều được giới hạn dưới 220 dòng. | **Tuân Thủ** |
| **Giới hạn Hàm $\le 40$ dòng** | Các hàm parse, validate, hash, punch được chia nhỏ và dùng Guard Clauses thoát sớm. | **Tuân Thủ** |
| **Tham số $\le 3\text{--}4$** | Gom dữ liệu gói tin vào struct DTO rõ ràng (`PheromoneHello`, `PunchRendezvous`, `GenesisConfig`). | **Tuân Thủ** |
| **Xử lý Lỗi Tường Minh** | Dùng enum lỗi `LedgerError`, `WireError`, trả về `Result<T, E>` không unwrap. | **Tuân Thủ** |
| **Bảo Mật & Zero-Magic Numbers** | Hằng số `MAX_TIMESTAMP_DRIFT_SEC = 30`, `RENDEZVOUS_TTL_SEC = 120` đặt tại hằng số chuyên biệt. | **Tuân Thủ** |

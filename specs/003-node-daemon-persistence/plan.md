# Technical Implementation Plan: Daemon `skynet-node` & Tầng Lưu Trữ Bền Vững

## 1. Kiến Trúc & Thiết Kế Tổng Thể (System Architecture)

Crate mới `crates/skynet-node` được định vị tại tầng thực thi phân phối cao cấp của SkyNet, tương đương với `skynet-sim` và `skynet-dashboard`, phụ thuộc vào các thư viện lõi:

```
[skynet-dna] ──► [skynet-cell] ──► [skynet-agent] ──┐
                         ▲                          │
[skynet-ledger] ─────────┴───────► [skynet-wire] ───┴──► [skynet-node]
```

### Cấu Trúc Thư Mục & Modules (`crates/skynet-node`)
```
crates/skynet-node/
├── Cargo.toml
└── src/
    ├── lib.rs            # Thư viện core: re-export các submodules cho integration test
    ├── config.rs         # Cấu hình NodeConfig, parse cờ CLI (run, status)
    ├── identity.rs       # Quản trị khóa Ed25519 NodeIdentity (0600 file permission)
    ├── storage_types.rs  # Định nghĩa DTO Snapshot (Ledger, Organism, PeerTable)
    ├── storage.rs        # StorageEngine: Ghi tệp nguyên tử (.tmp -> rename), load/save
    ├── runtime.rs        # NodeRuntime: Tokio event loop, UDP socket, biological ticks, graceful shutdown
    └── main.rs           # CLI Entrypoint: Dispatching run/status commands
```

Mỗi file đều được thiết kế dưới 250 dòng (tối đa 350 dòng), hàm dưới 40 dòng theo quy chuẩn Constitution.

---

## 2. Thiết Kế Chi Tiết Từng Module

### 2.1. Module `config.rs` (`[FR-001]`)
- Struct `NodeConfig`:
  - `port: u16` (mặc định: `8888`)
  - `data_dir: PathBuf` (mặc định: `~/.skynet/data` hoặc fallback `./.skynet_data`)
  - `name: String`
  - `tier: HardwareTier` (TinyPeripheral / StandardNode / ColossalOrganism)
  - `seed_peers: Vec<SocketAddr>`
  - `owner_wallet: Option<[u8; 32]>`
  - `tick_interval_ms: u64` (mặc định: 1000ms)
  - `auto_flush_interval_sec: u64` (mặc định: 60s)
- Phương thức `NodeConfig::from_args(args: &[String]) -> Result<CliCommand, ConfigError>` hỗ trợ các subcommands:
  - `CliCommand::Run(NodeConfig)`
  - `CliCommand::Status { data_dir: PathBuf }`

### 2.2. Module `identity.rs` (`[FR-002]`)
- Struct `NodeIdentity`:
  - `signing_key: SigningKey`
  - `pubkey: [u8; 32]`
- Phương thức:
  - `NodeIdentity::load_or_generate(key_path: &Path) -> Result<Self, StorageError>`
  - Lưu trữ byte bí mật 32-byte của `SigningKey`, trên Unix thiết lập `std::fs::set_permissions(0o600)` để ngăn ngừa rò rỉ khóa.

### 2.3. Module `storage_types.rs` & `storage.rs` (`[FR-003]`)
- **Định dạng dữ liệu Snapshot:**
  - `LedgerSnapshot`: Danh sách toàn bộ UTXO hiện có, roots Utreexo, và sequence number.
  - `OrganismSnapshot`: Danh sách tế bào `MicroCell` (DNA, vai trò, năng lượng, số lần phân bào/tự hủy), số dư `BloodPool`, tổng `voucher_nano_earned`.
  - `PeerTableSnapshot`: Danh sách các `PeerContact` được lưu trong bảng `SynapticPeerTable`.
- **Cơ chế Atomic Write (`StorageEngine`):**
  - Hàm `atomic_write(target_path: &Path, data: &[u8]) -> Result<(), StorageError>`:
    1. Ghi ra `target_path.with_extension("tmp")`.
    2. Gọi `sync_all()` để đảm bảo dữ liệu ghi vật lý xuống đĩa cứng.
    3. Gọi `std::fs::rename(tmp_path, target_path)`.
  - Cơ chế này đảm bảo tệp dữ liệu không bao giờ bị corrupt kể cả khi bị tắt nguồn đột ngột.

### 2.4. Module `runtime.rs` (`[FR-004]`, `[FR-005]`)
- Struct `NodeRuntime`:
  - `agent: AgentNode`
  - `config: NodeConfig`
  - `storage: StorageEngine`
  - `udp_socket: Arc<UdpSocket>`
- **Luồng Vận Hành (Async Loop):**
  - Sử dụng `tokio::select!`:
    - Nhận gói tin UDP: `udp_socket.recv_from(&mut buf)` $\to$ `agent.handle_incoming_datagram(&buf)`.
    - Interval sinh học: Gọi `agent.run_biological_tick(&stimulus, &target)`.
    - Interval lưu trạng thái: Gọi `storage.save_all(&agent)`.
    - Tín hiệu hủy: `tokio::signal::ctrl_c()` $\to$ Kích hoạt `graceful_shutdown()`.
- **Graceful Shutdown:**
  - Dừng nhận I/O mới.
  - Đồng bộ `storage.save_all(&agent)`.
  - In log thông báo hoàn tất thoát an toàn.

### 2.5. Module `main.rs` & Status Inspection (`[FR-006]`)
- Đọc tham số CLI.
- Nếu là `status`: Nạp `storage` từ `data_dir` và hiển thị trực quan thông số node (Pubkey, Tier, số dư `NanoSkn`, số tế bào sống, peers).
- Nếu là `run`: Khởi chạy `NodeRuntime::run().await`.

---

## 3. Chiến Lược Kiểm Thử (Testing Strategy)

1. **Unit Test `identity.rs`:** Kiểm tra tạo mới và nạp lại khóa Ed25519 trùng khớp.
2. **Unit Test `storage.rs`:** Kiểm tra ghi/nạp `LedgerSnapshot`, `OrganismSnapshot` và khả năng chịu lỗi atomic write.
3. **Integration Test `persistence_roundtrip`:**
   - Khởi tạo node với một số dư ban đầu, cho tiến hóa sinh học qua 5 ticks.
   - Lưu trạng thái, hủy `AgentNode`.
   - Nạp lại từ `StorageEngine`: Kiểm tra số lượng tế bào, năng lượng, số dư `NanoSkn` bảo toàn chính xác.
4. **Integration Test `multi_node_local`:** Khởi chạy 2 node trên 2 cổng UDP khác nhau, gửi gói tin `PheromoneHello` / `SynapticMicroPay` thành công.

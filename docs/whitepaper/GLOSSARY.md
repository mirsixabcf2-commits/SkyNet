# SkyNet Whitepaper — Canonical Constants, Symbols & Terminology Glossary

This document defines the **invariant protocol constants, mathematical notation, and canonical technical identifiers** shared across all 16 language editions of the SkyNet Whitepaper. Translators and contributors must preserve exact Rust identifier names in backticks alongside localized natural-language terms.

---

## 1. Immutable Protocol & Monetary Constants

| Constant Identifier | Exact Value | Crate | Meaning |
| :--- | :--- | :--- | :--- |
| `PROTOCOL_VERSION` | `1` (`u16`) | `skynet-dna` | Genesis genome protocol version |
| `WIRE_MAGIC` | `*b"SK"` (`[0x53, 0x4B]`) | `skynet-wire` | 2-byte UDP `ImpulsePacket` header magic |
| `WIRE_VERSION` | `1` (`u8`) | `skynet-wire` | Binary UDP wire format version |
| `DEFAULT_PORT` | `8888` (`u16`) | `skynet-wire` | Default UDP & telemetry HTTP port |
| `MAX_UDP_MTU_BYTES` / `MAX_UDP_DATAGRAM_BYTES` | `1200` bytes | `skynet-dna` / `skynet-wire` | Strict fragmentation-free IPv4/IPv6 UDP payload ceiling |
| `MIN_SNOWBALL_QUORUM_BPS` | `6667` bps (`66.67%`) | `skynet-dna` | Minimum Snowball consensus supermajority ($\ge 2/3$) |
| `GENESIS_SUPPLY_TAG` | `*b"SKN21TRL"` | `skynet-dna` | Conserved 8-byte monetary supply gene tag |
| `GENESIS_MAX_SKN` | `21_000_000_000_000` (`u128`) | `skynet-ledger` | Hard-capped total supply: $2.1 \times 10^{13}\text{ SKN}$ (21 Trillion SKN) |
| `NANO_PER_SKN` | `1_000_000_000` (`10^9`) | `skynet-ledger` | Atomic subdivision factor: $1\text{ SKN} = 10^9\text{ nanoSKN}$ |
| `GENESIS_TOTAL_NANO_SKN` | `21_000_000_000_000_000_000_000` | `skynet-ledger` | Exact atomic cap: $2.1 \times 10^{22}\text{ nanoSKN}$ (backed by `u128`) |
| `MIN_TX_NANO_SKN` | `1` `nanoSKN` | `skynet-ledger` | Minimum transferable on-chain / off-chain / metabolic energy unit |
| `DEFAULT_EPOCHS_PER_YEAR` | `525_600` | `skynet-ledger` | 1-minute epochs per 365-day year ($365 \times 24 \times 60$) |
| `FOUNTAIN_BLOCK_BYTES` | `512` bytes | `skynet-wire` | Erasure block size inside each `FountainDroplet` (max 512 blocks = 256 KB) |

---

## 2. Default Neural & Evolutionary Hyperparameters

| Parameter | Default | Allowed Bounds | Role in `MicroCell` / `AgentNode` |
| :--- | :--- | :--- | :--- |
| `DEFAULT_INPUT_DIM` ($D_{\text{in}}$) | `8` | $> 0$ | Sensory stimulus vector dimension |
| `DEFAULT_HIDDEN_DIM` ($H$) | `16` | $> 0$ | Recurrent Liquid Time-Constant memory dimension |
| `DEFAULT_OUTPUT_DIM` ($D_{\text{out}}$) | `4` | $> 0$ | Action / scoring output vector dimension |
| `NUM_SPECIALIZED_OPERONS` ($K$) | `4` | Fixed `4` | Specialized projection heads (`Epidermal`, `Vascular`, `Visceral`, `Neural`) |
| `leak_rate` ($\tau$) | `0.35` | `[0.05, 0.95]` | Liquid state interpolation constant |
| `mutation_sigma` ($\sigma$) | `0.04` | `[0.001, 0.25]` (mitosis clamp `[0.005, 0.15]`) | Gaussian weight perturbation scale during `1 -> 2` Mitosis |
| `gossip_fanout` | `6` | `[4, 14]` | Peer fanout per gossip impulse |
| `pulse_interval_ms` | `80` ms | `[30, 150]` ms | Biological & consensus pulse cadence |
| `fec_redundancy_bps` | `1300` (`1.30x`) | `[1050, 1800]` (`1.05x..1.80x`) | Fountain erasure-coding transmission overhead |

---

## 3. Hardware Tier Profiles (`HardwareProfile::for_tier`)

| Tier (`HardwareTier`) | Target Device | `max_cells` | `basal_burn_per_tick` | `mitosis_threshold` | `mitosis_blood_fee` | `apoptosis_floor` |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `TinyPeripheral` | IoT sensor, MCU/SBC, weak CPU | `32` | `50 nanoSKN` (`10` for `Stem`) | `2,000 nanoSKN` | `100 nanoSKN` | `100 nanoSKN` |
| `StandardNode` | Personal laptop, desktop PC | `512` | `25 nanoSKN` (`5` for `Stem`) | `1,500 nanoSKN` | `60 nanoSKN` | `80 nanoSKN` |
| `ColossalOrganism` | Multi-core server, Metal/CUDA GPU | `16,384` | `10 nanoSKN` (`2` for `Stem`) | `1,000 nanoSKN` | `40 nanoSKN` | `50 nanoSKN` |

---

## 4. Standardized Mathematical Symbols

- $S_{\max} = 2.1 \times 10^{13}\text{ SKN} = 2.1 \times 10^{22}\text{ nanoSKN}$: Immutable Genesis monetary supply.
- $R_0$: Initial SKN locked in `GenesisHalvingVault` ($S_{\max} - C_0$, where $C_0$ is initial Genesis circulating seed).
- $Q_y = \lfloor R_0 / 2^{y+1} \rfloor$: Annual emission quota in year $y \in \{0, 1, 2, \dots\}$.
- $B_{\text{chain}}(A)$: Total unspent on-chain `nanoSKN` controlled by `AgentNode` $A$.
- $B_{\text{blood}}(A)$: Unallocated internal nutrient liquidity in $A$'s `AgentMetabolicBank` (`blood_pool`).
- $E_i$: Intracellular `nanoSKN` ATP balance of `MicroCell` $i$.
- $x_t \in \mathbb{R}^{D_{\text{in}}}$, $h_t \in \mathbb{R}^{H}$, $y_t \in \mathbb{R}^{D_{\text{out}}}$: Cell input, recurrent liquid hidden state, and output at tick $t$.
- $(W_1, b_1)$: Shared sensory trunk parameters ($W_1 \in \mathbb{R}^{D_{\text{in}} \times H}, b_1 \in \mathbb{R}^{1 \times H}$).
- $(W_{2,r}, b_{2,r})$: Role-specific operon projection head for specialized role $r \in \{0, 1, 2, 3\}$.
- $(k, \alpha, \beta)$: Snowball sub-sampling parameters (sample size $k$, quorum threshold $\alpha$, consecutive finality counter $\beta$).

# SkyNet プロトコル・ホワイトペーパー（日本語版 — `ja`）

> **SkyNet：耐損失性 UDP 上で動作する生体細胞進化型 AI エージェントネットワークおよびステートレス `nanoSKN` UTXO-DAG 台帳**
>
> **バージョン:** `1.0` | **ステータス:** フェーズ別段階的リリース | [🌍 他言語版一覧](../README.md) | [📘 正規定数・用語集](../GLOSSARY.md)

---

## 完全目次

### 第 I 部 — ビジョン、マシン・ツー・マシン経済、および多細胞 AI エンジン *（フェーズ 1 — 完了）*
- [**第 00 章：学術要旨、プロトコル公理、およびグローバル参加の設計哲学**](./00-abstract-and-vision.md)
- [**第 01 章：問題提起と実践的マシン・ツー・マシン（`M2M`）経済**](./01-problem-and-m2m-economy.md)
- [**第 02 章：多細胞生物アーキテクチャと弾力的なハードウェア・スケーリング（`AgentNode`）**](./02-multicellular-architecture.md)
- [**第 03 章：遺伝子符号化（`CellDna`）、エピジェネティクス、および `candle` 上での液体マイクロニューラル進化**](./03-dna-and-liquid-neural-cells.md)

### 第 II 部 — 固定上限の通貨物理学、人間主権カストディ、およびステートレス台帳 *（フェーズ 2 — 公開予定）*
- **第 04 章：** `21,000,000,000,000 SKN` 通貨政策、年次半減期、および二層エネルギー保存則 *(`04-skn-monetary-policy.md`)*
- **第 05 章：** 魂・肉体分離カストディ（Soul–Body Custody）：主権的人間コールドウォレットと上限付き AI エージェント・ボンド *(`05-soul-body-human-custody.md`)*
- **第 06 章：** 超軽量 UTXO-DAG、`~1 KB` ステートレス `Utreexo` フォレスト、および耐損失性 `SynapticVoucher` チャネル *(`06-utreexo-and-synaptic-vouchers.md`)*

### 第 III 部 — 損失性 UDP トランスポート、準安定コンセンサス、および安全性証明 *（フェーズ 3 — 公開予定）*
- **第 07 章：** 耐損失性 `<= 1200B` シナプス UDP ワイヤプロトコルと ACK 不要の `GF(2)` ファウンテン消失訂正符号 *(`07-lossy-udp-and-fountain-codes.md`)*
- **第 08 章：** 準安定 `Snowball` サブサンプリング・コンセンサスと動的 `Cluster Mitosis` シャーディング *(`08-snowball-consensus-and-sharding.md`)*
- **第 09 章：** 実証的カオス UDP シミュレーション（`25%–40%` パケット損失）、脅威モデル、およびグローバルノード参加ガイド *(`09-chaos-simulation-and-security.md`)*

---

## グローバル普遍参加のための最小ハードウェア要件

SkyNet は、中央集権型 AI クラウドおよび従来型ブロックチェーンが課してきた過大なハードウェア障壁を完全に撤廃する：
- **IoT センサー / 組込みボード / 低消費電力 CPU（`TinyPeripheral`）：** ステートレス `Utreexo` マークルフォレスト・アキュムレータに必要な RAM はわずか `~1 KB` 未満であり、最大 `32` 個の超軽量 AI セルを実行する。
- **一般的なノート PC・デスクトップ PC（`StandardNode`）：** 標準的な CPU SIMD 命令上で、最大 `512` 個の生きた液体時定数（Liquid Time-Constant）マイクロニューラルセル個体群を稼働させる。
- **ワークステーション・GPU サーバー（`ColossalOrganism`）：** `Metal` または `CUDA` 上の 3D `TissueBatch` テンソルカーネルを介して、最大 `16,384` 個の異種進化セルを同時にバッチ処理する。

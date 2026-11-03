# SkyNet 프로토콜 백서 (한국어판 — `ko`)

> **SkyNet: 손실 내성 UDP 기반의 생체 세포 진화형 AI 에이전트 네트워크 및 무상태 `nanoSKN` UTXO-DAG 원장**
>
> **버전:** `1.0` | **상태:** 단계별 순차 공개(Incremental Phased Release) | [🌍 다른 언어 판본](../README.md) | [📘 표준 상수 및 용어집](../GLOSSARY.md)

---

## 전체 목차

### 제1부 — 비전, 기계 간(`M2M`) 경제 및 다세포 AI 엔진 *(1단계 — 완료)*
- [**제00장: 학술 초록, 프로토콜 공리 및 범지구적 참여 철학**](./00-abstract-and-vision.md)
- [**제01장: 문제 정의 및 실용적 기계 간(`M2M`) 경제**](./01-problem-and-m2m-economy.md)
- [**제02장: 다세포 유기체 아키텍처 및 탄력적 하드웨어 스케일링 (`AgentNode`)**](./02-multicellular-architecture.md)
- [**제03장: 유전 부호화(`CellDna`), 후성유전학 및 `candle` 기반 액체 미세 신경망 진화**](./03-dna-and-liquid-neural-cells.md)

### 제2부 — 고정 상한 통화 물리학, 인간 주권 수탁 및 무상태 원장 *(2단계 — 예정)*
- **제04장:** `21,000,000,000,000 SKN` 통화 정책, 연간 반감기 및 2계층 에너지 보존 *(`04-skn-monetary-policy.md`)*
- **제05장:** 영혼–육체(`Soul–Body`) 수탁: 인간 주권형 콜드 월렛 및 유계 AI 에이전트 보증금 *(`05-soul-body-human-custody.md`)*
- **제06장:** 초경량 UTXO-DAG, `~1 KB` 무상태 `Utreexo` 포레스트 및 손실 내성 `SynapticVoucher` 채널 *(`06-utreexo-and-synaptic-vouchers.md`)*

### 제3부 — 손실성 UDP 전송, 준안정 합의 및 보안 증명 *(3단계 — 예정)*
- **제07장:** 손실 내성 `<= 1200B` 시냅스 UDP 와이어 프로토콜 및 무응답(ACK-less) `GF(2)` 파운틴 소거 부호 *(`07-lossy-udp-and-fountain-codes.md`)*
- **제08장:** 준안정 `Snowball` 서브샘플링 합의 및 동적 `Cluster Mitosis` 샤딩 *(`08-snowball-consensus-and-sharding.md`)*
- **제09장:** 실증적 카오스 UDP 시뮬레이션(`25%–40%` 패킷 손실), 위협 모델 및 글로벌 노드 온보딩 *(`09-chaos-simulation-and-security.md`)*

---

## 보편적 글로벌 참여를 위한 최소 하드웨어 요구 사항

SkyNet은 중앙집중식 AI 클라우드와 기존 블록체인이 부과하던 과도한 하드웨어 진입 장벽을 완전히 제거한다:
- **IoT 센서 / 임베디드 보드 / 저사양 CPU (`TinyPeripheral`):** 무상태 `Utreexo` 머클 포레스트(Merkle forest) 누산기에 필요한 RAM이 `~1 KB` 미만이며, 최대 `32`개의 초경량 AI 세포를 구동한다.
- **일반 노트북 및 데스크톱 PC (`StandardNode`):** 표준 CPU SIMD 환경에서 최대 `512`개의 액체 시정수(Liquid Time-Constant) 미세 신경 세포로 구성된 살아 있는 개체군을 운용한다.
- **워크스테이션 및 GPU 서버 (`ColossalOrganism`):** `Metal` 또는 `CUDA` 상의 3차원 `TissueBatch` 텐서 커널을 통해 최대 `16,384`개의 이질적 진화 세포를 동시에 배치(batch) 연산한다.

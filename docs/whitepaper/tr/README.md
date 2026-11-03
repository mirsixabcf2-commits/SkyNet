# SkyNet Protokolü Teknik Raporu (Türkçe Sürüm — `tr`)

> **SkyNet: Paket Kaybına Dayanıklı UDP Üzerinde Biyo-Hücresel Evrimsel Yapay Zekâ Ajan Ağı ve Durumsuz `nanoSKN` UTXO-DAG Dağıtık Defteri**
>
> **Sürüm:** `1.0` | **Durum:** Aşamalı Artımlı Yayın | [🌍 Diğer Diller](../README.md) | [📘 Kanonik Sabitler ve Sözlük](../GLOSSARY.md)

---

## Tam İçindekiler Tablosu

### Kısım I — Vizyon, Makineden Makineye Ekonomi ve Çok Hücreli Yapay Zekâ Motoru *(Faz 1 — Tamamlandı)*
- [**Bölüm 00: Akademik Özet, Protokol Aksiyomları ve Küresel Katılım Felsefesi**](./00-abstract-and-vision.md)
- [**Bölüm 01: Problem Tanımı ve Pratik Makineden Makineye (`M2M`) Ekonomi**](./01-problem-and-m2m-economy.md)
- [**Bölüm 02: Çok Hücreli Organizma Mimarisi ve Esnek Donanım Ölçeklendirmesi (`AgentNode`)**](./02-multicellular-architecture.md)
- [**Bölüm 03: Genetik Kodlama (`CellDna`), Epigenetik ve `candle` Üzerinde Sıvı Mikro-Nöral Evrim**](./03-dna-and-liquid-neural-cells.md)

### Kısım II — Sabit Arzlı Parasal Fizik, İnsan Egemenliği ve Durumsuz Dağıtık Defter *(Faz 2 — Planlandı)*
- **Bölüm 04:** `21,000,000,000,000 SKN` Para Politikası, Yıllık Yarılanma (Halving) ve İki Katmanlı Enerji Korunumu *(`04-skn-monetary-policy.md`)*
- **Bölüm 05:** Ruh–Beden Saklama Sözleşmesi (`Soul–Body Custody`): Egemen İnsan Soğuk Cüzdanları ve Sınırlandırılmış Yapay Zekâ Ajan Teminatları *(`05-soul-body-human-custody.md`)*
- **Bölüm 06:** Ultra Hafif UTXO-DAG, `~1 KB` Durumsuz `Utreexo` Ormanı ve Paket Kaybına Dayanıklı `SynapticVoucher` Kanalları *(`06-utreexo-and-synaptic-vouchers.md`)*

### Kısım III — Kayıplı UDP İletimi, Yarı Kararlı Mutabakat ve Güvenlik Kanıtları *(Faz 3 — Planlandı)*
- **Bölüm 07:** Paket Kaybına Dayanıklı `<= 1200B` Sinaptik UDP Hat Protokolü ve Onaysız (ACK-less) `GF(2)` Çeşme Silme Kodları *(`07-lossy-udp-and-fountain-codes.md`)*
- **Bölüm 08:** Yarı Kararlı `Snowball` Alt-Örnekleme Mutabakatı ve Dinamik `Cluster Mitosis` Parçalanması (Sharding) *(`08-snowball-consensus-and-sharding.md`)*
- **Bölüm 09:** Ampirik Kaos UDP Simülasyonu (`25%–40%` Paket Kaybı), Tehdit Modeli ve Küresel Düğüm Katılımı *(`09-chaos-simulation-and-security.md`)*

---

## Evrensel Küresel Katılım İçin Asgari Donanım Gereksinimleri

SkyNet, hem merkezi yapay zekâ bulutlarının hem de geleneksel blokzincirlerin dayattığı fahiş donanım engellerini ortadan kaldırır:
- **IoT Sensörleri / Gömülü Kartlar / Düşük Güçlü İşlemciler (`TinyPeripheral`):** Durumsuz `Utreexo` Merkle ormanı akümülatörü için `~1 KB` altında RAM gerektirir ve `32` adede kadar ultra hafif yapay zekâ hücresi çalıştırır.
- **Günlük Dizüstü ve Masaüstü Bilgisayarlar (`StandardNode`):** Standart CPU SIMD komutlarıyla `512` adede kadar Sıvı Zaman Sabitli (Liquid Time-Constant) mikro-sinir hücresinden oluşan canlı bir popülasyonu işletir.
- **İş İstasyonları ve GPU Sunucuları (`ColossalOrganism`):** `Metal` veya `CUDA` üzerindeki 3B `TissueBatch` tensör çekirdekleri aracılığıyla aynı anda `16,384` adede kadar heterojen evrimleşen hücreyi toplu (batch) olarak işler.

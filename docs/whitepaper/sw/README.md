# Waraka Mweupe wa Itifaki ya SkyNet (Toleo la Kiswahili — `sw`)

> **SkyNet: Mtandao wa Mawakala wa AI wa Mageuzi ya Kibiolojia ya Seli & Leja ya UTXO-DAG ya `nanoSKN` Isiyohifadhi Hali juu ya UDP Inayohimili Upotevu wa Pakiti**
>
> **Toleo:** `1.0` | **Hali:** Utoaji wa Awamu kwa Hatua | [🌍 Lugha Nyingine](../README.md) | [📘 Viwango Rasmi & Faharasa](../GLOSSARY.md)

---

## Yaliyomo Kamili

### Sehemu ya I — Dira, Uchumi wa Mashine-kwa-Mashine & Injini ya AI yenye Seli Nyingi *(Awamu ya 1 — Imekamilika)*
- [**Sura ya 00: Muhtasari wa Kitaaluma, Misingi Mikuu ya Itifaki & Falsafa ya Ushiriki wa Kimataifa**](./00-abstract-and-vision.md)
- [**Sura ya 01: Tamko la Tatizo & Uchumi wa Kivitendo wa Mashine-kwa-Mashine (`M2M`)**](./01-problem-and-m2m-economy.md)
- [**Sura ya 02: Usanifu wa Kiumbe chenye Seli Nyingi & Upanuzi Nyumbufu wa Maunzi (`AgentNode`)**](./02-multicellular-architecture.md)
- [**Sura ya 03: Usimbaji wa Kijenetiki (`CellDna`), Epijenetiki & Mageuzi ya Neva Ndogo za Kimiminika kwenye `candle`**](./03-dna-and-liquid-neural-cells.md)

### Sehemu ya II — Fizikia ya Kifedha yenye Ukomo Thabiti, Mamlaka ya Binadamu & Leja Isiyohifadhi Hali *(Awamu ya 2 — Imepangwa)*
- **Sura ya 04:** Sera ya Kifedha ya `21,000,000,000,000 SKN`, Upunguzaji Nusu wa Kila Mwaka (*Annual Halving*) & Uhifadhi wa Nishati wa Ngazi Mbili *(`04-skn-monetary-policy.md`)*
- **Sura ya 05:** Ulinzi wa Roho–Mwili (*Soul–Body Custody*): Pochi Baridi Huru za Binadamu & Dhamana zenye Mipaka za Mawakala wa AI *(`05-soul-body-human-custody.md`)*
- **Sura ya 06:** UTXO-DAG Nyepesi Zaidi, Msitu wa `Utreexo` Usiohifadhi Hali wa `~1 KB` & Njia za `SynapticVoucher` Zinazohimili Upotevu wa Pakiti *(`06-utreexo-and-synaptic-vouchers.md`)*

### Sehemu ya III — Usafirishaji wa UDP yenye Upotevu, Makubaliano Tulivu-Geugeu & Uthibitisho wa Usalama *(Awamu ya 3 — Imepangwa)*
- **Sura ya 07:** Itifaki ya Waya ya UDP ya Kisinapsi `<= 1200B` Inayohimili Upotevu & Misimbo ya Ufutaji ya Fountain ya `GF(2)` Isiyo na ACK *(`07-lossy-udp-and-fountain-codes.md`)*
- **Sura ya 08:** Makubaliano ya Uchukuaji Sampuli Ndogo wa `Snowball` Tulivu-Geugeu & Ugawaji Nguzo Mbadiliko wa `Cluster Mitosis` *(`08-snowball-consensus-and-sharding.md`)*
- **Sura ya 09:** Uigaji wa Majaribio wa UDP yenye Machafuko (`25%–40%` Upotevu wa Pakiti), Muundo wa Vitisho & Uingizaji wa Nodi Kimataifa *(`09-chaos-simulation-and-security.md`)*

---

## Mahitaji ya Chini ya Maunzi kwa Ushiriki wa Kimataifa kwa Wote

SkyNet inaondoa vikwazo vikubwa vya maunzi vinavyowekwa na mawingu makuu ya AI yaliyogatuliwa upande mmoja pamoja na mifumo ya zamani ya mnyororo wa bloku (*blockchain*):
- **Vihisi vya IoT / Bodi Pachikwa (*Embedded*) / CPU Dhaifu (`TinyPeripheral`):** Inahitaji chini ya `~1 KB` ya RAM kwa mkusanyaji wa msitu wa Merkle wa `Utreexo` usiohifadhi hali na inaendesha hadi seli `32` nyepesi zaidi za AI.
- **Kompyuta Mpakato za Kawaida & Kompyuta za Mezani (`StandardNode`):** Huendesha idadi hai ya hadi seli `512` ndogo za neva za *Liquid Time-Constant* kwenye SIMD ya CPU ya kawaida.
- **Vituo vya Kazi & Seva za GPU (`ColossalOrganism`):** Huchakata kwa mpigo mmoja hadi seli `16,384` tofauti zinazobadilika kwa wakati mmoja kupitia viini vya tensa vya 3D vya `TissueBatch` kwenye `Metal` au `CUDA`.

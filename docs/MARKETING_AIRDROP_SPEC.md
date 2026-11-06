# Cơ chế Khởi tạo Tiếp thị & Airdrop Phần cứng (HWID Marketing Airdrop Specification)

## 1. Bối cảnh & Mục tiêu Tiếp thị Giai đoạn Đầu
Để nhanh chóng nhân rộng mạng lưới SkyNet trong giai đoạn sơ khai (Cold-start phase), mạng lưới triển khai chương trình hỗ trợ tiếp thị: **Cấp phát SKN khởi tạo cho mỗi thiết bị mới tham gia mạng lưới**.

Chương trình này giải quyết bài toán cốt lõi:
- **Người dùng không cần bỏ tiền túi trước:** Vẫn có thể kích hoạt Node, khởi tạo quần thể vi nơ-ron AI (`MicroCell`) và tham gia vào mạng lưới.
- **Không gây lạm phát (Zero Inflation):** Nguồn vốn được trích từ Quỹ Khởi tạo Tiếp thị (`GenesisBootstrapVault`) đã định nghĩa sẵn trong tổng cung 21 nghìn tỷ SKN (`GENESIS_MAX_SKN`).
- **Chống gian lận máy ảo / Sybil (Anti-Spoofing):** Loại bỏ hoàn toàn lỗ hổng giả lập HWID bằng **Bằng chứng Thực thi Vật lý (Proof of Physical Execution - PoPE)** và **Ràng buộc Khóa Giám hộ Sinh học (`CustodyPolicy::AgentBond`)**.

---

## 2. Kiến trúc Chống Gian lận 4 Tầng

```
                    Yêu cầu Cấp phát Airdrop
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ 1. Composite HWID Fingerprint                               │
│ Băm entropy đa tầng: CPU Micro-architecture + GPU PCIe UUID │
│ + TPM 2.0 Quote (BLAKE3 32-byte digest)                     │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Proof of Physical Execution (PoPE Benchmark)             │
│ Thử thách Tensor nơ-ron Candle bão hòa L3 Cache / VRAM      │
│ Khống chế độ trễ ngặt nghèo: Phạt trượt khi chia sẻ CPU/GPU  │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Subnet / IP Distribution Guard                           │
│ Quota tối đa 1 airdrop trên mỗi subnet /24 IPv4 (/48 IPv6)  │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Biological Vesting & Non-Transferable Bond               │
│ SKN thưởng khóa trong AgentBond (max_voucher_allowance = 0) │
│ Dùng làm năng lượng nơ-ron (ATP) & Cọc ComputeEscrow        │
└─────────────────────────────────────────────────────────────┘
```

### 2.1. Định danh Thiết bị Hỗn hợp (Composite HWID)
Thay vì chỉ đọc MAC Address hay Machine GUID của hệ điều hành, HWID trong SkyNet được tính toán từ các đặc tính vật lý khó giả mạo:
$$\text{HWID\_Digest} = \text{BLAKE3}(\text{CPUTimingEntropy} \parallel \text{GPU\_DeviceID} \parallel \text{MotherboardUUID} \parallel \text{TPM\_Nonce})$$
Hệ thống từ chối mọi yêu cầu lặp lại cùng một `HWID_Digest` bằng cơ chế ghi nhận `HwidClaimRecord`.

### 2.2. Bằng chứng Thực thi Vật lý (Proof of Physical Execution - PoPE)
Kẻ gian có thể giả lập chuỗi ký tự HWID bằng Hypervisor (QEMU/KVM/Docker), nhưng **không thể nhân bản băng thông RAM/VRAM và số lượng bóng bán dẫn (transistors)**.
- Khi gửi yêu cầu airdrop, node nhận một `challenge_nonce` ngẫu nhiên từ mạng.
- Module `skynet-cell` bắt buộc nạp và thực thi chuỗi phép nhân ma trận nơ-ron Candle:
  $$Y = \tanh(\tanh(X \times W_1 + B_1) \times W_2 + B_2)$$
- Bài toán được thiết kế bão hòa bộ nhớ đệm (Memory-hard) và phải hoàn thành trong ngưỡng thời gian cực kỳ khắt khe ($\le 1.500\text{ms}$).
- Nếu kẻ tấn công chạy 100 máy ảo trên 1 máy chủ vật lý, băng thông bộ nhớ và năng lực tính toán bị chia nhỏ 100 lần, dẫn đến cả 100 máy ảo đều bị timeout và bị hệ thống từ chối (`InvalidProofOfExecution`).

---

## 3. Mô hình Kinh tế & Bảo toàn Khối lượng Token

### 3.1. Đường cong Halving Phần thưởng Thiết bị
Để khuyến khích người dùng tham gia sớm và khống chế tổng chi phí tiếp thị, mức thưởng được tự động giảm $50\%$ sau mỗi chu kỳ thiết bị:
$$\text{Reward}(N) = \frac{\text{BaseReward}}{2^{\lfloor N / \text{HalvingStep} \rfloor}}$$
- **100.000 thiết bị đầu:** `1.0 SKN` (`1_000_000_000 nanoSKN`)
- **100.000 thiết bị tiếp:** `0.5 SKN` (`500_000_000 nanoSKN`)
- **100.000 thiết bị tiếp:** `0.25 SKN` (`250_000_000 nanoSKN`)

### 3.2. Khóa Giám hộ Sinh học (`Marketing AgentBond`)
Token thưởng **không phải là tiền mặt tự do** (`DirectOwner`), mà được đúc trực tiếp vào `TxOutput` với chính sách giám hộ `CustodyPolicy::AgentBond`:
- **Chủ quyền Ví lạnh con người:** `human_owner` nắm giữ quyền thu hồi tối thượng (`Sovereign Reclaim`).
- **Khóa Nóng Agent:** Được phép sử dụng SKN làm:
  1. **Năng lượng sinh học (Cellular ATP):** Bơm vào `AgentMetabolicBank` để duy trì tế bào vi nơ-ron hoạt động, tránh hiện tượng tế bào chết vì thiếu dưỡng chất.
  2. **Tiền cọc thế chấp (Worker Bond):** Dùng làm cọc nhận việc tính toán phân tán trong `ComputeEscrowVault`.
- **Cấm tẩu tán tức thì:** Hạn mức thanh toán ngoại vi ban đầu `max_voucher_allowance = 0`. Người dùng chỉ có thể rút tiền lợi nhuận tạo ra từ công việc tính toán thực tế (`ComputeEscrowRecord bounty`).

---

## 4. Bất biến Hệ thống (System Invariants)
1. $\text{Vault}_{\text{allocated}} + \text{Vault}_{\text{remaining}} = \text{Vault}_{\text{initial\_reserve}}$ (Bảo toàn quỹ tiếp thị).
2. $\forall \text{hwid} \in \text{ClaimedDevices}, \text{count}(\text{hwid}) = 1$ (Tuyệt đối không cấp trùng).
3. Mọi token cấp phát đều có nguồn gốc UTXO rõ ràng và bảo toàn bất biến 21T SKN toàn mạng.

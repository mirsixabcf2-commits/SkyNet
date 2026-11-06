# SkyNet Constitution

## Core Principles

### I. Modular Architecture & Strict Physical Code Limits
- Mỗi file mã nguồn PHẢI tập trung vào một khái niệm/trách nhiệm duy nhất (Single Concept per File) với mục tiêu dưới **250 dòng** (không tính import/comment) và **giới hạn cứng 350 dòng**. Khi chỉnh sửa file vượt 250 dòng, Agent PHẢI chủ động tách các logic độc lập sang submodule trước hoặc trong khi thêm tính năng.
- Mỗi hàm/phương thức PHẢI có độ dài tối đa **40 dòng**, tối đa **3–4 tham số** (nếu nhiều hơn PHẢI gom vào Struct/Options DTO có định kiểu rõ ràng), và độ sâu lồng nhau tối đa **2–3 cấp** (bắt buộc áp dụng Guard Clauses / Early Return, cấm dùng `else` dư thừa sau `return`).
- Hệ thống PHẢI tuân thủ phân tách tầng kiến trúc (Giao tiếp/Wire/UI, Nghiệp vụ/Domain, Dữ liệu/Transport, Định nghĩa Types/Errors) và tổ chức theo phân hệ nghiệp vụ (Feature-based). Nghiêm cấm phụ thuộc vòng (Circular Dependencies) và nghiêm cấm tạo file `utils.rs`/`helpers.rs` chung chung (mọi hàm tiện ích phải đặt tên theo miền chức năng cụ thể và là Pure Functions).

### II. Type Safety, Explicit Error Handling & Security
- Mọi tác vụ I/O, truyền thông UDP, giải mã nhị phân, tính toán sổ cái `NanoSkn(u128)` và xử lý bất đồng bộ PHẢI trả về `Result<T, E>` tường minh (sử dụng `thiserror`), kèm ngữ cảnh lỗi rõ ràng; nghiêm cấm nuốt lỗi (silent fail) hoặc dùng `.unwrap()`/`.expect()` trên dữ liệu runtime không đáng tin cậy.
- Toàn bộ dữ liệu đầu vào (gói tin UDP `<= 1200B`, chữ ký `Ed25519`, giao dịch UTXO-DAG, hạn mức `AgentBond`) PHẢI được kiểm증 (validate) nghiêm ngặt tại biên nhận trước khi đi vào tầng nghiệp vụ.
- Nghiêm cấm hardcode khóa bí mật, mật khẩu hoặc "Magic Numbers" rải rác trong logic; mọi ngưỡng giới hạn, hằng số giao thức và tham số hệ thống PHẢI được tập trung tại module `constants` hoặc `config`.

### III. Mandatory 4-Step Engineering Workflow & Verification
- Mọi thay đổi mã nguồn PHẢI tuân thủ quy trình 4 bước:
  1. **Khảo sát (Explore):** Đọc kỹ các file, kiểu dữ liệu và cấu trúc liên quan trước khi sửa; không đoán mò tên hàm hay cấu trúc dữ liệu.
  2. **Lập kế hoạch (Plan):** Tóm tắt giải pháp (3–5 gạch đầu dòng) và danh sách file bị ảnh hưởng.
  3. **Thực thi (Implement):** Viết code hoàn chỉnh chuẩn Production; tuyệt đối KHÔNG dùng comment rút gọn (`// ... code cũ giữ nguyên`, `// TODO: xử lý sau`).
  4. **Kiểm chứng (Verify):** Chủ động chạy `cargo check --workspace`, `cargo clippy --workspace --all-targets`, và `cargo test --workspace`, tự động sửa lỗi cho đến khi vượt qua 100%.

### IV. Blast Radius Discipline & Direct Communication
- Chỉ sửa đổi đúng phạm vi mã nguồn giải quyết yêu cầu (Blast Radius Discipline); không tự ý format lại các vùng code không liên quan hoặc thêm crate phụ thuộc mới khi chưa có sự đồng ý của người dùng.
- Giao tiếp kỹ thuật PHẢI trực diện, ngắn gọn, đi thẳng vào trọng tâm và bỏ qua các câu mở đầu xã giao.

## Technical & Security Constraints

- **Tech Stack:** Rust Edition 2021 (`rust-version = "1.83.0"`), Cargo Workspace gồm 7 crates (`skynet-dna`, `skynet-cell`, `skynet-ledger`, `skynet-wire`, `skynet-agent`, `skynet-sim`, `skynet-dashboard`).
- **Core Dependencies:** `tokio`, `candle-core`, `half`, `blake3`, `ed25519-dalek`, `rand`, `rand_chacha`, `serde`, `bincode`, `serde_json`, `thiserror`.
- **Protocol Invariants:** MTU gói tin UDP không vượt quá `1200 Bytes`; tiền tệ nguyên tử `NanoSkn(u128)` chống tràn số với tổng cung bất biến `21,000,000,000,000 SKN`; đảm bảo bất biến bảo mật lưu ký Soul–Body (`HumanPrincipalDiversionBlocked`, `Sovereign Reclaim`).

## Development Workflow & Quality Gates

- Mọi bản kế hoạch kỹ thuật (`plan.md`), danh sách tác vụ (`tasks.md`) và PR/commit đều PHẢI vượt qua kiểm tra đối chiếu với các giới hạn vật lý (File <= 350 dòng, Hàm <= 40 dòng, Tham số <= 4, Nesting <= 3) và kiểm chứng tự động (`cargo test --workspace`) trước khi hoàn tất.

## Governance

This Constitution supersedes ad-hoc implementation choices. Architectural plans (`plan.md`) and task lists (`tasks.md`) MUST pass a Constitution Check before implementation begins. Amendments require updating this file and incrementing its semantic version.

**Version**: 1.0.0 | **Ratified**: 2026-11-04 | **Last Amended**: 2026-11-04

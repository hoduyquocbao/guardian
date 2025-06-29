# Guardian Store

## Triết lý Kiến trúc

- **Mọi định danh (biến, hàm, type, module, const) phải là MỘT TỪ TIẾNG ANH**
- Cấu trúc module cung cấp ngữ cảnh, loại bỏ sự mơ hồ mà không cần tên dài dòng
- Refactor định kỳ để loại bỏ mọi nợ định danh, đảm bảo sự thanh lịch và nhất quán

## Quy tắc Định danh

- Không sử dụng snake_case, PascalCase, hay SCREAMING_SNAKE_CASE cho bất kỳ định danh nào
- Ví dụ refactor:
  - `batch_save` → `batch`
  - `from_bytes` → `unpack`
  - `MAX_SEGMENT_SIZE` → `MAXSIZE`
  - `base_path` → `base`

## Roadmap mới

1. **Refactor toàn bộ định danh trong storage và guardian-macros** (ĐÃ HOÀN THÀNH)
2. **Triển khai guardian-macros MVP**: parser, generator, trybuild test UI
3. **Mở rộng tính năng macro**: hỗ trợ nhiều kiểu, endianness, version, nesting

## Kiểm thử & CI

- Sử dụng script `naming.sh` để kiểm tra định danh toàn bộ workspace
- Tích hợp kiểm tra naming vào CI để ngăn chặn nợ định danh mới
- Tất cả macro phải có test UI với trybuild (pass/fail)

## PKB & Tài liệu

- Mọi quyết định, chỉ thị, thay đổi lớn đều được ghi nhận trong:
  - `memories.csv` (tri thức, milestone)
  - `todo.csv` (nhiệm vụ, chỉ thị)
  - `decisions.csv` (quyết định kiến trúc)
  - `vocabulary.csv` (từ điển định danh)

## Ví dụ sử dụng macro
```rust
use guardian_macros::frame;

#[frame]
pub struct Packet {
    id: u32,
    data: rest,
}
```

## Đóng góp
- Mọi pull request phải tuân thủ triết lý một từ và có kiểm thử định danh tự động.

## License

MIT License - see LICENSE file for details. 
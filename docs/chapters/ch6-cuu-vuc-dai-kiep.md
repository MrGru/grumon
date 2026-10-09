# Chương 6 — Cửu Vực Đại Kiếp

> Trạng thái: **kịch bản cấp sự kiện**.

## 1. Tổng quan

| Mục | Nội dung |
|-----|----------|
| Bản đồ mới | Bản đồ chiến dịch Cửu Vực (chọn điểm nóng), Thanh Huyền Sơn (bị vây), Mật đạo hậu sơn, Thiên Trụ Sơn chân núi, Kho Mệnh Bộ, Đại điện Thiên Diễn (ngoại vi) |
| Cảnh giới | Kết Đan → **Nguyên Anh sơ kỳ** |
| Cơ chế mới | **Bản đồ chiến dịch**: mỗi lượt chiến dịch chọn 1 trong 3 điểm nóng; điểm không được chọn diễn biến theo danh vọng/đồng minh. Kẻ địch phản chế nâng cao (phá trận + huyễn thuật phối hợp) |
| Cảm xúc | Chiến tranh → sự thật động trời → đoàn tụ đắng cay |

## 2. Dòng sự kiện

1. **Hịch Thiên Diễn:** nhân vật chính bị tuyên là **Nghịch Thiên Tà Đồ**. Âu Dương Hạc đảo chính
   Thanh Huyền Môn, giam Chưởng môn Thẩm Huyền Thanh.
2. **Bản đồ chiến dịch** — các điểm nóng và hệ quả lựa chọn trước:
   - Thanh Huyền Sơn: mật đạo do **bé Đậu** dẫn (nếu `ch1.saved_dau`), hoặc phải phá cổng chính.
   - Bạch Thủy trấn: gửi viện binh (nếu `ch5.saved_bach_thuy`) hoặc trở thành điểm Thiên Diễn đóng quân.
   - Vân Châu: dân chúng nổi dậy giúp (nếu `ch4.exposed`) hoặc Chính Khí Minh chặn đường.
   - Vạn Hồn Điện: phe ly khai của Tạ Vô Ưu đứng lên (theo `trust.ta_vo_uu`).
3. **Âu Dương Liệt:** đứng về phía nhân vật chính nếu `ch2.spared_liet` **và** đã đưa hắn bằng chứng
   (Ch5) → đồng hành; nếu không → cường địch, chết trong tay chú mình (cảnh bi kịch).
4. **Cường địch Âu Dương Hạc** (giải cứu Chưởng môn). Trương Mặc sống hay chết tùy việc có kịp tới
   Khu tạp dịch không (lựa chọn điểm nóng).
5. **Kho Mệnh Bộ:** sự thật về linh căn — trang ghi tên **Tô Thanh Liên**: “Song linh căn Thủy–Mộc
   — *điều chỉnh: thượng đẳng* — tuyển làm Thánh nữ.” Trang ghi tên nhân vật chính: trống. Ngọc lão
   nhớ ra **Linh Mạch Ấn** (mảnh ký ức thứ 4), thú nhận một nửa: “Lão phu… biết người tạo ra nó.”
6. **Đoàn tụ Liên:** Thánh nữ Thanh Liên chặn đường. ⚖
   - Nếu `ch1.gave_hairpin` và `trust.to_thanh_lien ≥ 3`: đối thoại — nhắc lại lời hứa ở bến
     (`ch1.promise`), cô chạm vào trâm, ký ức vỡ òa → gia nhập ngay (`ch6.lien_awakened = 1`).
   - Ngược lại: cường địch (Thủy–Mộc, hồi phục, khiên), thắng rồi cô mới tỉnh, mang vết thương lòng
     (niềm tin khởi điểm thấp).
7. **Thiên kiếp là thu hoạch:** xác nhận qua lời khai của một trưởng lão Thiên Diễn hấp hối. Văn Trọng
   Khanh rút quân lên Thiên Trụ: “Ngươi sẽ hiểu khi nhìn thấy nó.”

## 3. Nhiệm vụ

| ID | Loại | Tên |
|----|------|-----|
| `ch6_hich` | Chính | Hịch truy nã |
| `ch6_chien_dich` | Chính | Cửu Vực dậy sóng (chuỗi 3 lượt chiến dịch) |
| `ch6_thanh_huyen` | Chính | Giải vây Thanh Huyền |
| `ch6_menh_bo` | Chính | Mệnh Bộ |
| `ch6_thanh_nu` | Chính | Thánh nữ Thanh Liên |
| `ch6_nguyen_anh` | Chính | Nguyên Anh |
| `ch6_ngon_den_2` | Phụ (Tạ Vô Ưu, phần 2) | Ngọn đèn cho em — Vạn Hồn Điện |
| `ch6_liet` | Phụ (Âu Dương Liệt) | Họ Âu Dương |
| `ch6_kiem_2` | Phụ (Diệp Hàn Sương, phần 3) | Kiếm chỉ Văn Trọng Khanh |
| `ch6_thanh_khe` | Phụ | Những người còn lại của Thanh Khê |

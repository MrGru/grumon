# Chương 2 — Ngoại Môn Phong Vân

> Trạng thái: **kịch bản cấp sự kiện** (đủ để triển khai; lời thoại sản xuất sẽ viết khi bắt đầu Cột mốc 4).

## 1. Tổng quan

| Mục | Nội dung |
|-----|----------|
| Bản đồ mới | Chân núi Thanh Huyền (sơn môn), Quảng trường ngoại môn, Khu tạp dịch, Vườn thuốc, Tàng Kinh Các (tầng 1), Lôi đài ngoại môn, Hậu sơn (đêm) |
| Cảnh giới | Luyện Khí sơ kỳ → trung/hậu kỳ |
| Cơ chế mới | Luyện đan cơ bản, điểm cống hiến, Tụ khí nâng cao (giữ bậc qua hai lượt có Hộ tâm), Hộ Tâm Trận cơ bản (khi có đồng môn), lôi đài 1-đấu-1 |
| Cảm xúc | Lạc lõng → ấm áp tình thầy trò → nghi ngờ người trên |

## 2. Dòng sự kiện

1. **Sơn môn** (`ch2_son_mon`): hàng dài thiếu niên chờ đo linh căn. Âu Dương Liệt xuất hiện cùng
   tùy tùng, chế nhạo đám phàm nhân. 🔍 Trắc Linh Thạch **run lên** khi nhân vật chính chạm — nhưng
   không sáng. Chấp sự **Trương Mặc** do dự, rồi ghi tên làm **tạp dịch đệ tử** “vì thằng/con bé
   này nhìn đá mà không run”.
2. **Khu tạp dịch:** sư huynh **Lưu Thành** dẫn đường, chia chăn, kể chuyện tông môn. Bạn cùng phòng
   **Tiểu Thạch** (NPC phụ, kẻ nói nhiều).
3. **Việc vặt có ý nghĩa** (mỗi việc dạy một cơ chế):
   - Gánh nước suối Băng Tuyền (giờ cao điểm: tránh đám Liệt — đi đường vòng có bí mật nhỏ).
   - **Vườn thuốc Đỗ lão quái** (`ch2_vuon_thuoc`): hướng dẫn **Luyện đan** — Hồi Khí Đan, Dưỡng Huyết Đan.
   - Dọn **Tàng Kinh Các**: lão giữ sách **Mặc lão** cho đọc lén sổ nhiệm vụ cũ.
4. **Bắt nạt:** Liệt ép nhân vật chính đấu “luyện tập” (trận có thể thua — thua vẫn tiếp tục câu chuyện,
   Trương Mặc can thiệp; thắng thì Liệt thù dai hơn).
5. **Trương Mặc nhận dạy** (`ch2_truong_mac`): bài học “Tụ khí là lòng kiên nhẫn” — trận luyện tập
   với **Mộc Nhân Trận** (hình nhân gỗ đánh vào ai đang Tụ khí; dạy dùng Thủ thế để giữ bậc).
6. **Điều tra:** so sánh lệnh bài với sổ đệ tử — thuộc **Hồ Bằng**, đệ tử ngoại môn “chết khi làm
   nhiệm vụ” ba tháng trước, thuộc phe Đại trưởng lão Âu Dương Hạc. Sổ nhiệm vụ ghi: “Đông Lâm — Thanh
   Khê — truy Vô Ấn. Phối hợp ngoại viện.” Chữ ký bị cạo.
7. **Đêm hậu sơn:** Mặc lão bị ám sát. Đuổi theo **thích khách áo đen** (cường địch — kiểu *thích
   khách*, Thân pháp cao, đánh hàng sau). Hắn tự sát bằng độc trước khi khai. Bạch Vân Cơ xuất hiện,
   nói một câu bí ẩn: “Ngươi có đôi mắt giống một người ta từng biết… Vân Thường.” 🔍
8. **Ngoại Môn Tiểu Tỷ** (`ch2_tieu_ty`): ba vòng lôi đài (mỗi vòng đối thủ khác kiểu: hộ vệ khiên,
   kẻ hút linh, Âu Dương Liệt). Chung kết với Liệt: hắn dùng **Xích Viêm Châu** mượn của chú — bị
   Quá nhiệt. ⚖ **Tha thứ** (đỡ hắn dậy trước mặt mọi người, `ch2.spared_liet = 1`) hay **làm nhục**
   (`stat.tam_ma +1`, Liệt thù hận).
9. **Kết chương:** thăng **ngoại môn đệ tử**, được ghi danh vào đoàn thám hiểm **Huyền Cốc Bí Cảnh**.
   Âu Dương Hạc nhìn từ xa: “Đứa trẻ không linh căn ấy… điều tra nó.”

## 3. Nhiệm vụ

| ID | Loại | Tên | Mục tiêu chính |
|----|------|-----|----------------|
| `ch2_nhap_mon` | Chính | Cửa núi | Xếp hàng đo linh căn; đến khu tạp dịch |
| `ch2_tap_dich` | Chính | Phận tạp dịch | Hoàn thành 3 việc vặt (nước, vườn thuốc, Tàng Kinh Các) |
| `ch2_tu_khi` | Chính | Lòng kiên nhẫn | Học Tụ khí nâng cao với Trương Mặc |
| `ch2_lenh_bai` | Chính | Chủ nhân lệnh bài | Tra sổ đệ tử; tìm sổ nhiệm vụ |
| `ch2_hau_son` | Chính | Đêm hậu sơn | Đuổi theo thích khách |
| `ch2_tieu_ty` | Chính | Ngoại Môn Tiểu Tỷ | Thắng 3 vòng lôi đài |
| `ch2_dan_dau_tien` | Phụ | Viên đan đầu tiên | Luyện thành Hồi Khí Đan cho Đỗ lão quái |
| `ch2_tieu_thach` | Phụ | Lá thư của Tiểu Thạch | Viết thư hộ người bạn mù chữ gửi mẹ (Tiểu Thạch sẽ là nhân chứng ở Ch6) |
| `ch2_lao_tien` | Phụ | Thương nhân dưới chân núi | Gặp lão Tiền; nghe chuyện con trai “phi thăng” 🔍 |
| `ch2_ho_bang` | Phụ | Mộ gió | Tìm gia đình Hồ Bằng ở thôn dưới núi — hắn chưa chết? (mở tuyến Ch4) |

## 4. Trận đấu và kẻ địch mới
- Mộc Nhân (luyện tập, kẻ ngắt quãng đơn giản), Đệ tử hộ vệ (khiên), Đệ tử hút linh, Âu Dương Liệt
  (Hỏa, bùng nổ, Quá nhiệt), Thích khách áo đen (thích khách, cường địch).

## 5. Phần thưởng chính
Truy Phong Ngoa (từ Mặc lão), công thức Hồi Khí Đan, Dưỡng Huyết Đan, chiêu **Thổ Lao Ấn** (khống
chế), điểm cống hiến.

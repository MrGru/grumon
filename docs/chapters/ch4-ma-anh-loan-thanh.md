# Chương 4 — Ma Ảnh Loạn Thành

> Trạng thái: **kịch bản cấp sự kiện**.

## 1. Tổng quan

| Mục | Nội dung |
|-----|----------|
| Bản đồ mới | Cổng Vân Châu, Phố chợ, Chợ đêm, Nha môn, Phủ Chính Khí Minh, Khu ổ chuột, Hội quán Vạn Bảo, Cống ngầm, Xưởng Hồn Đan |
| Cảnh giới | Trúc Cơ sơ → hậu kỳ |
| Cơ chế mới | **Sổ Manh Mối** (thu thập lời khai, đối chiếu mâu thuẫn), đấu giá, cộng hưởng pháp bảo, danh vọng thành phố |
| Cảm xúc | Hồi hộp điều tra → vùng xám đạo đức → báo thù |

## 2. Dòng sự kiện

1. **Đến Vân Châu** với Diệp Hàn Sương (và Đỗ lão quái được tông môn cử đi mua dược liệu — gia nhập).
2. **Vụ mất tích:** mười bảy phàm nhân có linh căn yếu biến mất trong một tháng. Nha môn đổ cho “yêu
   quái”. Gameplay điều tra: 8 nhân chứng, 3 lời khai mâu thuẫn; trình bày manh mối đúng để mở khóa
   hướng điều tra (sai không thất bại, chỉ tốn thời gian và danh vọng).
3. **Tạ Vô Ưu:** chạm trán trên mái nhà — tưởng là hung thủ. Trận chiến bị gián đoạn khi cả hai cùng
   bị phục kích. Hắn tiết lộ: Vạn Hồn Điện bắt người luyện **Hồn Đan**. “Ta không phải người tốt. Nhưng
   ta ghét kẻ ăn thịt trẻ con.” → gia nhập (`ch4_ta_vo_uu_join`).
4. **Đấu giá Vạn Bảo** (lão Tiền dẫn vào): Hồn Đan xuất hiện như “Thiên Kiếp Hộ Thân Đan”, người mua là
   trưởng lão các tông “chính đạo”. Có thể mua **Huyễn Ảnh Kính** hoặc **Trấn Hồn Linh** (chỉ đủ tiền
   cho một — lựa chọn tải trang bị).
5. **Âm mưu:** Minh chủ Chính Khí Minh Hà Chính Dương làm ngơ để đổi Hồn Đan. Ngọc lão nhận ra kỹ
   thuật tách hồn: “Bản sao vụng về… của thứ lão phu từng làm.” (mảnh ký ức thứ 2, giấu nửa sự thật).
6. ⚖ **Lựa chọn 1:** công khai tội ác trước dân chúng (`ch4.exposed = 1`: nổi loạn, Chính Khí Minh
   thù địch, nhưng Vân Châu ủng hộ ở Ch6) hay thỏa thuận ngầm với Hà Chính Dương (`ch4.exposed = 0`:
   thành phố yên ổn, Tiết Mị Nương trốn thoát).
7. **Xưởng Hồn Đan:** ⚖ **Lựa chọn 2:** cứu nạn nhân trong lồng (`ch4.saved_victims = 1`, trận bảo vệ
   mục tiêu) hay truy đuổi chủ mưu (bắt được Tiết Mị Nương, có thêm bằng chứng cho Ch6).
8. **Cường địch Tiết Mị Nương:** huyễn thuật — phân thân giả trên thứ tự hành động; Hư ảnh; Chậm.
9. **Đồ Cuồng:** trận báo thù Thanh Khê. Hắn niệm Huyết Sát Trảm như đêm ấy — lần này người chơi có
   đủ công cụ để ngắt. ⚖ Kết liễu hay để Tạ Vô Ưu mang về Vạn Hồn Điện xét xử (niềm tin Tạ Vô Ưu).

## 3. Nhiệm vụ

| ID | Loại | Tên |
|----|------|-----|
| `ch4_van_chau` | Chính | Thành của những cánh cửa đóng |
| `ch4_nhan_chung` | Chính | Lời khai |
| `ch4_mai_nha` | Chính | Bóng trên mái ngói |
| `ch4_dau_gia` | Chính | Phiên đấu giá Vạn Bảo |
| `ch4_hon_dan` | Chính | Hồn Đan |
| `ch4_cong_ngam` | Chính | Dưới lòng thành |
| `ch4_bao_thu` | Chính | Món nợ Thanh Khê |
| `ch4_ngon_den` | Phụ (Tạ Vô Ưu, phần 1) | Ngọn đèn cho em |
| `ch4_ho_bang` | Phụ | Hồ Bằng còn sống (nhân chứng vụ Thanh Khê) |
| `ch4_dan_su` | Phụ (Đỗ lão quái) | Đan phương bị đánh cắp |
| `ch4_cho_dem` | Phụ | Ba chuyện ở chợ đêm (3 nhiệm vụ nhỏ về số phận phàm nhân) |

## 4. Kẻ địch mới
Âm Hồn (hút linh), Huyết Y Vệ (phá trận), Thiết Giáp Vệ (hộ vệ), Ảnh Sát (thích khách), Tiết Mị
Nương (huyễn sư), Đồ Cuồng (chú sư).

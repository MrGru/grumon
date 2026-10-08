# Chương 1 — Phàm Trần Huyết Kiếp

> Trạng thái: **đã triển khai, chơi được từ đầu đến cuối** (xem `plan.md`).
> Lời thoại đầy đủ (bản gốc có thẩm quyền): `assets/locale/vi-VN/ch1.locale.ron`.
> Cấu trúc hội thoại, nhiệm vụ, trận đấu: `assets/data/ch1.data.ron`.

## 1. Tổng quan

| Mục | Nội dung |
|-----|----------|
| Thời lượng | 60–90 phút |
| Bản đồ | Thanh Khê thôn (`Village`), Rừng Tây Thanh Khê (`Forest`), Bến Thanh Khê (`Beach`), Hàn Phong Lĩnh (`Snowfield`) |
| Thời gian trong ngày | Sáng → Hoàng hôn → Đêm mưa (thảm sát) → Bình minh |
| Cơ chế mới | Di chuyển, trò chuyện, nhặt đồ, túi đồ, nhiệm vụ, thứ tự hành động, ĐHĐ, Linh lực, Thủ thế, Đẩy lùi (Sỏi Ném), trận pháp (khách), pháp bảo, Tụ khí, ngắt quãng |
| Cảnh giới | Phàm Nhân → **Luyện Khí sơ kỳ** |
| Cảm xúc | Ấm áp → chia ly → kinh hoàng → mất mát → quyết tâm |

## 2. Dòng sự kiện

### Cảnh 0 — Giấc mơ (`ch1_prologue`)
Màn hình đen. Giọng nói xa xăm: “Ngươi… nghe thấy ta không?” Ánh ngọc xanh nứt vỡ. Tỉnh dậy:
ông Mạc gõ cửa gọi dậy.

### Cảnh 1 — Buổi sáng ở Thanh Khê
- **Ông Mạc** (`ch1_ong_mac_morning`): càu nhàu vì cháu ngủ quên; giao nhiệm vụ **Hái Thanh Tâm Thảo**
  (`ch1_hai_thuoc`) — 3 nhánh trong Rừng Tây. Nhắc tránh rừng sau hoàng hôn vì có sói.
- Dân làng (tự do trò chuyện, mỗi người 2–3 câu khác nhau theo tiến trình):
  - **Liên** — vừa được người Thanh Huyền Môn đo linh căn, mùa xuân sẽ lên núi. Vui mà buồn.
    Hẹn chiều ra bến thả đèn trong **lễ Cúng Thần Sông**.
  - **Chú Sơn** thợ rèn — cha Liên. Ít nói, nhờ trông chừng con gái.
  - **Trưởng thôn Lý Đức** — kể về lễ hội, lo vì “mấy người lạ áo đen ghé quán thím Ba hôm qua”. 🔍
  - **Bé Đậu** — mê kiếm hiệp; diều mắc trên cây ở bến → nhiệm vụ phụ **Con diều của bé Đậu** (`ch1_dieu`).
  - **Thím Ba** — cần cá cho nồi cháo lễ → nhiệm vụ phụ **Nồi cháo cá của thím Ba** (`ch1_chao_ca`).

### Cảnh 2 — Rừng Tây (ban ngày)
- 3–4 bụi **Thanh Tâm Thảo** (vật thể nhặt được).
- **Anh Võ Tráng** tiều phu: cảnh báo sói; tặng **Sỏi Ném** nếu hỏi.
- Trận hướng dẫn **Dã Trư** (`ch1_da_tru`): dạy thứ tự hành động, ĐHĐ, Thủ thế trước đòn “Húc” (niệm 400 tích).
- 🔍 **Dấu giày vải đen** (vùng kích hoạt `ch1_dau_giay`): “Không phải giày người trong thôn.”

### Cảnh 3 — Nhiệm vụ phụ (tùy chọn, trước hoàng hôn)
| ID | Tên | Bước | Thưởng | Hệ quả |
|----|-----|------|--------|--------|
| `ch1_dieu` | Con diều của bé Đậu | Lấy diều mắc trên cây dừa ở bến → trả bé Đậu | Sỏi Ném ×4 | Bé Đậu tin tưởng: dễ thuyết phục trong đêm |
| `ch1_chao_ca` | Nồi cháo cá của thím Ba | Xin cá ở chú Năm (bến) → đem về cho thím Ba | Bánh Đậu Xanh ×3, Khói Mê Hương ×1 | Hồi phục trong các trận đêm |
| `ch1_tram_go` | Trâm gỗ hoa sen | Nhặt **gỗ đào** trôi dạt ở bến → mượn **dao khắc** của chú Sơn → khắc ở bàn gỗ nhà ông Mạc | Trâm gỗ hoa sen | Tặng Liên ở cảnh 5 (`ch1.gave_hairpin`), trả ở Ch6 |

### Cảnh 4 — Giao thuốc
Ông Mạc nhận thuốc (`ch1_ong_mac_herbs`). Cuộc trò chuyện về việc nhân vật chính không có linh căn:
“Có linh căn hay không, con vẫn là con.” Ông đưa túi thuốc và bảo ra bến dự lễ. Trời chuyển
**hoàng hôn**. Nhiệm vụ chính **Lễ Cúng Thần Sông** (`ch1_le_hoi`).

### Cảnh 5 — Hoàng hôn ở bến (`ch1_lien_dusk`)
- Liên hỏi đã xong việc chưa (cảnh báo: sau khi thả đèn không thể quay lại làm việc dang dở).
- Thả đèn hoa đăng, trò chuyện về tương lai. ⚖ **Lựa chọn:**
  - “Tớ sẽ lên núi thăm cậu.” → `ch1.promise = 1`
  - “Thật ra… tớ sợ bị bỏ lại.” → `ch1.promise = 2` (Liên an ủi, niềm tin +1 thêm)
- Nếu có trâm gỗ: tặng (`ch1.gave_hairpin = 1`).
- Mây đen, mưa. Ánh lửa đỏ phía làng. Tiếng chuông báo động. → **Đêm mưa** (`world.time = 3`).

### Cảnh 6 — Đêm thảm sát (Thanh Khê thôn)
- Vào làng: lửa, mưa, tiếng la hét (`ch1_raid_enter`). Nhiệm vụ **Huyết Kiếp** (`ch1_huyet_kiep`).
- **Thím Ba** chạy nạn: bé Đậu chạy lạc về phía cây lớn.
- **Bé Đậu** núp sau gốc cây (`ch1_dau_raid`). ⚖ Đưa bé đến chỗ thím Ba (`ch1.saved_dau = 1`, ông
  Mạc phải cầm chân địch lâu hơn: vào trận với 75 % khí huyết) hoặc bảo bé trốn kỹ (`ch1.saved_dau = 0`).
- **Hắc Y Tay Sai** chặn đường (`ch1_hac_y`): trận nhỏ, đánh bằng gậy.
- **Ông Mạc và Đồ Cuồng** trước sân nhà (`ch1_raid_ong_mac`). Ông Mạc lộ tu vi Kết Đan bị phong ấn,
  bày **Hộ Tâm Trận**.
- **Trận trình diễn 3** (`ch1_dem_mua`): mục tiêu “Đưa Hộ Tâm Trận lên tầng 3”. Đồ Cuồng không thể
  bị đánh bại; có đòn **Huyết Sát Trảm** niệm lâu — đưa Huyền Quy Thuẫn ra đỡ để chuyển sát thương
  thành trận lực; có **Phá Trận Quyền** đánh vỡ trận nhãn.
- Sau trận (`ch1_ong_mac_death`): ông Mạc phát trận đánh lui Đồ Cuồng nhưng phá luôn phong ấn tâm
  mạch. Trăn trối, trao **Tàn Ngọc** và **Tụ Linh Hồ Lô**. Tiếng Liên thét: một bóng áo trắng trên
  phi kiếm mang cô đi (Văn Trọng Khanh — chưa nêu tên). “Chạy về phía tây… Hàn Phong Lĩnh… Miếu
  Sơn Thần…” Nhiệm vụ **Chạy trốn** (`ch1_chay_tron`).

### Cảnh 7 — Rừng đêm
- Vào rừng: sói săn đuổi. **Trận trình diễn 1** (`ch1_lang_dem`): Lang Đầu + 2 Linh Lang. Linh
  Lang nhanh, Lang Đầu niệm “Cắn Xé”. Sỏi Ném đẩy lùi, Khói Mê Hương đẩy lùi cả bầy, Quét Gậy đánh lan.
- 🔍 Xác anh Võ Tráng bên gốc thông, vết vuốt và phi tiêu tẩm độc (`ch1_vo_trang_body`).

### Cảnh 8 — Miếu Sơn Thần (Hàn Phong Lĩnh)
- Miếu đổ nát giữa tuyết. Tương tác → **Tàn Ngọc thức tỉnh** (`ch1_awakening`).
- Ngọc lão xuất hiện, châm biếm, rồi nghiêm túc. ⚖ Tin tưởng (`trust.ngoc_lao +2`) / đề phòng
  (`+0`, Ngọc lão tôn trọng sự thận trọng) / giận dữ (“Vì ngươi mà ông ta chết!”, `stat.tam_ma +1`).
- Truyền **Nghịch Mệnh Quyết** tầng một: đau đớn, máu, rồi đột phá **Luyện Khí sơ kỳ**. Học **Phá
  Thạch Quyền** và **Tụ khí**.
- Tiếng sói tru — Lang Nha đã lần theo.

### Cảnh 9 — Cường địch Lang Nha (`ch1_lang_nha`)
**Trận trình diễn 2**: Lang Nha phóng phi tiêu ngắt Tụ khí bất cứ khi nào thấy người đang tích lực
(phản ứng hiển thị trên Ý đồ). Hai cách thắng: Tụ khí ngay sau lượt Lang Nha và dùng Sỏi Ném đẩy
hắn ra sau; hoặc diệt Linh Lang trước rồi dùng Tụ Linh Hồ Lô để tích hai bậc trong một lượt.
Sau trận (`ch1_after_lang_nha`): Lang Nha hấp hối buột miệng “Đồ đại ca… sẽ không tha…”, rồi
tắt thở. Trên người hắn có **thư lệnh không đề tên** đóng dấu mây xanh. 🔍 (Ch2: đó là mẫu dấu
của phe Đại trưởng lão Thanh Huyền Môn.)

### Cảnh 10 — Bình minh (`world.time = 4`)
- Trở về làng cháy. **Trưởng thôn** bị thương, **thím Ba** (và bé Đậu nếu đã cứu).
- **Mộ ông Mạc** (`ch1_grave`): chôn cất dưới gốc đào; nhận **Sổ thuốc của ông Mạc**.
- Nhặt **Lệnh bài ngoại môn Thanh Huyền Môn** ở sân (`ch1_token`). Ngọc lão: “Thanh Huyền Môn…
  muốn biết sự thật thì phải vào tận hang cọp.”
- Nói chuyện với trưởng thôn → quyết định lên đường (`ch1_farewell`) → thẻ chương “Hết Chương 1”,
  tự động lưu.

## 3. Nhiệm vụ

| ID | Loại | Tên | Mục tiêu | Mở khi | Thưởng | Tiếp |
|----|------|-----|----------|--------|--------|------|
| `ch1_hai_thuoc` | Chính | Hái Thanh Tâm Thảo | Hái 3 Thanh Tâm Thảo; đem về cho ông Mạc | Mở đầu | 20 đồng tiền | `ch1_le_hoi` |
| `ch1_le_hoi` | Chính | Lễ Cúng Thần Sông | Gặp Liên ở bến lúc hoàng hôn | Giao thuốc xong | – | `ch1_huyet_kiep` |
| `ch1_huyet_kiep` | Chính | Huyết Kiếp | Tìm ông Mạc | Mưa máu | – | `ch1_chay_tron` |
| `ch1_chay_tron` | Chính | Chạy trốn | Băng qua rừng; tìm Miếu Sơn Thần ở Hàn Phong Lĩnh | Ông Mạc mất | Tụ Linh Hồ Lô, Tàn Ngọc | `ch1_binh_minh` |
| `ch1_binh_minh` | Chính | Tro tàn và bình minh | Chôn cất ông Mạc; tìm manh mối; từ biệt | Thắng Lang Nha | Sổ thuốc, Lệnh bài | Chương 2 |
| `ch1_dieu` | Phụ | Con diều của bé Đậu | Lấy diều; trả bé Đậu | Nói chuyện bé Đậu | Sỏi Ném ×4 | – |
| `ch1_chao_ca` | Phụ | Nồi cháo cá của thím Ba | Xin cá chú Năm; mang về | Nói chuyện thím Ba | Bánh Đậu Xanh ×3, Khói Mê Hương | – |
| `ch1_tram_go` | Phụ | Trâm gỗ hoa sen | Gỗ đào; dao khắc; khắc trâm | Nói chuyện Liên (lần 2) | Trâm gỗ hoa sen | Ch6 |

Nhiệm vụ phụ còn dang dở khi lễ hội bắt đầu sẽ **thất bại** có chủ đích (thông báo “Không còn kịp
nữa…”), và được nhắc trước khi xác nhận.

## 4. Trận đấu

| ID | Kẻ địch | Mục tiêu | Ghi chú cân bằng |
|----|---------|----------|------------------|
| `ch1_da_tru` | Dã Trư | Đánh bại | Hướng dẫn. Húc niệm 400 tích, 180 % — Thủ thế giảm còn một nửa |
| `ch1_hac_y` | Hắc Y Tay Sai | Đánh bại | Phàm nhân đối phàm nhân. Có thể dùng Sỏi Ném |
| `ch1_dem_mua` | Đồ Cuồng (không thể hạ), Hắc Y Tay Sai | Hộ Tâm Trận tầng 3 | Khách: ông Mạc. Thua nếu nhân vật chính hoặc ông Mạc ngã |
| `ch1_lang_dem` | Lang Đầu, Linh Lang ×2 | Đánh bại | Không thể bỏ chạy |
| `ch1_lang_nha` | Lang Nha, Linh Lang | Đánh bại | Cường địch, kẻ ngắt quãng |

## 5. Cờ trạng thái

`ch1.herbs_delivered`, `ch1.dusk`, `ch1.raid_started`, `ch1.saved_dau`, `ch1.promise`,
`ch1.gave_hairpin`, `ch1.ong_mac_dead`, `ch1.awakened`, `ch1.lang_nha_defeated`, `ch1.buried`,
`ch1.token_found`, `ch1.complete`, `trust.ngoc_lao`, `trust.to_thanh_lien`, `stat.tam_ma`,
`stat.nhan_tam`, `world.time`.

## 6. Hệ quả cho các chương sau

- `ch1.saved_dau` → Ch6: bé Đậu (lớn hơn) dẫn đường bí mật vào Thanh Huyền Sơn.
- `ch1.promise` → Ch6: câu đầu tiên Liên (đã bị tẩy ký ức) vô thức lặp lại.
- `ch1.gave_hairpin` → Ch6: đánh thức Liên không cần chiến đấu (kết hợp niềm tin).
- `trust.ngoc_lao` → Ch3/Ch7: Ngọc lão chia sẻ ký ức sớm hơn; ảnh hưởng điều kiện kết thúc 3.

# Cẩm nang văn phong tiếng Việt — THIÊN MỆNH: TÀN HỒN

Tiếng Việt (`vi-VN`) là ngôn ngữ gốc và duy nhất của bản phát hành đầu tiên. Mọi chữ người chơi
nhìn thấy phải viết trực tiếp bằng tiếng Việt tự nhiên, có dấu đầy đủ. Tài liệu này là chuẩn để
viết, duyệt và kiểm thử nội dung.

---

## 1. Nguyên tắc chung

1. **Viết thẳng bằng tiếng Việt**, không dịch từ bản nháp tiếng Anh. Câu phải đọc lên tự nhiên.
2. **Có dấu đầy đủ, chuẩn Unicode NFC.** Không viết không dấu, không dùng ký tự tổ hợp rời.
3. **Không chữ tiếng Anh trong giao diện**, kể cả bản thử nghiệm sớm. Bảng thuật ngữ ở §3.
4. **Văn phong cổ phong vừa phải:** dùng từ Hán Việt cho thuật ngữ tu tiên, tên chiêu thức, cảnh
   giới; lời thoại đời thường dùng tiếng Việt thuần, dễ hiểu. Không nhồi từ cổ khó hiểu.
5. **Câu ngắn trong hộp thoại:** tối đa ~90 ký tự mỗi dòng thoại, 1–4 dòng mỗi lượt nói.
6. **Tránh lặp:** không để nhiều NPC nói cùng một câu chào; mỗi người có giọng riêng.
7. **Không lộ cốt truyện sớm** trong mô tả vật phẩm, tên nhiệm vụ, tooltip.

## 2. Định dạng

| Loại | Quy tắc | Ví dụ |
|------|---------|-------|
| Tên riêng | Viết hoa mỗi âm tiết | Thanh Khê thôn, Âu Dương Liệt |
| Thuật ngữ tu luyện | Viết hoa mỗi âm tiết khi là danh xưng cảnh giới | Luyện Khí, Trúc Cơ, Kết Đan |
| Tiểu cảnh giới | Viết thường | Luyện Khí sơ kỳ, Kết Đan đỉnh phong |
| Danh từ chung tu tiên | Viết thường | linh thạch, pháp bảo, linh căn, đan dược |
| Tên chiêu thức, pháp bảo, trận pháp | Viết hoa mỗi âm tiết | Phá Thạch Quyền, Tụ Linh Hồ Lô, Hộ Tâm Trận |
| Nút bấm, mục menu | Viết hoa chữ đầu | Bắt đầu hành trình, Túi đồ |
| Thông báo | Câu hoàn chỉnh, có dấu chấm | Đã lưu hành trình. |
| Số lượng | `×3`, không viết “x3” | Thanh Tâm Thảo ×3 |
| Dấu ngoặc kép trong lời thoại | “ ” | “Chạy đi!” |
| Dấu ba chấm | Một ký tự `…` | Ngươi… nghe thấy ta không? |

## 3. Thuật ngữ chuẩn (bắt buộc dùng thống nhất)

### 3.1 Giao diện

| Khái niệm | Tiếng Việt chuẩn | KHÔNG dùng |
|-----------|-----------------|-----------|
| New game | **Bắt đầu hành trình** | Start, Chơi mới |
| Continue | **Tiếp tục** | Continue |
| Load | **Tải hành trình** | Load |
| Save | **Lưu hành trình** / Lưu nhanh | Save |
| Settings | **Thiết lập** | Settings, Cài đặt (tránh lẫn) |
| Quit | **Rời khỏi** | Quit, Exit |
| Loading | **Đang tải…** | Loading |
| Inventory | **Túi đồ** | Inventory, Hành trang |
| Quest / journal | **Nhiệm vụ** / **Nhật ký nhiệm vụ** | Quest |
| Main quest / side quest | **Nhiệm vụ chính** / **Nhiệm vụ phụ** | |
| Skill / technique | **Công pháp** (bộ môn), **Chiêu thức** (đòn cụ thể) | Skill |
| Level / realm | **Cảnh giới** | Level |
| Level up (minor stage) | **Tinh tiến** | Level Up |
| Breakthrough | **Đột phá** | |
| EXP | **Tu vi** | EXP, XP |
| HP | **Khí huyết** | HP, Máu |
| MP / energy | **Linh lực** | MP, Mana |
| Action points | **Điểm hành động** (viết tắt **ĐHĐ**) | AP |
| Speed | **Thân pháp** | Speed, Tốc độ (chỉ dùng cho tốc độ trận đấu) |
| Attack / Spell power / Defense | **Công** / **Pháp** / **Thủ** | ATK, DEF |
| Turn order / timeline | **Thứ tự hành động** | Timeline |
| Charge | **Tụ khí** (bậc 1/2/3), **Quá Tụ** (bậc 4) | Charge |
| Formation | **Trận pháp**; node = **trận nhãn** | Formation |
| Artifact | **Pháp khí / Bảo khí / Linh khí / Pháp bảo / Bản mệnh pháp bảo** | Artifact, Item |
| Cooldown | **Hồi chiêu** (“còn 2 lượt”) | CD |
| Boss | **Cường địch** (trong giao diện), tên riêng trong lời thoại | Boss |
| Enemy intent | **Ý đồ** | Intent |
| Victory / Defeat | **Chiến thắng** / **Thất bại** | Victory, Game Over |
| Game over screen | **Hành trình gián đoạn** | Game Over |
| Gold / money | **Đồng tiền** (phàm nhân), **linh thạch** (tu sĩ) | Gold |
| Party | **Đội ngũ** | Party |
| Companion | **Đồng hành** | |
| Reputation | **Danh vọng** | |
| Codex | **Bách khoa** | |
| Autosave | **Tự động lưu** | |
| Status effect | **Trạng thái** | Buff, Debuff (dùng “cường hóa” / “suy yếu”) |
| Guard | **Thủ thế** | Defend |
| Flee | **Bỏ chạy** | Run |
| Back / Cancel | **Quay lại** / **Hủy** | Back |
| Confirm | **Xác nhận** | OK |
| Yes / No | **Đồng ý** / **Không** | |

### 3.2 Cảnh giới

**Phàm Nhân → Luyện Khí → Trúc Cơ → Kết Đan → Nguyên Anh → Hóa Thần.**
Tiểu cảnh giới: **sơ kỳ, trung kỳ, hậu kỳ, đỉnh phong.** Ví dụ: “Trúc Cơ hậu kỳ”.

### 3.3 Ngũ hành

**Kim, Mộc, Thủy, Hỏa, Thổ**; không thuộc hệ nào: **Vô hệ**. Viết “Thủy”, “Hỏa” (dấu đặt theo kiểu
cũ thống nhất toàn game: *Thủy, Hỏa, Hóa Thần, Họa*).

### 3.4 Quy tắc đặt dấu thanh

Dùng **kiểu cũ** (đặt dấu ở nguyên âm thứ nhất trong cặp “oa, oe, uy” khi không có phụ âm cuối):
*hòa, thủy, khỏe, Hóa Thần*. Khi có phụ âm cuối, đặt ở nguyên âm sau: *hoàng, thuyền, hoạch*.
Nguyên âm mang dấu phụ (â, ă, ê, ô, ơ, ư) luôn nhận dấu thanh: *tuấn, người, thuở*.
Bộ gõ Telex trong game tuân theo đúng quy tắc này.

## 4. Xưng hô

Xưng hô thể hiện vai vế, tuổi tác, thân sơ, phe phái. **Không chọn ngẫu nhiên.**

### 4.1 Trong tông môn

| Quan hệ | Gọi người kia | Tự xưng |
|---------|---------------|---------|
| Đệ tử → sư phụ | sư phụ | đệ tử / con (thân thiết) |
| Đệ tử → sư huynh / sư tỷ (vào trước) | sư huynh / sư tỷ | sư đệ / sư muội / đệ / muội |
| Đệ tử → trưởng lão | trưởng lão | đệ tử |
| Trưởng lão → đệ tử | tên / “ngươi” / “con” (thân) | ta / bổn tọa (kiêu ngạo) |
| Tu sĩ ngang hàng, không quen | đạo hữu | tại hạ / ta |
| Vãn bối → tiền bối | tiền bối | vãn bối |
| Tu sĩ → phàm nhân | ngươi / tiểu tử (miệt thị) | ta / bản tọa |

### 4.2 Trong thôn (Chương 1) — tiếng Việt đời thường

| Người nói → người nghe | Gọi | Tự xưng |
|------------------------|-----|---------|
| Ông Mạc → nhân vật chính | con, `{playerName}` | ông |
| Nhân vật chính → ông Mạc | ông | con |
| Liên ↔ nhân vật chính | tên / cậu | tớ |
| Bé Đậu → nhân vật chính | `{g:anh|chị|anh}` + tên hoặc không tên | em |
| Thím Ba → nhân vật chính | con / `{g:thằng quỷ nhỏ|con quỷ nhỏ|đứa quỷ nhỏ}` (trêu) | thím |
| Chú Sơn → nhân vật chính | cháu | chú |
| Trưởng thôn → dân | cháu / bà con | ta / ông |

### 4.3 Ngọc lão

- Tự xưng **lão phu**. Gọi nhân vật chính: `{g:tiểu tử|nha đầu|tiểu gia hỏa}`; từ Chương 5 gọi
  `{playerName}`.
- Nhân vật chính gọi: **Ngọc lão** (lịch sự), “lão già” (khi giận — lựa chọn đối thoại).

### 4.4 Phản diện

- Văn Trọng Khanh: lễ độ lạnh lùng, “tại hạ” / “các hạ”, không bao giờ chửi.
- Đồ Cuồng: thô lỗ, “lão tử”, gọi người khác “lũ sâu kiến”.
- Lang Nha: nham hiểm, “ta”, gọi nhân vật chính “con chuột nhắt”.

### 4.5 Xưng hô theo lựa chọn giới của người chơi

Người chơi chọn **Nam**, **Nữ** hoặc **Trung tính**. Mọi chỗ phụ thuộc giới dùng token
`{g:nam|nữ|trung tính}`. Dạng trung tính dùng từ không chỉ giới: *nhóc con, đứa nhỏ, đạo hữu,
tiểu gia hỏa, người ấy*. Khi một câu khó viết trung tính, viết lại câu (không gọi bằng đại từ).
Ví dụ:

| Nam | Nữ | Trung tính |
|-----|----|------------|
| thằng nhóc | con bé | nhóc con |
| sư huynh | sư tỷ | sư huynh/sư tỷ → dùng “đồng môn” hoặc tên |
| tiểu tử | nha đầu | tiểu gia hỏa |
| anh | chị | anh (bé Đậu quen gọi) / tên |

## 5. Tên riêng

- Hán Việt, 2–3 âm tiết, có nghĩa phù hợp tính cách (Diệp Hàn Sương: lá – lạnh – sương).
- Không trùng tên nhân vật nổi tiếng trong tiểu thuyết/trò chơi có bản quyền (xem danh sách
  kiểm tra trong `story-bible.md`).
- Tên người chơi: 2–24 ký tự, chữ cái tiếng Việt, khoảng trắng, dấu gạch nối và dấu nháy đơn.
  Thông báo lỗi bằng tiếng Việt: “Tên phải có từ 2 đến 24 ký tự.”, “Tên chỉ được chứa chữ cái,
  khoảng trắng, dấu gạch nối hoặc dấu nháy.”

## 6. Giọng điệu theo ngữ cảnh

| Ngữ cảnh | Giọng | Ví dụ |
|----------|-------|-------|
| Đời thường ở thôn | Ấm, dí dỏm, khẩu ngữ | “Nồi cháo cá của thím mà nguội là thím cạo đầu con đấy.” |
| Bi kịch | Câu ngắn, đứt quãng | “Ông… ông ơi…” |
| Tu sĩ cao giai | Trang trọng, Hán Việt | “Thiên đạo vô tình, coi vạn vật như chó rơm.” |
| Hướng dẫn | Rõ ràng, trực tiếp, không hoa mỹ | “Nhấn Tụ khí để tích lực. Bị đánh mạnh sẽ mất Tụ khí.” |
| Nhật ký chiến đấu | Ngắn, động từ mạnh | “Lang Nha cào trúng {target}, gây 18 sát thương.” |

## 7. Kiểm tra chất lượng (QA)

Tự động (cargo test, `content::validate`):
- [ ] Không thiếu khóa `vi-VN`; không có khóa thừa.
- [ ] Token nội suy hợp lệ; `{g:…}` đủ 3 lựa chọn.
- [ ] Không có từ tiếng Anh cấm trong chuỗi hiển thị.
- [ ] Mọi ký tự đều có glyph trong phông chữ đi kèm.
- [ ] Chuỗi là NFC (không có dấu tổ hợp rời).

Thủ công (trước khi đánh dấu hoàn thành một chương):
- [ ] Đọc to từng đoạn hội thoại: tự nhiên, đúng vai vế.
- [ ] Thử cả ba lựa chọn xưng hô; không có câu sai ngữ pháp.
- [ ] Thử tên dài (24 ký tự) và tên có dấu phức tạp (“Nguyễn Thị Hường”) trong mọi khung thoại.
- [ ] Không có lỗi chính tả (s/x, ch/tr, dấu hỏi/ngã).

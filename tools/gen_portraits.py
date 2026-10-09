#!/usr/bin/env python3
"""Generates the original dialogue portraits (48×48 pixel-art busts).

Each portrait is built from a small spec (skin, hair style and colours, eyes,
clothes, extras such as a nón lá, beard or scar), with colours matching the
character's overworld sheet. Output: assets/gfx/portraits/<id>.png, where <id>
is a character id, or `player_<sheet>` for the four protagonist looks.

Requires Pillow. Run from the repository root:  python3 tools/gen_portraits.py
"""

from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path("assets/gfx/portraits")
S = 48


def rgb(hex_):
    hex_ = hex_.lstrip("#")
    return tuple(int(hex_[i:i + 2], 16) for i in (0, 2, 4)) + (255,)


def shade(c, k):
    return tuple(max(0, min(255, int(v * k))) for v in c[:3]) + (c[3],)


SKIN = {
    "light": rgb("f2c9a0"),
    "tan": rgb("d9a273"),
    "dark": rgb("a86b45"),
    "pale": rgb("e8d2bd"),
    "old": rgb("d6a47c"),
}


def put(d, pts, c):
    for p in pts:
        d.point(p, fill=c)


def base(d, spec):
    skin = SKIN[spec.get("skin", "light")]
    skin_dark = shade(skin, 0.78)
    outline = shade(skin, 0.45)
    cloth = rgb(spec["cloth"])
    inner = rgb(spec.get("inner", "eee6d2"))
    # Shoulders and robe with a crossed collar (áo giao lĩnh).
    d.polygon([(4, 47), (8, 39), (17, 35), (31, 35), (40, 39), (44, 47)], fill=cloth,
              outline=shade(cloth, 0.5))
    d.polygon([(19, 35), (24, 44), (29, 35), (27, 35), (24, 40), (21, 35)], fill=inner)
    d.line([(19, 35), (25, 45)], fill=shade(cloth, 0.55))
    d.line([(29, 35), (26, 41)], fill=shade(inner, 0.7))
    if spec.get("armor"):
        armor = rgb(spec["armor"])
        d.polygon([(6, 47), (9, 40), (16, 37), (16, 47)], fill=armor, outline=shade(armor, 0.5))
        d.polygon([(42, 47), (39, 40), (32, 37), (32, 47)], fill=armor, outline=shade(armor, 0.5))
    if spec.get("necklace"):
        neck = rgb(spec["necklace"])
        put(d, [(20, 37), (22, 39), (24, 40), (26, 39), (28, 37)], neck)
        put(d, [(24, 41)], rgb("f4f0e0"))
    # Neck, ears, head.
    d.rectangle([21, 29, 27, 36], fill=skin_dark)
    d.ellipse([12, 19, 16, 25], fill=skin, outline=outline)
    d.ellipse([32, 19, 36, 25], fill=skin, outline=outline)
    d.ellipse([14, 9, 34, 33], fill=skin, outline=outline)
    for y in range(12, 31):
        for x in range(30, 33):
            if d.im.getpixel((x, y))[:3] == skin[:3]:
                d.point((x, y), fill=skin_dark)
    # Eyes.
    eye = rgb(spec.get("eye", "3a2a20"))
    for x0 in (18, 27):
        put(d, [(x0, 21), (x0 + 1, 21), (x0 + 2, 21)], outline)
        put(d, [(x0, 22), (x0 + 1, 22)], eye)
        put(d, [(x0 + 2, 22)], outline)
        put(d, [(x0 + 1, 23)], shade(eye, 0.7))
        if not spec.get("narrow"):
            put(d, [(x0, 23)], eye)
            put(d, [(x0 + 1, 22)], rgb("ffffff"))
    # Brows.
    brow = rgb(spec.get("brow", spec.get("hair", "3a2a20")))
    mood = spec.get("mood", "calm")
    if mood == "angry":
        put(d, [(17, 18), (18, 18), (19, 19), (20, 19), (28, 19), (29, 19), (30, 18), (31, 18)], brow)
    elif mood == "sad":
        put(d, [(17, 19), (18, 19), (19, 18), (20, 18), (28, 18), (29, 18), (30, 19), (31, 19)], brow)
    else:
        put(d, [(18, 19), (19, 18), (20, 18), (28, 18), (29, 18), (30, 19)], brow)
    # Nose and mouth.
    put(d, [(24, 25), (25, 26)], skin_dark)
    lips = shade(skin, 0.6)
    if spec.get("smile"):
        put(d, [(21, 28), (22, 29), (23, 29), (24, 29), (25, 29), (26, 28)], lips)
    elif mood == "angry":
        put(d, [(21, 29), (22, 28), (23, 28), (24, 28), (25, 28), (26, 29)], lips)
        put(d, [(23, 29), (25, 29)], rgb("f4f0e0"))
    else:
        put(d, [(22, 28), (23, 28), (24, 28), (25, 28)], lips)
    if spec.get("cheeks"):
        put(d, [(16, 26), (17, 26), (31, 26), (32, 26)], rgb("e8908a"))
    if spec.get("wrinkles"):
        put(d, [(17, 24), (31, 24), (20, 30), (28, 30)], skin_dark)
    return skin, outline


def hair(d, spec, layer):
    style = spec.get("style", "short")
    c = rgb(spec.get("hair", "3a2a20"))
    dark = shade(c, 0.6)
    light = shade(c, 1.25)
    if layer == "back":
        if style in ("long", "long_flower"):
            d.polygon([(12, 14), (36, 14), (40, 42), (8, 42)], fill=c, outline=dark)
        if style == "bun_long":
            d.polygon([(13, 14), (35, 14), (37, 38), (11, 38)], fill=c, outline=dark)
        return
    if style == "hood":
        hood = rgb(spec.get("accent", "2a2830"))
        d.polygon([(10, 34), (11, 12), (17, 5), (31, 5), (37, 12), (38, 34), (34, 34), (33, 15),
                   (24, 11), (15, 15), (14, 34)], fill=hood, outline=shade(hood, 0.5))
        d.rectangle([15, 25, 33, 33], fill=hood)
        put(d, [(x, 25) for x in range(16, 33)], shade(hood, 1.4))
        return
    if style == "bald_old":
        d.polygon([(13, 14), (16, 11), (14, 24), (12, 22)], fill=c)
        d.polygon([(35, 14), (32, 11), (34, 24), (36, 22)], fill=c)
        return
    # Fringe over the top of the head.
    d.chord([13, 6, 35, 30], 180, 360, fill=c, outline=dark)
    if style == "spiky":
        for x0, top in ((13, 3), (18, 0), (23, 2), (28, 0), (32, 4)):
            d.polygon([(x0, 14), (x0 + 3, top), (x0 + 6, 14)], fill=c, outline=dark)
        bangs = [(16, 16), (19, 19), (22, 16), (25, 19), (28, 16), (31, 18)]
    else:
        bangs = [(15, 15), (17, 18), (20, 16), (23, 18), (26, 16), (29, 18), (32, 15)]
    d.polygon([(14, 15)] + bangs + [(34, 15), (34, 12), (14, 12)], fill=c)
    # Side locks.
    if style in ("long", "long_flower", "bun_long"):
        d.polygon([(13, 15), (16, 15), (16, 34), (12, 34)], fill=c, outline=dark)
        d.polygon([(35, 15), (32, 15), (32, 34), (36, 34)], fill=c, outline=dark)
    else:
        d.polygon([(13, 14), (16, 14), (15, 22), (13, 21)], fill=c)
        d.polygon([(35, 14), (32, 14), (33, 22), (35, 21)], fill=c)
    # Highlight streak.
    put(d, [(18, 10), (19, 9), (20, 9), (21, 9), (22, 10)], light)
    if style in ("bun", "bun_long"):
        d.ellipse([19, 1, 29, 10], fill=c, outline=dark)
        put(d, [(21, 3), (22, 3)], light)
        pin = rgb(spec.get("accent", "c8a050"))
        d.line([(16, 6), (32, 4)], fill=pin)
    if style == "pigtails":
        d.ellipse([7, 9, 15, 19], fill=c, outline=dark)
        d.ellipse([33, 9, 41, 19], fill=c, outline=dark)
        bow = rgb(spec.get("accent", "d63a3a"))
        put(d, [(14, 9), (15, 10), (33, 10), (34, 9), (14, 11), (34, 11)], bow)
    if style == "long_flower":
        flower = rgb(spec.get("accent", "f08aa8"))
        d.ellipse([29, 9, 35, 15], fill=flower, outline=shade(flower, 0.6))
        put(d, [(32, 12)], rgb("ffe080"))


def extras(d, spec, skin, outline):
    if spec.get("headband"):
        band = rgb(spec["headband"])
        d.rectangle([14, 13, 34, 14], fill=band)
        put(d, [(35, 14), (36, 15), (37, 17), (36, 13)], band)
    if spec.get("beard"):
        b = rgb(spec["beard"])
        length = spec.get("beard_len", 40)
        d.polygon([(18, 27), (30, 27), (29, length - 4), (24, length), (19, length - 4)], fill=b,
                  outline=shade(b, 0.7))
        put(d, [(22, 28), (23, 28), (24, 28), (25, 28)], shade(skin, 0.6))
        put(d, [(19, 26), (20, 25), (21, 25), (27, 25), (28, 25), (29, 26)], b)
    if spec.get("stubble"):
        for x in range(18, 31, 2):
            for y in (29, 31):
                put(d, [(x + (y % 2), y)], shade(skin, 0.7))
    if spec.get("scar"):
        scar = rgb("c45a5a")
        put(d, [(26, 17), (27, 18), (28, 19), (28, 20), (29, 21), (29, 23), (30, 24), (30, 25)], scar)
    if spec.get("mask"):
        m = rgb(spec["mask"])
        d.rectangle([15, 25, 33, 32], fill=m)
        put(d, [(x, 25) for x in range(15, 34)], shade(m, 1.4))
    if spec.get("non_la"):
        straw = rgb("d8b878")
        d.polygon([(3, 16), (24, 0), (45, 16)], fill=straw, outline=shade(straw, 0.55))
        for i in range(1, 4):
            y = 4 * i
            half = int(21 * y / 16)
            d.line([(24 - half + 1, y), (24 + half - 1, y)], fill=shade(straw, 0.8))
        d.line([(3, 16), (45, 16)], fill=shade(straw, 0.5))
        d.line([(15, 16), (19, 33)], fill=shade(straw, 0.45))


def spirit(img):
    """Translucent jade tint for Ngọc lão's soul."""
    px = img.load()
    for y in range(S):
        for x in range(S):
            r, g, b, a = px[x, y]
            if a == 0:
                continue
            lum = (r * 0.3 + g * 0.59 + b * 0.11) / 255
            lum = lum ** 1.4
            px[x, y] = (int(30 + 170 * lum), int(110 + 145 * lum), int(100 + 140 * lum), 225)
    glow = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    gp = glow.load()
    for y in range(S):
        for x in range(S):
            if px[x, y][3]:
                continue
            near = any(
                0 <= x + dx < S and 0 <= y + dy < S and px[x + dx, y + dy][3]
                for dx in (-1, 0, 1) for dy in (-1, 0, 1)
            )
            if near:
                gp[x, y] = (150, 255, 220, 110)
    glow.alpha_composite(img)
    return glow


def portrait(spec):
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    hair(d, spec, "back")
    skin, outline = base(d, spec)
    hair(d, spec, "front")
    extras(d, spec, skin, outline)
    if spec.get("spirit"):
        img = spirit(img)
    return img


# Colours follow each character's overworld sheet (ow1..ow10).
PORTRAITS = {
    # The four protagonist looks (character creation sheets 1, 3, 8, 2).
    "player_1": dict(skin="light", hair="6b4128", cloth="c0392b", inner="f0ece0", headband="2e9b8f",
                     eye="2e6b5e"),
    "player_3": dict(skin="tan", style="long", hair="7b4fa0", cloth="5a4a8a", inner="e8d8f0",
                     eye="2e6b5e"),
    "player_8": dict(skin="light", hair="4b3d6b", cloth="3f7f8f", inner="e9dfc8", eye="3a2a50"),
    "player_2": dict(skin="light", style="bun", hair="d9a440", cloth="b8312f", inner="f3efe5",
                     eye="6b4128", accent="8b5a2b"),
    "ong_mac": dict(skin="old", style="bald_old", hair="e8e4dc", brow="e8e4dc", beard="ece8e0",
                    cloth="8a6a44", inner="e9dfc8", non_la=True, wrinkles=True, smile=True,
                    narrow=True),
    "to_thanh_lien": dict(skin="tan", style="long_flower", hair="3fb8b0", cloth="3d78c8",
                          inner="eef4ff", eye="2a4f7a", smile=True, cheeks=True, accent="f08aa8"),
    "ngoc_lao": dict(skin="pale", style="bun_long", hair="f0f0f0", brow="f0f0f0", beard="f4f4f4",
                     beard_len=46, cloth="dcdcdc", inner="ffffff", narrow=True, wrinkles=True,
                     spirit=True, accent="b0e0d0"),
    "thim_ba": dict(skin="tan", style="pigtails", hair="6b3f2a", cloth="d07a3a", inner="f5e6c8",
                    smile=True, cheeks=True, accent="d63a3a"),
    "be_dau": dict(skin="light", hair="3b3050", cloth="3f8a8f", inner="e9dfc8", cheeks=True,
                   smile=True),
    "ly_duc": dict(skin="old", style="bald_old", hair="c8c4bc", brow="c8c4bc", beard="d8d4cc",
                   beard_len=36, cloth="6d5a8a", inner="e9e2d0", wrinkles=True, mood="sad"),
    "to_dai_son": dict(skin="dark", hair="1e1a18", cloth="5a4a3a", inner="c8b89a",
                       headband="8a5a2a", beard="241e1c", beard_len=34, narrow=True),
    "vo_trang": dict(skin="dark", hair="2a2220", cloth="5a7a3a", inner="d8c8a8", headband="3c8c3c",
                     smile=True),
    "hoang_khai": dict(skin="dark", style="spiky", hair="c0392b", cloth="8a3a2a", inner="e0c8a0",
                       eye="5a2a20"),
    "chu_nam": dict(skin="tan", hair="2f6b3a", cloth="3a6a7a", inner="d8d0c0", non_la=True,
                    stubble=True, smile=True),
    "lao_ha": dict(skin="old", style="bun", hair="bdb8b0", brow="bdb8b0", beard="d0ccc4",
                   beard_len=44, cloth="3a5a6a", inner="d8d0c0", wrinkles=True, narrow=True,
                   smile=True, accent="5a4030"),
    "tinh_an": dict(skin="pale", style="bun_long", hair="2e2240", cloth="9a8ab8", inner="f0e8f8",
                    eye="4a2a6a", mood="sad", accent="d8d0e8"),
    "lao_moc": dict(skin="old", style="bun", hair="a89a8a", brow="a89a8a", beard="bfb4a8",
                    beard_len=32, cloth="6b5a3a", inner="d8c8a8", wrinkles=True, accent="6b4a2a"),
    "lang_nha": dict(skin="pale", style="spiky", hair="5f7a6a", cloth="4a4e58", inner="8a8e98",
                     eye="c8a020", mood="angry", narrow=True, necklace="e8e0c8"),
    "do_cuong": dict(skin="dark", style="spiky", hair="b02a20", cloth="6a1e1a", inner="2a1a18",
                     armor="3a2a28", eye="e0a020", mood="angry", scar=True, stubble=True),
    "hac_y_tay_sai": dict(skin="tan", style="hood", hair="1a1820", cloth="2a2830", inner="3a3840",
                          accent="24222a", mask="1e1c24", mood="angry", narrow=True),
}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for name, spec in PORTRAITS.items():
        portrait(spec).save(OUT / f"{name}.png")
    sheet = Image.new("RGBA", (S * 5, S * ((len(PORTRAITS) + 4) // 5)), (30, 30, 36, 255))
    for i, name in enumerate(PORTRAITS):
        sheet.alpha_composite(Image.open(OUT / f"{name}.png"), ((i % 5) * S, (i // 5) * S))
    print(f"{len(PORTRAITS)} portraits written to {OUT}")
    return sheet


if __name__ == "__main__":
    main()

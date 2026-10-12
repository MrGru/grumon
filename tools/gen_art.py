#!/usr/bin/env python3
"""Generates original prototype art for THIÊN MỆNH: TÀN HỒN (requires Pillow).

Outputs (all original, see docs/asset-manifest.md):
  assets/gfx/objects/objects.png   map objects (herb, kite, jar, grave, shrine…)
  assets/gfx/enemies/*.png         beast battle sprites (32x32)
  assets/gfx/battle/*.png          battle backgrounds (960x640), graded from
                                   assets/gfx/backgrounds/background*.png
  assets/gfx/ui/title.png          ink-wash mountains title background

Run from the repository root:  python3 tools/gen_art.py
"""

import math
import random
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ROOT = Path("assets/gfx")


def sprite(rows, palette):
    h, w = len(rows), max(len(r) for r in rows)
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    px = img.load()
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch in palette:
                px[x, y] = palette[ch]
    return img


def pad(rows, width, height):
    rows = [r.ljust(width, ".")[:width] for r in rows]
    top = height - len(rows)
    return ["." * width] * top + rows


# --------------------------------------------------------------------- beasts
WOLF_PAL = {
    "K": (24, 24, 36, 255), "G": (96, 112, 140, 255), "D": (64, 74, 98, 255),
    "L": (178, 190, 205, 255), "E": (230, 60, 50, 255), "N": (20, 20, 20, 255),
    "W": (240, 240, 240, 255),
}
ALPHA_PAL = dict(WOLF_PAL, G=(70, 74, 92, 255), D=(44, 46, 60, 255), L=(150, 150, 165, 255),
                 E=(255, 90, 40, 255))
WOLF = [
    ".......................K..K.....",
    "......................KGKKGK....",
    ".....................KGGGGGGK...",
    "....................KGGGEGGGGK..",
    "..............KKKKKGGGGGGGGLLNK.",
    "....KK......KKGGGGGGGGGGGGLLLWK.",
    "...KGGK...KKGGGGGGGGGGGGGGLLLK..",
    "...KGGGKKKGGGGGGGGGGGGGGGGKKK...",
    "....KGGGGGGGGGGGGGGGGGGGGLLK....",
    ".....KGGGGGGGGGGGGGGGGGGLLLK....",
    "......KGGGGGGGGGGGGGGGGGLLK.....",
    "......KDGGGGGGGGGGGGGGGGGGK.....",
    "......KDDGGGGGGGGGGGGGDDGGK.....",
    ".......KDDDGGGGGGGGGDDDKGGK.....",
    ".......KDDKKDDDDDDDDKK.KDGK.....",
    ".......KGK..KDK..KDK...KGK......",
    ".......KGK..KGK..KGK...KGK......",
    "......KKGK.KKGK.KKGK..KKGK......",
    "......KKK..KKK..KKK...KKK.......",
]
BOAR_PAL = {
    "K": (30, 20, 16, 255), "B": (120, 82, 54, 255), "D": (84, 56, 36, 255),
    "E": (240, 200, 40, 255), "N": (196, 120, 110, 255), "W": (245, 240, 220, 255),
    "H": (60, 40, 28, 255),
}
BOAR = [
    "...........KKKKKKKKK............",
    ".........KKHHHHBBBBBKK..........",
    ".......KKHHHHBBBBBBBBBKKK.......",
    "......KBBBBBBBBBBBBBBBBBBKK.....",
    ".....KBBBBBBBBBBBBBBBBBBBBBKK...",
    "....KBBBBBBBBBBBBBBBBBBBBBBBBK..",
    "....KBBBBBBBBBBBBBBBBBBBBBEBBBK.",
    "....KDBBBBBBBBBBBBBBBBBBBBBBBNNK",
    "....KDDBBBBBBBBBBBBBBBBBBBBWBNNK",
    "....KDDDBBBBBBBBBBBBBBBBBBWWBBK.",
    ".....KDDDDBBBBBBBBBBBBBBBKWKKK..",
    ".....KDDDDDDBBBBBBBBBDDDK.......",
    "......KDDKKDDDDDDDDDKKDDK.......",
    "......KDK..KDDK..KDK..KDK.......",
    "......KDK..KDK...KDK..KDK.......",
    "......KKK..KKK...KKK..KKK.......",
]

# --------------------------------------------------------------------- objects
OBJ_PAL = {
    "K": (30, 26, 22, 255), "Y": (255, 236, 140, 255), "W": (255, 255, 255, 255),
    "R": (214, 54, 46, 255), "r": (150, 30, 30, 255), "O": (245, 140, 40, 255),
    "B": (120, 80, 48, 255), "b": (84, 54, 32, 255), "T": (196, 150, 98, 255),
    "G": (60, 150, 120, 255), "g": (36, 104, 86, 255), "C": (110, 200, 220, 255),
    "P": (240, 130, 160, 255), "p": (200, 80, 120, 255), "S": (150, 150, 150, 255),
    "s": (100, 100, 105, 255), "E": (110, 84, 60, 255), "e": (80, 60, 44, 255),
    "M": (120, 36, 30, 255), "D": (60, 58, 66, 255), "d": (40, 38, 46, 255),
    "L": (200, 190, 170, 255),
}
SPARKLE = [
    "................", "................", ".......Y........", ".......Y........",
    "......YWY.......", "...YYYWWWYYY....", "......YWY.......", ".......Y........",
    ".......Y........", "............Y...", "...........YWY..", "............Y...",
    "................", "................", "................", "................",
]
KITE = [
    "........K.......", ".......KRK......", "......KRRRK.....", ".....KRRYRRK....",
    "....KRRYYYRRK...", ".....KRRYRRK....", "......KRRRK.....", ".......KRK......",
    "........K.......", "........r.......", ".......r........", "........r.......",
    ".........r......", "........r.......", ".......r........", "................",
]
DRIFTWOOD = [
    "................", "................", "................", "................",
    "................", "................", "................", "........KK......",
    "..KKKKKKKTTKK...", ".KTTTBTTTTBTTK..", "KTBBTTTBBTTTBTK.", ".KBBBBBBBBBBBK..",
    "..KKKKKKKKKKK...", "................", "................", "................",
]
HERB = [
    "................", "................", "......C...C.....", ".....CGC.CGC....",
    "....CGGC.CGGC...", ".C..CGGCCGGC..C.", "CGC..CGGGGC..CGC", ".CGC.CGGGGC.CGC.",
    "..CGCGGgGGGCGC..", "...CGGGgGGGGC...", "....CGGgGGGC....", ".....CGgGGC.....",
    "......Kg.K......", ".....KgggK......", "....KbbbbbK.....", "................",
]
JAR = [
    "................", "................", ".....KKKKKK.....", "....KMMRRMMK....",
    ".....KbbbbK.....", "....KEEEEEEK....", "...KEeEEEEeEK...", "...KELLLLLLEK...",
    "...KELYYYYLEK...", "...KELLLLLLEK...", "...KEEEEEEEEK...", "...KeEEEEEEeK...",
    "....KeeEEeeK....", ".....KKKKKK.....", "................", "................",
]
FIRE = [
    "........Y.......", ".......YY.......", "......YYOY......", "....Y.YOOY..Y...",
    "....YYOOOOYYY...", "...YOOOROOOOY...", "...YOORRROOOY...", "..YOORRRRROOY...",
    "..YORRRrRRROY...", "..YORRrrrRROY...", "...ORRrrrRRO....", "...KbbBbbBbK....",
    "..KbBKbbKBbbK...", "...KKKKKKKKK....", "................", "................",
]
LANTERN = [
    "................", "................", "................", ".......Y........",
    "......YWY.......", ".....PYWYP......", "...p.PPYPP.p....", "...ppPPPPPpp....",
    "...pPPPPPPPp....", "....ppPPPpp.....", ".CC.........CC..", "C..CC.CCC.CC..C.",
    "................", "................", "................", "................",
]
GRAVE = pad([
    "..............KK................",
    ".............KTTK...............",
    ".............KTTK..S.S..........",
    ".............KTTK..S.S..........",
    ".............KTTK..R.R..........",
    ".............KTTK..K.K..........",
    "......KKKKKKKKTTKKKKKKKK........",
    "....KKEEEEEEEETTEEEEEEEEKK......",
    "...KEEEEeEEEEETTEEEEeEEEEEK.....",
    "..KEEEeEEEEEEEEEEEEEEEeEEEEK....",
    ".KEEEEEEEEEeEEEEEEEEEEEEEEEEK...",
    ".KEeEEEEEEEEEEEEEEeEEEEEEEEEK...",
    "KEEEEEEEeEEEEEEEEEEEEEEEEeEEEK..",
    "KEEEEEEEEEEEEEEEeEEEEEEEEEEEEK..",
    ".KKeeeeeeeeeeeeeeeeeeeeeeeeKK...",
    "...KKKKKKKKKKKKKKKKKKKKKKKKK....",
], 32, 32)
BENCH = pad([
    "................................",
    "..KKKKKKKKKKKKKKKKKKKKKKKKKKKK..",
    ".KTTTTTTTTTTTTTTTTTTTTTTTTTTTTK.",
    ".KTTTTTGGTTTTTTTTTTTTTTSSTTTTTK.",
    ".KBBBBBBBBBBBBBBBBBBBBBBBBBBBBK.",
    ".KbbbbbbbbbbbbbbbbbbbbbbbbbbbbK.",
    "..KKBBKKKKKKKKKKKKKKKKKKKKBBKK..",
    "...KBBK..................KBBK...",
    "...KBBK..................KBBK...",
    "...KBBK..................KBBK...",
    "...KBbK..................KBbK...",
    "...KBbK..................KBbK...",
    "...KBbK..................KBbK...",
    "...KKKK..................KKKK...",
], 32, 24)
SHRINE = pad([
    "...................KK...........................",
    "................KKKddKKK........................",
    ".............KKKdddddddKKK......................",
    "..KK......KKKdddDDDDDDDdddKKK......KK...........",
    "..KdKK.KKKdddDDDDDDDDDDDDDdddKKK.KKdK...........",
    "...KddddddDDDDDDDDDDDDDDDDDDDdddddK.............",
    "....KKKKKKKKKKKKKKKKKKKKKKKKKKKKKK..............",
    "......KMMK...................KMMK...............",
    "......KMMK...KKKKKKKKKKKK....KMMK...............",
    "......KMMK...KSSSSSSSSSSK....KMMK...............",
    "......KMMK...KSsSSLLSSsSK....KMMK...............",
    "......KMMK...KSSSLLLLSSSK....KMMK...............",
    "......KMMK...KSSSLKKLSSSK....KMMK...............",
    "......KMMK...KSSSSLLSSSSK....KMMK...............",
    "......KMMK...KSSSSSSSSSSK....KMMK...............",
    "......KMMK...KKKKKKKKKKKK....KMMK...............",
    "......KMMK..KSSSSSSSSSSSSK...KMMK...............",
    "......KMMK..KSsSSSSYSSSsSK...KMMK...............",
    "......KMMK..KSSSSSSYSSSSSK...KMMK...............",
    ".....KSSSSKKKSSSSSSSSSSSSKKKKSSSSK..............",
    "....KSSSSSSSSSSSSSSSSSSSSSSSSSSSSSK.............",
    "....KSsSSSSsSSSSSsSSSSSsSSSSSsSSSSK.............",
    "....KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK.............",
], 48, 48)
BODY = pad([
    "................................",
    "........KK......................",
    ".......KSSK.....................",
    "......KSSSSK.........M..........",
    "......KSSSK......MM.MMM..M......",
    ".......KKBBK....MMMMMMMMMM......",
    ".........KBBK....MMrrrMMM.......",
    "..........KBK.....MMrrMM.KKK....",
    "...........KK.....MMMMM.KBBBK...",
    "........................KKKK....",
], 32, 16)


def build_objects():
    sheet = Image.new("RGBA", (128, 64), (0, 0, 0, 0))
    for (x, y), rows in {
        (0, 0): SPARKLE, (16, 0): KITE, (32, 0): DRIFTWOOD, (48, 0): HERB,
        (64, 0): FIRE, (80, 0): LANTERN, (96, 0): JAR, (0, 16): GRAVE, (32, 16): BENCH,
        (64, 16): SHRINE, (0, 48): BODY,
    }.items():
        sheet.alpha_composite(sprite(rows, OBJ_PAL), (x, y))
    out = ROOT / "objects" / "objects.png"
    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)


def build_enemies():
    out = ROOT / "enemies"
    out.mkdir(parents=True, exist_ok=True)
    sprite(pad(WOLF, 32, 32), WOLF_PAL).save(out / "linh_lang.png")
    sprite(pad(WOLF, 32, 32), ALPHA_PAL).save(out / "lang_dau.png")
    sprite(pad(BOAR, 32, 32), BOAR_PAL).save(out / "da_tru.png")


# --------------------------------------------------------------------- backgrounds
W, H = 960, 640


def cover(img):
    scale = max(W / img.width, H / img.height)
    resized = img.convert("RGB").resize((round(img.width * scale), round(img.height * scale)), Image.LANCZOS)
    left = (resized.width - W) // 2
    top = (resized.height - H) // 2
    return resized.crop((left, top, left + W, top + H))


def grade(img, mul, add=(0, 0, 0)):
    r, g, b = img.split()
    r = r.point(lambda v: min(255, int(v * mul[0] + add[0])))
    g = g.point(lambda v: min(255, int(v * mul[1] + add[1])))
    b = b.point(lambda v: min(255, int(v * mul[2] + add[2])))
    return Image.merge("RGB", (r, g, b))


def vignette(img, strength=0.55):
    mask = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(mask)
    d.ellipse((-W * 0.25, -H * 0.3, W * 1.25, H * 1.3), fill=255)
    mask = mask.filter(ImageFilter.GaussianBlur(120))
    dark = Image.new("RGB", (W, H), (0, 0, 0))
    return Image.composite(img, Image.blend(img, dark, strength), mask)


def build_backgrounds():
    src = ROOT / "backgrounds"
    out = ROOT / "battle"
    out.mkdir(parents=True, exist_ok=True)
    rng = random.Random(7)
    meadow = cover(Image.open(src / "background1.png"))
    snow = cover(Image.open(src / "background3.png"))

    vignette(grade(meadow, (0.95, 0.95, 0.9)), 0.35).save(out / "forest_day.png")

    night = grade(meadow, (0.28, 0.36, 0.62), (4, 8, 22))
    d = ImageDraw.Draw(night)
    for _ in range(90):
        x, y = rng.randrange(W), rng.randrange(int(H * 0.35))
        d.point((x, y), fill=(220, 225, 255))
    d.ellipse((760, 60, 830, 130), fill=(235, 235, 210))
    vignette(night, 0.6).save(out / "forest_night.png")

    raid = grade(meadow, (0.55, 0.22, 0.18), (30, 4, 6))
    glow = Image.new("RGB", (W, H), (255, 110, 30))
    mask = Image.new("L", (W, H), 0)
    ImageDraw.Draw(mask).rectangle((0, int(H * 0.25), W, int(H * 0.45)), fill=120)
    raid = Image.composite(glow, raid, mask.filter(ImageFilter.GaussianBlur(60)))
    d = ImageDraw.Draw(raid)
    for _ in range(420):
        x, y = rng.randrange(W), rng.randrange(H)
        d.line((x, y, x - 6, y + 22), fill=(150, 150, 175), width=1)
    for _ in range(70):
        x, y = rng.randrange(W), rng.randrange(int(H * 0.6))
        d.point((x, y), fill=(255, 200, 90))
    vignette(raid, 0.55).save(out / "village_raid.png")

    snow_night = grade(snow, (0.30, 0.36, 0.55), (6, 10, 26))
    d = ImageDraw.Draw(snow_night)
    for _ in range(260):
        x, y = rng.randrange(W), rng.randrange(H)
        r = rng.choice((1, 1, 2))
        d.ellipse((x, y, x + r, y + r), fill=(235, 240, 255))
    vignette(snow_night, 0.6).save(out / "snow_night.png")


# --------------------------------------------------------------------- title
def ridge(rng, base, amp, rough):
    pts = [base + rng.uniform(-amp, amp) for _ in range(9)]
    for _ in range(6):
        nxt = []
        for a, b in zip(pts, pts[1:]):
            nxt += [a, (a + b) / 2 + rng.uniform(-amp, amp)]
        nxt.append(pts[-1])
        pts = nxt
        amp *= rough
    return pts


def build_title():
    rng = random.Random(1907)
    img = Image.new("RGB", (W, H))
    d = ImageDraw.Draw(img)
    top, mid, low = (22, 24, 48), (96, 70, 92), (232, 168, 112)
    for y in range(H):
        t = y / H
        if t < 0.6:
            k = t / 0.6
            c = tuple(int(top[i] + (mid[i] - top[i]) * k) for i in range(3))
        else:
            k = (t - 0.6) / 0.4
            c = tuple(int(mid[i] + (low[i] - mid[i]) * k) for i in range(3))
        d.line((0, y, W, y), fill=c)
    # Pale moon with halo.
    halo = Image.new("L", (W, H), 0)
    ImageDraw.Draw(halo).ellipse((640, 70, 820, 250), fill=90)
    img = Image.composite(Image.new("RGB", (W, H), (250, 228, 190)), img, halo.filter(ImageFilter.GaussianBlur(40)))
    d = ImageDraw.Draw(img)
    d.ellipse((690, 120, 770, 200), fill=(250, 240, 214))
    layers = [
        (360, 90, (70, 64, 92)), (420, 80, (52, 50, 74)),
        (480, 70, (36, 36, 56)), (550, 50, (20, 22, 34)),
    ]
    for i, (base, amp, color) in enumerate(layers):
        pts = ridge(rng, base, amp, 0.55)
        poly = [(x * W / (len(pts) - 1), y) for x, y in zip(range(len(pts)), pts)]
        d.polygon(poly + [(W, H), (0, H)], fill=color)
        # Mist band at the foot of each layer.
        mist = Image.new("L", (W, H), 0)
        ImageDraw.Draw(mist).rectangle((0, base + 30, W, base + 90), fill=70 - i * 12)
        img = Image.composite(Image.new("RGB", (W, H), (210, 200, 210)), img, mist.filter(ImageFilter.GaussianBlur(28)))
        d = ImageDraw.Draw(img)
        if i == 0:
            # A pagoda on the far ridge.
            px = 210
            py = int(pts[int(px / W * (len(pts) - 1))]) + 4
            for level in range(4):
                w = 30 - level * 6
                y = py - level * 12
                d.polygon([(px - w, y), (px + w, y), (px + w - 6, y - 5), (px - w + 6, y - 5)], fill=(60, 56, 84))
                d.rectangle((px - w // 2, y, px + w // 2, y + 8), fill=(60, 56, 84))
        if i == len(layers) - 1:
            # Pines on the nearest ridge.
            for k in range(26):
                x = rng.randrange(W)
                y = int(pts[int(x / W * (len(pts) - 1))]) + 6
                hgt = rng.randrange(24, 52)
                d.polygon([(x, y - hgt), (x - hgt // 4, y), (x + hgt // 4, y)], fill=(12, 14, 22))
    # A few birds.
    for k in range(7):
        bx, by = 480 + k * 23 + rng.randrange(-8, 8), 210 + rng.randrange(-20, 20)
        d.line((bx - 6, by - 3, bx, by, bx + 6, by - 3), fill=(30, 28, 44), width=2)
    img = vignette(img, 0.5)
    out = ROOT / "ui" / "title.png"
    out.parent.mkdir(parents=True, exist_ok=True)
    img.save(out)


if __name__ == "__main__":
    build_objects()
    build_enemies()
    build_backgrounds()
    build_title()
    print("art generated")

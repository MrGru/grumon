# Asset Manifest

Every file under `assets/`, where it came from, its licence and what still needs to be made.
Update this file whenever an asset is added, replaced or removed.

Provenance legend:
- **Repo**: was in the Grumon repository before THIÊN MỆNH work began. The authors are the
  repository owners, and no separate licence file exists. Treat these as **placeholders to
  replace** before any public release, unless the owner confirms the rights.
- **Generated**: original art produced by `tools/gen_art.py` (Pillow, fixed seeds, no external
  inputs except where noted). Owned by the project.
- **OFL**: third-party font under the SIL Open Font License 1.1. The licence text ships next to
  the font.

## 1. Fonts

| File | Use | Source | Licence |
|------|-----|--------|---------|
| `fonts/BeVietnamPro-Regular.ttf` | Body text, dialogue, menus | Be Vietnam Pro (github.com/bettergui/BeVietnamPro) | OFL, `fonts/OFL-BeVietnamPro.txt` |
| `fonts/BeVietnamPro-SemiBold.ttf` | Names, headings, selected rows | Be Vietnam Pro | OFL, same file |
| `fonts/NotoSerifDisplay.ttf` | Title, chapter cards | Noto Serif Display (Google) | OFL, `fonts/OFL-NotoSerifDisplay.txt` |

`cargo test` (`fonts_cover_every_character`) checks that every character in every vi-VN string
and every symbol used by the UI has a glyph in these fonts. The old pixel font `grumon.ttf` had no
Vietnamese glyphs, so it was removed.

## 2. Graphics

| File(s) | Use | Provenance |
|---------|-----|------------|
| `gfx/tileset/tileset.png` | All map tiles, trees, houses | Repo |
| `gfx/characters/ow1.png` … `ow10.png` | Overworld characters (player appearances 1/3/8/2, NPCs, humanoid enemies with tint) | Repo |
| `gfx/backgrounds/background1..3.png` | Inputs for the battle backgrounds (not loaded by the game) | Repo |
| `gfx/battle/forest_day.png`, `forest_night.png`, `village_raid.png`, `snow_night.png` | Battle backgrounds | Generated: colour-graded, vignetted and overpainted from `background1/3.png`, so they inherit the Repo provenance of those inputs |
| `gfx/enemies/linh_lang.png`, `lang_dau.png`, `da_tru.png` | Beast battle sprites (32×32) | Generated (pixel art in code) |
| `gfx/objects/objects.png` | Map objects: sparkle, kite, driftwood, herb, campfire, lantern, grave, workbench, shrine, fallen body | Generated |
| `gfx/ui/title.png` | Title screen ink-wash mountains | Generated |

Regenerate with `python3 tools/gen_art.py` from the repository root.

## 3. Data (not art, listed for completeness)

`world.ldtk` (maps, placements written partly by `tools/build_ch1_map.py`), `data/*.data.ron`
(game content), `locale/vi-VN/*.locale.ron` (all player-facing text). All original to the project.

## 4. Missing assets (honest gaps)

These are required by the design (game-design.md §7–8) and **do not exist yet**:

| Need | Notes | Priority |
|------|-------|----------|
| **Audio: music** | Village day, raid, forest night, battle, boss, title. The game is currently silent | High (Milestone 2) |
| **Audio: SFX** | Menu move/confirm/cancel, typewriter blip, hits, charge stages, interrupt, formation phase, footsteps | High |
| **Dialogue portraits** | Protagonist (4 appearances × 2 expressions), ông Mạc, Tô Thanh Liên, Đồ Cuồng, Lang Nha at least | High |
| **Xianxia tileset** | The Repo tileset is a generic western village. Needs a Vietnamese/xianxia village (bamboo, tiled roofs, shrine, rice fields) | Medium |
| **Humanoid battle sprites** | Hắc Y, Đồ Cuồng and Lang Nha currently reuse tinted overworld sheets | Medium |
| **Battle VFX** | Charge rings, formation sigils, element hits (currently UI-only feedback) | Medium |
| **Ch2+ backgrounds and enemies** | Sect, secret realm, city, etc. | Later milestones |
| **Icons** | Items, artifacts, statuses (UI is text-only now) | Low |

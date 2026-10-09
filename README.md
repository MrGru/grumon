# THIÊN MỆNH: TÀN HỒN

An offline, single-player 2D top-down xianxia **JRPG** in Vietnamese, built with
[Bevy](https://bevyengine.org) 0.19. Maps are made in [LDtk](https://ldtk.io).

> Một thiếu niên thôn Thanh Khê mất tất cả trong một đêm mưa máu, mang theo khối Tàn Ngọc chứa một hồn
> phách cổ xưa, bước lên con đường nghịch thiên cải mệnh.

The game ships in **Vietnamese only** (vi-VN). Source code and developer docs are in English.

## Status

**Chapter 1 (Phàm Trần Huyết Kiếp)** can be played from start to finish. Chapters 2–8 are outlined
in the docs but not built yet. For what is done, what is missing and the risks, see
[plan.md](plan.md). It uses generated placeholder art and music
([docs/asset-manifest.md](docs/asset-manifest.md)).

## Features

- Title screen and character creation:
  - name entry accepts full Vietnamese Unicode, through a built-in Telex mode or the OS input method;
  - choose how characters address you (nam / nữ / trung tính) and one of four appearances.
- Data-driven story: branching dialogue with choices, story flags, quests with a journal and
  tracker, and side quests that can fail.
- Tactical battles:
  - a **Thân pháp** initiative timeline;
  - **Điểm hành động** and **Linh lực**, with **Tụ khí** charge stages that enemies try to interrupt;
  - telegraphed channelled attacks, push and pull;
  - **Trận pháp** formations with phases;
  - active **Pháp bảo** artifacts and Ngũ hành elements;
  - visible enemy intents.
- Pause menu: party (battle rows, order, artifacts, formation), inventory with item use, journal,
  saves and settings (battle speed, music and sound volume).
- Shops, Luyện đan (deterministic alchemy) and Luyện khí (artifact refinement).
- Generated music, sound effects and dialogue portraits.
- Saves are versioned and written atomically, with a backup copy:
  - autosave, quick save and 3 manual slots;
  - you can save anywhere, including inside a dialogue or a battle.
- Every player-facing string is validated by `cargo test`: missing keys, broken templates,
  English leakage, unaccented text and font coverage.

## Controls

| Action | Keyboard | Gamepad |
| --- | --- | --- |
| Move / choose | WASD or arrow keys | D-pad / left stick |
| Confirm, talk, interact | Space, Enter, E or Z | South (A) |
| Cancel / back | Esc, X or Backspace | East (B) |
| Pause menu | Esc | Start |
| Toggle Telex while entering a name | Tab | North (Y) |
| Quick save / quick load | F5 / F9 | – |
| World inspector / collision gizmos (debug builds) | F1 / F2 | – |
| Open forge / alchemy screen (debug builds) | F3 / F4 | – |

## Getting started

Requirements: Rust (stable, edition 2024). On Linux you also need:

```bash
sudo apt install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev pkg-config
```

Run:

```bash
cargo run                    # debug build
cargo run --features dev     # faster rebuilds via Bevy dynamic linking
cargo run --release          # optimized build
cargo test                   # unit, content-validation, balance and playthrough tests
```

Saves go to the platform data directory (`~/.local/share/thien-menh-tan-hon/saves` on Linux).
Set `THIEN_MENH_SAVE_DIR` to override it.

## Project layout

```
assets/
  world.ldtk            LDtk project: maps, NPC/trigger/object placements, warps, props
  data/*.data.ron       game content: items, skills, artifacts, formations, enemies,
                        encounters, quests, NPCs, dialogue graphs, triggers, objects
  locale/vi-VN/*.ron    every player-facing string, keyed by stable IDs
  fonts/                Be Vietnam Pro and Noto Serif Display (OFL)
  gfx/                  tileset, character sheets, generated objects/enemies/backgrounds
src/
  content/              data definitions, loaders, locale rendering, validator
  story.rs              story state: flags, inventory, quests, party, cultivation
  battle/core/          deterministic battle engine (no ECS), AI, tests
  battle/               battle session, input and UI
  dialogue.rs, flow.rs  dialogue engine and story action queue
  save.rs               save files, migrations, slots
  ...                   see AGENTS.md for the full code map
tools/
  build_ch1_map.py      places Chapter 1 entities in world.ldtk
  gen_art.py            generates the original placeholder art
docs/                   design docs (see below)
plan.md                 milestones, status, risks
```

## Documentation

- [docs/game-design.md](docs/game-design.md): pitch, pillars, architecture.
- [docs/story-bible.md](docs/story-bible.md) and [docs/chapters/](docs/chapters/): world, cast, eight-chapter plot.
- [docs/game-systems.md](docs/game-systems.md): combat and progression rules and formulas.
- [docs/content-schema.md](docs/content-schema.md): data and save formats, validation rules.
- [docs/vietnamese-style-guide.md](docs/vietnamese-style-guide.md): writing rules for vi-VN text.
- [docs/asset-manifest.md](docs/asset-manifest.md): asset sources, licences, missing assets.
- [AGENTS.md](AGENTS.md): guide for contributors and AI agents.

# plan.md — THIÊN MỆNH: TÀN HỒN

Production plan, current status and risks. A box is ticked **only** when the feature works in the
game and a test or a recorded in-game check backs it up. "Evidence" names the test or check.

Last updated: 2026-10-08.

Status legend: `[x]` done · `[~]` partly done (the note says what is missing) · `[ ]` not started.

---

## Current state in one paragraph

Milestone 1 is complete. Chapter 1 can be played from the prologue to the farewell at dawn: 5
battles, 3 side quests, about 800 Vietnamese strings. Scripted playthrough tests cover the
chapter, but only part of it has been checked by hand in the running game (see M2). The battle
engine already implements every core system (timeline, ĐHĐ/Linh lực, Tụ khí, channels, push and
pull, formations, artifacts, elements). Content counts are still below the targets (see M3). There
is no audio, there are no portraits, and the tileset is the original Grumon placeholder.

---

## Milestone 1 — Playable foundation ✅

| Task | Status | Evidence |
|------|--------|----------|
| Project setup on Bevy 0.19 + LDtk (ADR-001 in game-design.md §6.1) | [x] | `cargo build`, `cargo clippy --all-targets` clean |
| Data-driven content (RON) and a vi-VN locale with stable IDs | [x] | `content::tests::shipped_content_is_valid` |
| Interpolation: `{playerName}`, `{g:nam\|nữ\|trung tính}`, `{param}` | [x] | `content::locale::tests::*` |
| Validation: missing keys, broken templates, duplicate IDs, English leakage, unaccented prose, NFC, orphan keys, unreachable dialogue nodes, LDtk ids | [x] | `content::validate::tests::*`, `shipped_content_is_valid` |
| Fonts that cover every Vietnamese character in use | [x] | `content::tests::fonts_cover_every_character` |
| Every UI key used in code exists | [x] | `content::tests::code_keys_exist` |
| Vietnamese-only title screen | [x] | In-game screenshot (Xvfb), session log |
| New game: name entry (Unicode, built-in Telex, OS IME), addressing/gender, 4 appearances, confirmation | [x] | `telex::tests::*`, `character_creation::tests::*`, in-game: typed "Nguyễn Thị Hường" with Telex |
| Map movement, collision, warps with fades | [x] | `collision::tests::*`, in-game walk Village → Forest |
| NPC interaction (data-driven NPCs, visibility by story condition) | [x] | In-game talk, `story_tests` |
| Dialogue engine: branches, choices, conditions, effects, resumable state | [x] | `dialogue::tests::*`, in-game F5/F9 inside a dialogue |
| Combat prototype: Thân pháp timeline + ĐHĐ/Linh lực | [x] | `battle::core::tests::*` (22 tests), in-game boar battle |
| Inventory (stacking, categories, quest items) | [x] | `story::tests::items_never_go_negative`, pause menu screenshot |
| Local saving: versioned JSON, atomic write, `.bak` fallback, migrations, autosave, quick save/load, 3 manual slots | [x] | `save::tests::*`, in-game F5/F9 |
| Opening of Chapter 1 | [x] | `story_tests::chapter_1_*` |

## Milestone 2 — Fully playable Chapter 1 🟡

| Task | Status | Notes / evidence |
|------|--------|------------------|
| Village day sequence: festival prep, herbs, Liên's bridge talk, side quests (kite, fish porridge, lotus hairpin) | [x] | `story_tests::chapter_1_full_playthrough_with_side_quests` |
| Character bonds (`trust.*` flags change with choices) | [~] | Flags are set and stored; nothing reads them yet (they pay off in Ch2+) |
| Massacre: raid, saving bé Đậu, ông Mạc's last stand | [x] | `story_tests`, `showcase_3_night_raid_two_approaches` |
| Escape: night forest, wolves, shrine awakening | [x] | `story_tests`, `showcase_1_*` |
| Jade pendant (Tàn Ngọc) and first cultivation (Luyện Khí) | [x] | `story_tests`, `story::tests::tu_vi_advances_stages_but_not_mortals` |
| Tutorial hints in battles | [x] | Data `hints`, seen in-game in the boar battle |
| Quests with journal and tracker | [x] | `story::tests::quest_completes_and_chains`, HUD screenshot |
| Boss: Lang Nha | [x] | `showcase_2_lang_nha_two_approaches` |
| Every story branch reaches the end of the chapter | [x] | Two scripted paths plus side-quest failure tests. Not exhaustive over all choice combinations |
| **Hand-checked in the running game: raid formation battle, Lang Nha, chapter end** | [ ] | Covered only by simulations and story tests so far |
| **Save/load tested in-game during combat and at chapter transitions** | [~] | `save_and_load_mid_battle_is_identical` (engine) and in-game save inside dialogue. A save made mid-battle has not been loaded in the running game |
| Audio (music + SFX) | [ ] | No audio exists. See asset-manifest.md §4 |
| Visual presentation: portraits, xianxia tileset, battle VFX | [ ] | Generated placeholder art only |
| Item use outside battle (pause menu) | [ ] | Items can only be used in battle |

Acceptance: a new player finishes Ch1 without help, every in-game save/load case passes, and
audio and portraits are present.

## Milestone 3 — Core cultivation gameplay 🟡

| Task | Status | Notes |
|------|--------|-------|
| Tụ khí stages 1–3 + Quá Tụ, breaking, Hộ tâm/Thủ thế protection | [x] | `charge_*`, `interrupter_breaks_charge_unless_warded`, `overcharge_costs_hp_and_backlashes` |
| Channelled techniques, interrupts, push/pull with cap, stun immunity, no triple turns | [x] | core tests |
| Formations: energy, phases, aura/pulse, node breaking, release | [x] engine / [~] content | 4 of 6 formations in data (`luong_nghi_tran`, `cuu_cung_me_tran` missing) |
| Active artifacts with passives and limits | [x] engine / [x] content | 12 in data; the 13th (`chieu_hon_phien`) needs summons ⏳ |
| Charge techniques | [~] | 3 of 8 (`pha_thach_quyen`, `nghich_menh_chi`, `kiem_phoi_tram`) |
| Enemy archetypes that react to the systems | [~] | 4 of 6+ in data (assassin, channeller, interrupter, formation breaker) |
| Three showcase battles, each with two viable approaches, and tests | [x] | `battle::balance_tests::showcase_*` |
| Breakthrough events (major realms) | [~] | `SetRealm` effect and Ch1 awakening work; no breakthrough scene beyond Luyện Khí yet |
| Party management UI (order, slots, formation choice) | [ ] | The party tab is view-only |
| Equipment UI (artifact slots by realm) | [ ] | Equipping is done only through story effects |
| Alchemy (Luyện đan), artifact refinement (Luyện khí), shops | [ ] | Designed in game-systems.md §10 |
| Balanced progression across chapters | [ ] | Only Ch1 is balanced |

## Milestone 4 — Chapters 2–4 ⏳

| Task | Status |
|------|--------|
| Ch2 Ngoại Môn Phong Vân: sect life, entrance trial, companions | [ ] (event-level outline in `docs/chapters/ch2-*.md`) |
| Ch3 Huyền Cốc Bí Cảnh: secret realm, puzzles, Trúc Cơ | [ ] (outline) |
| Ch4 Ma Ảnh Loạn Thành: investigation, auction | [ ] (outline) |
| Companion arcs and consequential choices (`trust.*`, `rep.*`) | [ ] |

## Milestone 5 — Chapters 5–8 and endings ⏳

| Task | Status |
|------|--------|
| Ch5–Ch8 content | [ ] (outlines in `docs/chapters/`, story bible complete) |
| Three endings driven by `stat.tam_ma`, `stat.nhan_tam` and key choices | [ ] (designed in story-bible.md) |

## Milestone 6 — Production polish ⏳

| Task | Status | Notes |
|------|--------|-------|
| Content validation in CI | [~] | Runs in `cargo test`; no CI workflow yet |
| Quest dependency checks (every quest is startable and completable) | [~] | Ch1 only, via scripted playthroughs |
| Save migrations | [~] | Framework and a v0 → v1 test exist |
| Accessibility: text speed, fast battle, font scaling, colour-blind-safe cues | [~] | Fast battle setting only |
| Rebindable controls | [ ] | |
| Performance pass, release packaging, offline QA | [ ] | The game makes no network calls, but no formal offline QA run has been done |

---

## Dependencies

- M2's audio and portraits block the "presentation" acceptance, not gameplay.
- Party and equipment UI (M3) must land before Ch2, where a second permanent companion joins.
- Remaining formations, charge techniques and archetypes (M3) are needed by the Ch2–Ch3 encounters.
- An xianxia tileset is needed before building new Ch2 maps, to avoid rebuilding them.

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| No artist or composer: generated placeholder art, no audio | High: presentation quality | asset-manifest.md lists every gap; `tools/gen_art.py` keeps placeholders reproducible |
| Repo tileset and character sheets have no licence file | Release blocker | Confirm the rights with the repo owner or replace them (asset-manifest.md) |
| Scope: 8 chapters of handcrafted content | High | Polish one chapter at a time (prompt §16); data-driven content and validators keep the cost per chapter down |
| Battle balance drifts as content grows | Medium | Policy simulations in `cargo test`; add per-chapter showcase tests |
| Bevy and plugin API churn | Medium | Pinned versions; pure logic is kept free of ECS (most tests do not touch Bevy) |
| Vietnamese text quality (tone, register, typos) | Medium | vietnamese-style-guide.md; validator catches leakage and unaccented text but not style. A native-speaker review is needed |
| Story branch explosion | Medium | Flags are namespaced; scripted path tests per chapter |

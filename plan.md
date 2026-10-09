# plan.md — THIÊN MỆNH: TÀN HỒN

Production plan, current status and risks. A box is ticked **only** when the feature works in the
game and a test or a recorded in-game check backs it up. "Evidence" names the test or check.

Last updated: 2026-10-08.

Status legend: `[x]` done · `[~]` partly done (the note says what is missing) · `[ ]` not started.

---

## Current state in one paragraph

Milestone 1 is complete. Chapter 1 can be played from the prologue to the farewell at dawn: 5
battles, 3 side quests, about 850 Vietnamese strings, generated music, sound effects and dialogue
portraits. Scripted playthrough tests cover the whole chapter, and the chapter end, every battle
type and saving inside dialogue and battle have been checked by hand in the running game. The battle
engine implements every core system and the Milestone 3 content targets for combat are met
(6 formations, 13 artifacts, 8 Tụ khí techniques, 6 enemy archetypes). Party management, equipment
and item use work from the pause menu. Still missing: shops, alchemy and refinement (M3), Chapters
2–8 (only Chapter 2's battle content exists), a listening pass on the audio, and a xianxia tileset
to replace the original Grumon placeholder.

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
| Raid formation battle and Lang Nha battle checked in the running game | [~] | Loaded from QA saves (`story_tests::export_qa_saves`): opening turns played, shield, guard, Tụ khí stages 1–2 and a stage-2 *Phá Thạch Quyền* release work. Neither fight was played to the end by hand |
| Battle ends and returns to the map, autosave written | [x] | In-game: boar battle won from a QA save, back on the Forest map, `auto.json` has `ch1.da_tru_defeated` |
| Chapter end (dawn, farewell) checked by hand | [x] | In-game from a QA save: grave, token, medicine jar, thím Ba, Lý Đức's farewell, "Hết Chương 1" card, autosave with all five main quests Done. This check found the raid-entry objective could stall the quest chain if its trigger was skipped; fixed and now asserted by both playthrough tests |
| Save/load in-game during dialogue and combat | [x] | F5 inside a dialogue and inside the raid battle; F9 and the title load screen resume the exact node / battle state |
| Save/load across a chapter transition | [ ] | There is only one chapter so far |
| Audio (music + SFX) | [~] | 8 generated loops and 17 sound effects (`tools/gen_audio.py`), data-driven music per map/time/battle/story, volume settings. Verified in game through logs with a null audio device (`audio::tests`, music log); **not yet listened to by a human** |
| Dialogue portraits | [x] | 20 generated portraits (`tools/gen_portraits.py`), shown left of the dialogue text; `content::tests::every_speaker_has_a_portrait`. One expression each |
| Visual presentation: xianxia tileset, burned village at dawn, battle VFX | [ ] | Generated placeholder art; the village looks untouched after the raid (only a screen tint) |
| Item use outside battle (pause menu) | [x] | `party::tests::*`, playthrough uses the Tụ Khí Đan from ông Mạc's jar; in-game: pill used from Túi đồ, tu vi 0 → 40. Healing items stay battle-only because every battle starts at full Khí huyết (game-systems §8.6) |

Acceptance: a new player finishes Ch1 without help, every in-game save/load case passes, and
audio and portraits are present.

## Milestone 3 — Core cultivation gameplay 🟡

| Task | Status | Notes |
|------|--------|-------|
| Tụ khí stages 1–3 + Quá Tụ, breaking, Hộ tâm/Thủ thế protection | [x] | `charge_*`, `interrupter_breaks_charge_unless_warded`, `overcharge_costs_hp_and_backlashes` |
| Channelled techniques, interrupts, push/pull with cap, stun immunity, no triple turns | [x] | core tests |
| Formations: energy, phases, aura/pulse, node breaking, release | [x] | 6 of 6 in data; `FreeSwap` aura and `StatusRow` pulse for Lưỡng Nghi / Cửu Cung (`free_swap_aura_and_row_release`) |
| Active artifacts with passives and limits | [x] | 13 in data; Chiêu Hồn Phiên summons a taunting spirit (`summon_taunts_then_fades_and_never_decides_the_battle`, `ch2_spirit_draws_the_blows`) |
| Charge techniques | [x] (content) | 8 in data: 3 for the hero, 5 for the Ch2+ companions (defined in data, not yet joinable in story) |
| Enemy archetypes that react to the systems | [x] 6 / [ ] 7th | Assassin, channeller, interrupter, formation breaker (Ch1) + guardian and drainer (Ch2 data, simulated). The illusionist comes with Ch4 |
| Chapter 2 battle content | [~] | `ch2.data.ron`: Mộc Nhân, guardian, drainer, Âu Dương Liệt, assassin and six encounters, simulated in `ch2_battles_reward_the_new_lessons`; not placed on maps yet |
| Three showcase battles, each with two viable approaches, and tests | [x] | `battle::balance_tests::showcase_*` |
| Breakthrough events (major realms) | [~] | Mechanic done (`Breakthrough` item effect, `party::tests::breakthrough_needs_the_peak`); no breakthrough item or scene exists in content until Ch3 |
| Party management UI (order, slots, formation choice) | [x] | `party::tests::*`; in-game: row change refused for the last front-row fighter, formation chosen from learned ones. Order swapping is tested in code only (Ch1 has a party of one) |
| Equipment UI (artifact slots by realm) | [x] | `party::tests::equipping_respects_slots_and_binding`; in-game: gourd unequipped to the bag and re-equipped |
| Shops | [x] | `economy::tests::buying_respects_money_and_stock`, `selling_pays_half_and_refuses_quest_items`; in-game: bought a cake and sold a fang at thím Ba's stall |
| Alchemy (Luyện đan) | [x] | `brewing_is_deterministic_and_improves_with_practice`, full playthrough brews at ông Mạc's stove; in-game: brewed Thuốc Trị Thương |
| Artifact refinement (Luyện khí) | [x] rules / [~] content | `refining_steps_in_order_until_the_last`, `refinement_levels_reach_the_battle`; in-game screen via debug F3. No Ch1 station: it opens with the sect in Ch2 |
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
- Ch2 encounters exist in data and simulation; they need maps, a Ch2 battle background set, and a re-balance against Ch2's real progression.
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

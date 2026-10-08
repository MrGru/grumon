# THIÊN MỆNH: TÀN HỒN — Game Design Document

> Internal document (English). Player-facing text is Vietnamese only; names and terms below are the
> canonical Vietnamese forms from [story-bible.md](story-bible.md) and
> [vietnamese-style-guide.md](vietnamese-style-guide.md).

## 1. Pitch

A player-named mortal with no spiritual root survives the massacre of their village, inherits a
cracked jade pendant holding the remnant soul of an ancient sage, and cultivates a forbidden path
(**Nghịch Mệnh Quyết**) through a cultivation world whose Heavenly Dao turns out to be a machine
that feeds on souls. Eight chapters, 20–30 hours, three endings. Fully offline, Vietnamese-first.

## 2. Pillars

1. **A real JRPG, not a visual novel.** Walk maps, talk to people, explore, fight, grow, craft.
2. **Tactical combat with decisions every turn.** Timeline (Thân pháp), Điểm hành động, Linh lực,
   Tụ khí, Trận pháp and active Pháp bảo. Never a fixed “everyone attacks once” loop.
3. **Choices with visible consequences**, kept to a maintainable number of meaningful branches.
4. **Vietnamese prose of literary quality**, consistent terminology and correct forms of address.
5. **Respect the player's time:** no mandatory grinding, short battles, readable UI.

## 3. Player experience by chapter

| Ch. | Title | Hours | New mechanics | Key choice |
|-----|-------|-------|---------------|------------|
| 1 | Phàm Trần Huyết Kiếp | 1.5–2 | Movement, dialogue, inventory, timeline, AP, Linh lực, guard, push; Tụ khí; first formation (guest) | Promise to Liên; save bé Đậu; trust Ngọc lão |
| 2 | Ngoại Môn Phong Vân | 3 | Sect chores, alchemy, advanced Tụ khí, tournament | Spare or humiliate Âu Dương Liệt |
| 3 | Huyền Cốc Bí Cảnh | 3 | Puzzles, party of 3, Trận pháp proper, Bản Mệnh Kiếm Phôi | Kill or spare Lưu Thành |
| 4 | Ma Ảnh Loạn Thành | 3.5 | Investigation (clue book), auction, artifact resonance | Expose vs deal; victims vs ringleader |
| 5 | Kim Đan Vấn Tâm | 3 | Inner-demon trial, artifact forging, tribulation battle | Golden lotus vs Bạch Thủy trấn |
| 6 | Cửu Vực Đại Kiếp | 4 | Campaign map, faction support, advanced enemy counters | Who to defend; Liên |
| 7 | Thiên Môn Vô Lộ | 3 | Celestial puzzles, soul sea | Forgive / blame Hư Minh; Văn Trọng Khanh |
| 8 | Nghịch Thiên Cải Mệnh | 2.5 | Multi-phase finale | Three endings |

## 4. Core loop

Explore map → talk / find objects / trigger events → fixed encounters (no random battles) →
rewards (tu vi, items, story flags) → progress quests → story beats and choices → new maps.

Encounters are placed by hand (LDtk `Trigger` zones or NPC dialogue effects), so pacing is authored.
Repeatable training fights exist in towns for players who want them, but are never required.

## 5. Systems overview

Detailed rules with formulas: [game-systems.md](game-systems.md).

- **Combat:** CTB timeline driven by Thân pháp with diminishing returns; 3 AP per activation with
  1 AP carry; Linh lực regen; Tụ khí stages 1–3 (4 with Quá Tụ); channelled enemy attacks as
  telegraphs; interrupts; push/pull; formations with phases and node breaking; artifacts with
  cooldowns and charges; Ngũ hành khắc and Tương sinh combos; encounter objectives beyond
  “defeat all”.
- **Cultivation:** six realms × four stages; tu vi from authored sources; story-gated breakthroughs.
- **Inventory & equipment:** stackable items, quest items, artifact slots by realm.
- **Quests:** data-driven objectives evaluated against story state; journal UI.
- **Dialogue:** branching graph with conditions and effects, `{playerName}` and gendered addressing.
- **Saves:** 3 manual slots, autosave, quick save; works during exploration, dialogue and battle.

## 6. Architecture

### 6.1 Decision: keep Bevy 0.19 + LDtk (ADR-001)

The repository already contained a working Bevy 0.19 top-down prototype (movement, collision, LDtk
maps, dialogue box, warps, y-sorting, camera). Bevy is fully offline, data-driven friendly and
supports Unicode text with any TTF font. We extend it rather than switching engines.

### 6.2 Layers

```
            ┌────────────────────────────────────────────────────────────────┐
 Content    │ assets/data/*.data.ron   assets/locale/vi-VN/*.locale.ron  LDtk │
            └──────────────┬─────────────────────────────────────────────────┘
                           │ loaded once (bevy_asset_loader), validated by tests
            ┌──────────────▼─────────────────────────────────────────────────┐
 Pure logic │ content::{defs, db, locale, condition}   telex   battle::core   │
 (no ECS,   │ story::Progress (flags, inventory, quests, party)   save        │
 unit tests)│ dialogue::runner   progression                                   │
            └──────────────┬─────────────────────────────────────────────────┘
                           │ thin systems
            ┌──────────────▼─────────────────────────────────────────────────┐
 Bevy glue  │ plugins: title, character_creation, level, player, npc,         │
            │ map_events, dialogue (UI), battle (UI), hud, pause_menu, save,  │
            │ screen_fx, transition, camera, input                             │
            └────────────────────────────────────────────────────────────────┘
```

### 6.3 States

```
GameState:  Loading ─► Title ─► CharacterCreation ─► Playing
                         ▲                              │
                         └───────── (return to title) ──┘
PlayState (sub-state of Playing):
  Exploring ◄─► Dialogue      Exploring ◄─► Transition
  Exploring ◄─► Battle        Exploring ◄─► Paused (pause menu)
  any ─► Card (full-screen chapter/title card) ─► Exploring
```

### 6.4 Effects pipeline

Dialogue options, quest completion, triggers, objects and battle results all emit the same
`StoryEffect` list. `story::apply_effect` mutates `Progress` (pure) and returns *deferred* actions
(start dialogue, start battle, warp, show card, autosave) that the ECS layer executes in order.
This makes content logic testable without running Bevy.

### 6.5 Determinism

Battle core uses integer math and a serialisable SplitMix64 RNG. Given the same `BattleState` and
inputs the outcome is identical, which makes saving mid-battle and automated balance simulations
possible.

### 6.6 Input

Keyboard (WASD/arrows, Space/Enter/E/Z confirm, Esc/X/Backspace cancel, Tab speed toggle,
F5 quick save, F9 quick load) and gamepad (D-pad/left stick, South confirm, East cancel, Start
menu) are mapped in `input.rs` into one `MenuInput` resource. Mouse works on title and creation
buttons. Touch is out of scope for the desktop build.

### 6.7 Offline guarantee

No networking crates; all assets live under `assets/`; fonts are bundled (OFL). The game never
opens a socket. Verified by `cargo tree` review and by running with networking disabled.

## 7. UI and accessibility

- Screen-space UI at 960×640, font sizes ≥ 18 px for body text.
- Battle screen always shows: timeline (next 8 entries with channel markers), every unit's Khí
  huyết / Linh lực bars, Tụ khí pips, statuses, enemy **Ý đồ**, active unit's ĐHĐ, formation phase
  and energy, artifact cooldowns, and the reason an action is unavailable.
- Battle speed ×1/×2 (Tab). Text speed fixed fast with instant-complete on confirm.
- Colour is never the only signal (statuses have text labels).

## 8. Audio direction (Milestone 2+)

Guzheng/dizi-led themes: Thanh Khê (warm pentatonic), raid (taiko + low strings), shrine (bell
drone), battle (fast pipa ostinato), boss (erhu + percussion). See
[asset-manifest.md](asset-manifest.md).

## 9. Scope targets (production, not filler quotas)

~40 main quests, 30 side quests, 15 significant NPCs, 5 companions, 40 enemy variants, 12 bosses,
20 maps, 60 techniques, 12+ artifacts, 6+ formations, 3 endings. Progress is tracked in
[../plan.md](../plan.md).

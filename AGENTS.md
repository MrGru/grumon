# AGENTS.md

Guide for AI coding agents (Claude Code, Codex, Cursor, ...) and human contributors working on
**THIÊN MỆNH: TÀN HỒN**. Read this before changing code.

Other docs:
- Project overview: [README.md](README.md).
- Status and next tasks: [plan.md](plan.md).
- Rules and formulas: [docs/game-systems.md](docs/game-systems.md).
- Data formats: [docs/content-schema.md](docs/content-schema.md).
- Writing rules: [docs/vietnamese-style-guide.md](docs/vietnamese-style-guide.md).
- Story: [docs/story-bible.md](docs/story-bible.md).
- JRPG design rules: [docs/JRPG_BEST_PRACTICES.md](docs/JRPG_BEST_PRACTICES.md).

## Project at a glance

- 2D top-down xianxia JRPG written in Rust with **Bevy 0.19** (ECS game engine). Rust edition 2024.
- **Vietnamese-only** game text. Every player-facing string lives in
  `assets/locale/vi-VN/*.locale.ron`. English is fine in code and dev docs, **never on screen**.
- Game content (items, skills, enemies, dialogue graphs, quests…) is RON in `assets/data/`.
  Maps are authored in **LDtk** (`assets/world.ldtk`) and loaded with `bevy_ecs_ldtk 0.15`.
- Assets are preloaded with `bevy_asset_loader 0.27`. The debug inspector is
  `bevy-inspector-egui 0.37`. Data uses `serde` + `ron`; saves use `serde_json`.

## Commands

```bash
cargo run                      # play (debug build, inspector available)
cargo run --features dev       # faster incremental builds (Bevy dynamic linking)
cargo test                     # all tests: logic, content validation, battle balance, Ch1 playthroughs
cargo test balance -- --nocapture   # print battle simulation win rates
cargo clippy --all-targets     # lint; keep it warning-free
cargo fmt                      # format before committing
cargo build --release          # size-optimized release build
python3 tools/gen_art.py       # regenerate generated art (needs Pillow)
python3 tools/build_ch1_map.py # re-place Chapter 1 entities in world.ldtk (idempotent)
```

Linux needs system libraries: `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev pkg-config`.
The first build compiles all of Bevy and takes several minutes. Run `cargo check` for quick feedback.
Set `THIEN_MENH_SAVE_DIR` to keep test saves out of your real save directory.

## Code map

| Path | Responsibility |
| --- | --- |
| `src/main.rs` | Window setup, adds `GamePlugin` |
| `src/lib.rs` | `GameState` / `PlayState`, global constants, plugin registration |
| `src/asset.rs` | `GameAssets` collection (sheets, fonts, data and locale files, battle art) |
| `src/content/` | `defs.rs` data types, `db.rs` `GameDb`, `locale.rs` templates and rendering, `validate.rs` validator, `mod.rs` loaders, the `Content` resource and content tests |
| `src/story.rs` | `Progress`: flags, inventory, quests, party, cultivation. Applies `StoryEffect`s and evaluates `Condition`s (pure, no ECS) |
| `src/flow.rs` | New game / load bridge, `ActionQueue` for deferred story actions, the `Story` system param, play time, F5/F9 |
| `src/save.rs` | Save files (versioned JSON, atomic write, `.bak` fallback, migrations), slots |
| `src/dialogue.rs` | Dialogue graph walker (`DialogueHost`), dialogue box UI with choices |
| `src/battle/core/` | Deterministic battle engine: `state.rs`, `engine.rs`, `ai.rs`, `rng.rs`, `tests.rs` (no ECS) |
| `src/battle/` | `mod.rs` battle session and input, `view.rs` UI, `text.rs` log/names, `balance_tests.rs` simulations |
| `src/character_creation.rs` | Name entry (Telex/IME), addressing, appearance, confirmation |
| `src/telex.rs` | Built-in Telex input engine |
| `src/main_menu.rs` | Title screen and load view |
| `src/pause_menu.rs` | Pause menu tabs (party, inventory, journal, save, load, settings), `Settings` |
| `src/hud.rs` | Notices, quest tracker, map banner, controls hint, chapter/story cards |
| `src/screen_fx.rs` | Time-of-day tint, raid rain, `fx.blackout` overlay |
| `src/ui.rs` | Palette, fonts, panels, menu rows (shared UI helpers) |
| `src/input.rs` | `MenuInput`: keyboard + gamepad mapped to actions |
| `src/level.rs` | LDtk world, `LevelInfo` (wall grid), props (trees, houses), `Warp` entities |
| `src/map_events.rs` | LDtk `Trigger` zones and `Object` interactables, interaction with NPCs/objects |
| `src/npc.rs` | `Npc` entities from LDtk (`id` → `NpcDef`), visibility by condition |
| `src/player.rs` | Player spawn, movement, facing, position recorded into `Progress` |
| `src/collision.rs` | `Collider` component, `CollisionWorld::move_and_slide` (AABB, per-axis) |
| `src/transition.rs` | Fading between maps (`PendingWarp`) |
| `src/camera.rs` | 2x zoom camera following the player, clamped to the level |
| `src/ysort.rs` | Depth sorting by feet position |
| `src/animation.rs` | Sprite-sheet animation, `Facing` |
| `src/story_tests.rs` | Scripted Chapter 1 playthroughs (test only) |
| `src/debug.rs` | Debug builds only: F1 inspector, F2 collision gizmos |
| `tools/` | `build_ch1_map.py` (LDtk placements), `gen_art.py` (art generator) |

## State machine

```
GameState:  Loading ──► Menu ──► CharacterCreation ──► Starting ──► Playing
                         ▲  └────────── (load save) ────────┘          │
                         └──────────────── (to title) ─────────────────┘
PlayState (sub-state of Playing, starts in Transition):
  Transition ──► Exploring ◄──► Dialogue | Battle | Paused | Card
```

Gate every gameplay system with `run_if(in_state(...))`. The player only moves in
`PlayState::Exploring`. Story actions (start a dialogue, battle, warp, card, autosave) are queued
in `flow::ActionQueue` and run one at a time while exploring.

## Conventions

- **One plugin per feature.** A new feature gets its own module with a `Plugin`, added in `GamePlugin`.
- **Data lives in data files, not in code.**
  - NPC definitions, dialogue, quests, encounters, triggers and objects live in `assets/data/*.data.ron`.
  - Placements live in LDtk (an entity with an `id` field).
  - Text lives in the locale.
  - Don't hard-code map content or strings in Rust.
- **Text:** never write a Vietnamese string in Rust. Add a key to `ui.locale.ron` (or the matching
  file) and use `content.ui("ui.…")` / `content.ui_format(...)`. Content keys are derived from IDs
  (`item.<id>.name`, `dlg.<dialogue>.<node>`…, see content-schema.md). Use only characters the
  fonts cover; `cargo test` checks this.
- **Pure logic stays pure:** content, story, telex and `battle::core` don't touch the ECS, so they
  can be unit tested. The battle core is deterministic (integer math, seeded RNG) and serialisable.
- **Coordinates:** LDtk is y-down with the origin at the top-left; Bevy is y-up. Convert with
  `LevelInfo::ldtk_to_world`. Levels spawn at translation zero.
- **Collision:** anything with a `Collider` blocks the player. Wall cells come from the `IntGrid`
  layer (value `1 = Wall`). Colliders cover the *feet*, not the whole sprite.
- **Depth:** give every world sprite a `YSort` with the correct `foot` offset. Don't set `translation.z` by hand.
- **Bevy 0.19 idioms:**
  - `Message` / `MessageReader` / `MessageWriter` for buffered events;
  - `Single<..>` and `query.single()` returning `Result`;
  - `ChildOf`, `children![]`, `despawn()` (already recursive), `DespawnOnExit`;
  - required components instead of bundles for new code.
- Systems that can fail return `Result`. Log and recover instead of `unwrap()`/`panic!` in gameplay code.
- Add tests next to the code in `#[cfg(test)] mod tests`. Battle content changes must keep
  `battle::balance_tests` green, and story changes must keep `story_tests` green.
- Match the surrounding style: small systems, doc comments on public items, constants for tuning values.
- Update `plan.md` checkboxes only when the feature works in the game and a test backs it up.

## Editing maps (`assets/world.ldtk`)

Prefer the [LDtk editor](https://ldtk.io) (version 1.5.x). For bulk Chapter 1 placements, edit the
tables in `tools/build_ch1_map.py` and re-run it. If you must edit the JSON by hand or with a script:

- Keep every `uid` unique and bump the root `nextUid`.
- Each entity instance needs a fresh `iid` (UUID), `__grid`, `px`, `__worldX/__worldY` and a
  `fieldInstances` entry per field, including `realEditorValues`.
- IntGrid `intGridCsv` length must equal `__cWid * __cHei`.
- After changing the file, run `cargo test` (it checks that every LDtk `id` exists in data), run
  the game and press **F2** to check walls and colliders.

Entity types:

| Entity | Fields | Notes |
| --- | --- | --- |
| `Player` | – | Fallback start position, only in `Village` (chapters set `start_feet`) |
| `Npc` | `id: String` | References an `NpcDef` in data (character, sheet, `visible` condition, `talk` entries choosing a dialogue by condition). Several placements can share a character with different conditions (e.g. day vs raid) |
| `Trigger` | `id: String` | Resizable zone. References a `TriggerDef`. Fires when the player's feet enter it |
| `Object` | `id: String` | Point. References an `ObjectDef` (sprite, solid, `once`, effects). Interact with confirm |
| `Warp` | `to_level: String`, `to_x: Int`, `to_y: Int` | Resizable zone. Target is where the player's **feet** land, in LDtk pixels of the target level. Put it at least 2 tiles away from the target level's warp, or the player bounces straight back |
| `Tree`, `Pine`, `SnowTree`, `Palm` | – | Trunk collider only |
| `House`, `HouseSmall`, `HouseSmallPurple`, `HouseBig` | – | Collider on the lower 60% |

Levels: `Village` (Thanh Khê thôn, start), `Forest` (west of the village), `Snowfield` (north of the
forest), `Beach` (south of the village).

## Before you finish

1. `cargo fmt && cargo clippy --all-targets && cargo test` pass.
2. The game starts, and you can create a character, walk, talk (Space/E/Enter), fight and change maps.
3. Update this file, the README and `plan.md` if you add a module, entity type, control or feature.

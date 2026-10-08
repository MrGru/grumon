# AGENTS.md

Guide for AI coding agents (Claude Code, Codex, Cursor, ...) and human contributors working on **Grumon**.
Read this before changing code. Project overview: [README.md](README.md).
Design rules: [docs/JRPG_BEST_PRACTICES.md](docs/JRPG_BEST_PRACTICES.md).

## Project at a glance

- 2D top-down JRPG written in Rust with **Bevy 0.19** (ECS game engine).
- Maps are authored in **LDtk** (`assets/world.ldtk`) and loaded with `bevy_ecs_ldtk 0.15`.
- Assets are preloaded with `bevy_asset_loader 0.27`; debug inspector via `bevy-inspector-egui 0.37`.
- Rust edition 2024.

## Commands

```bash
cargo run                      # play (debug build, inspector available)
cargo run --features dev       # faster incremental builds (Bevy dynamic linking)
cargo test                     # unit tests (pure logic: collision, dialogue, level grid, camera)
cargo clippy --all-targets     # lint; keep it warning-free
cargo fmt                      # format before committing
cargo build --release          # size-optimized release build
```

Linux needs system libraries: `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev pkg-config`.

The first build compiles all of Bevy and takes several minutes. Run `cargo check` for quick feedback.

## Code map

| File | Responsibility |
| --- | --- |
| `src/main.rs` | Window setup, adds `GamePlugin` |
| `src/lib.rs` | `GameState` / `PlayState`, global constants, plugin registration |
| `src/asset.rs` | `GameAssets` collection (character sheets, font) |
| `src/level.rs` | LDtk world, `LevelInfo` (wall grid), props (trees, houses), `Warp` entities |
| `src/collision.rs` | `Collider` component, `CollisionWorld::move_and_slide` (AABB, per-axis) |
| `src/player.rs` | Player spawn (from the LDtk `Player` marker), movement, facing |
| `src/npc.rs` | `Npc` entities from LDtk, talking to them |
| `src/dialogue.rs` | Dialogue box UI, typewriter text |
| `src/transition.rs` | Fading between maps when the player steps on a `Warp` |
| `src/camera.rs` | 2x zoom camera following the player, clamped to the level |
| `src/ysort.rs` | Depth sorting by feet position |
| `src/animation.rs` | Sprite-sheet animation, `Facing` |
| `src/main_menu/` | Title screen |
| `src/debug.rs` | Debug builds only: F1 inspector, F2 collision gizmos |

## State machine

```
GameState:  Loading ──► Menu ──► Playing
PlayState (sub-state of Playing):  Exploring ◄──► Dialogue
                                   Exploring ◄──► Transition (map change)
```

Gate every gameplay system with `run_if(in_state(...))`. The player must only move in `PlayState::Exploring`.

## Conventions

- **One plugin per feature.** A new feature gets its own module with a `Plugin`, added in `GamePlugin`.
- **Data lives in LDtk, not in code.** NPC names, sprites and dialogue lines, warp targets and props are LDtk entities with fields. Don't hard-code map content in Rust.
- **Coordinates:** LDtk is y-down with the origin at the top-left; Bevy is y-up. Convert with `LevelInfo::ldtk_to_world`. Levels spawn at translation zero, so a level's local coordinates are world coordinates.
- **Collision:** anything with a `Collider` blocks the player. Wall cells come from the `IntGrid` layer (value `1 = Wall`). Colliders cover the *feet*, not the whole sprite.
- **Depth:** give every world sprite a `YSort` with the correct `foot` offset. Don't set `translation.z` by hand.
- **Bevy 0.19 idioms:** `Message`/`MessageReader`/`MessageWriter` for buffered events, `Single<..>` / `query.single()` returning `Result`, `ChildOf`, `children![]`, `despawn()` (already recursive), required components instead of bundles for new code.
- Systems that can fail return `Result`; log and recover instead of `unwrap()`/`panic!` in gameplay code.
- Keep pure logic (collision math, dialogue state, grid conversion) free of ECS so it can be unit tested. Add tests next to the code in `#[cfg(test)] mod tests`.
- Match the surrounding style: small systems, doc comments on public items, constants for tuning values.

## Editing maps (`assets/world.ldtk`)

Prefer the [LDtk editor](https://ldtk.io) (version 1.5.x). If you must edit the JSON by hand or with a script:

- Keep every `uid` unique and bump the root `nextUid`.
- Each entity instance needs a fresh `iid` (UUID), `__grid`, `px`, `__worldX/__worldY` and a `fieldInstances` entry per field, including `realEditorValues`.
- IntGrid `intGridCsv` length must equal `__cWid * __cHei`.
- After changing the file, run the game and press **F2** to check walls and colliders.

Entity types:

| Entity | Fields | Notes |
| --- | --- | --- |
| `Player` | – | Start position, only in `Village` |
| `Npc` | `name: String`, `sprite: Int` (1–10, `ow{n}.png`), `dialogue: Array<String>` | Lines must be ASCII/Latin-1: `grumon.ttf` has no Vietnamese glyphs |
| `Warp` | `to_level: String`, `to_x: Int`, `to_y: Int` | Resizable zone. Target is where the player's **feet** land, in LDtk pixels of the target level. Put it at least 2 tiles away from the target level's warp, or the player bounces straight back |
| `Tree`, `Pine`, `SnowTree`, `Palm` | – | Trunk collider only |
| `House` | – | Collider on the lower 60% |

Levels: `Village` (start), `Forest` (west of the village), `Snowfield` (north of the forest), `Beach` (south of the village).

## Before you finish

1. `cargo fmt && cargo clippy --all-targets && cargo test` pass.
2. The game starts, you can walk, talk (Space/E/Enter) and change maps.
3. Update this file and the README if you add a module, entity type or control.

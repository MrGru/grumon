# JRPG Best Practices for Grumon

Design and engineering guidelines for building a top-down JRPG with Bevy + LDtk.
They describe how Grumon is built today and how new features should be added.

---

## 1. Architecture

### States drive everything
A JRPG is a stack of modes: title, field exploration, dialogue, menus, battle, cutscenes.
Model them as Bevy states, not booleans.

- `GameState` holds the application flow (`Loading → Menu → Playing`).
- `PlayState` is a sub-state of `Playing` (`Exploring`, `Dialogue`, `Transition`). Add new modes here (`Battle`, `PauseMenu`, `Cutscene`).
- Every system has a `run_if(in_state(...))`. Then "the player walked while the shop menu was open" can't happen.
- Use `OnEnter` / `OnExit` to spawn and despawn mode-specific UI and resources.

### One feature, one plugin
Each module exposes a `Plugin` and owns its components, resources and systems.
Plugins talk to each other through components, resources, messages and system sets, not through each other's internals.

### Data-driven content
- Maps, NPCs, dialogue, warps and props are **data** (LDtk entities and fields). Designers can change content without recompiling.
- Upcoming content (items, monsters, skills, shops, quests) should go in data files (RON/JSON loaded as assets) with typed Rust structs. Don't build giant `match` blocks in code.
- Reference content by stable string IDs (`"potion"`, `"elder_rowan"`), not by indices.

### Keep logic testable
Pure rules (collision math, damage formulas, dialogue flow, inventory operations) live in plain structs and functions with unit tests.
ECS systems are thin glue that read input, call the logic, and write results.

---

## 2. World and maps

- **Grid-aligned world, free movement.** Tiles are 16×16. Walls come from the `IntGrid` layer and props have their own colliders.
- **Colliders cover the feet**, not the whole sprite. Characters can then stand "behind" a tree canopy or roof, which is what top-down JRPGs look like.
- **Y-sorting** by feet position (`YSort`) handles overlap. Never hand-tune `z`.
- **Readable exits.** Every exit has a visible path or opening and a `Warp` zone. Arrival points sit 2+ tiles away from the return warp so the player doesn't bounce straight back.
- **Consistent world geography.** If the forest is west of the village, the village exit is on the west edge and the forest's return exit is on the east edge. Mirror this in the LDtk world layout.
- **Each map has a purpose:** a landmark, at least one NPC and something to discover. Empty filler maps are boring.
- **Leave room.** Paths at least 2 tiles wide, and one free tile around NPCs and doors.
- **Fade on map transitions** (≈0.25 s) to hide loading and give a sense of travel.
- Persistent entities (player, party, global flags) live outside the level hierarchy so they survive level changes.

## 3. Movement and controls

- Support WASD **and** arrow keys. Plan for gamepad (add input mapping through one input module).
- Resolve collision **per axis** (move-and-slide). Diagonal input against a wall should slide, not stick.
- Normalize diagonal movement, cap the frame delta, and sub-step movement so the player can't tunnel through walls.
- Keep the facing direction stable when walking diagonally, and show an idle frame when stopped.
- One "confirm" action (Space / Enter / E / Z) to talk, read signs, open chests and advance text. Add a "cancel" action (Esc / X) when menus arrive.
- Input pressed to close a dialogue must not re-open it in the same frame. State transitions handle this.

## 4. NPCs and dialogue

- NPCs **turn to face the player** when spoken to.
- Use a **typewriter effect**. The first confirm press completes the line, the second advances.
- Show a blinking "continue" indicator once a line is complete.
- Keep lines short (≤ 60 characters), and use 2–4 lines per conversation.
- Every NPC needs a reason to exist: a hint about nearby areas, a piece of lore, humor, or a quest hook.
- Give each NPC a distinct voice (the stern guard, the shy kid, the boastful sailor).
- Later: dialogue trees with choices, conditions (`if flag X`) and effects (`give item`, `set flag`) should be data (e.g. a RON dialogue asset or Yarn Spinner), not Rust code.
- **Fonts and text:** all game text goes through the vi-VN locale (`assets/locale/`) and is drawn with Be Vietnam Pro / Noto Serif Display. `cargo test` fails if any character lacks a glyph, so check coverage before adding new symbols.

## 5. Camera and presentation

- Use an integer zoom (2×) for crisp pixel art with `ImagePlugin::default_nearest()`.
- Clamp the camera to level bounds and center levels smaller than the screen.
- Ease the camera toward the player, but snap on teleports.
- UI (dialogue, menus) lives in screen space, separate from the world camera's zoom.

## 6. Progression systems (roadmap guidance)

- **Save data:** a serializable `SaveGame` struct (current level, position, party, inventory, flags), separate from ECS components. Version it.
- **Story flags:** a `HashMap<String, FlagValue>` resource is enough for a long time. Dialogue, warps and chests read and write flags.
- **Battle:** a separate `PlayState::Battle` with its own UI. Keep the turn order and damage formulas as pure, tested functions.
- **Encounters:** mark encounter zones in LDtk (an IntGrid value or an entity) and roll per step, not per frame.
- **Balancing data** (stats, EXP curves, drop tables) belongs in data files so it can be tuned without code changes.

## 7. Performance and engineering hygiene

- Use `Added<T>` / `Changed<T>` filters for one-time setup instead of re-running every frame.
- Avoid per-frame allocations in hot paths when it's easy to do so. Collision currently rebuilds a small `Vec<Rect>` each frame, which is fine at this scale. Move to a spatial grid if maps grow to hundreds of colliders.
- Load assets up front with `bevy_asset_loader` in `GameState::Loading`, so gameplay never waits on disk.
- Use the debug tools: **F1** for the world inspector and **F2** for collision and warp gizmos.
- Run `cargo fmt`, `cargo clippy --all-targets` and `cargo test` before every commit.
- For fast iteration, run `cargo run --features dev` (Bevy dynamic linking).

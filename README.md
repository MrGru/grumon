# Grumon

A cozy top-down **JRPG** built with the [Bevy](https://bevyengine.org) game engine (0.19) and maps made in [LDtk](https://ldtk.io).

Explore a small world of four connected maps, bump into trees and walls, and chat with the locals.

## Features

- Title screen, then free exploration with 4-directional walking animations
- Collision against map walls (LDtk IntGrid), trees, houses and NPCs, with sliding along walls
- Four maps connected by warps with fade transitions:
  - **Village**: the starting town
  - **Whisperwood Forest**: west of the village
  - **Frostpeak Snowfield**: north of the forest
  - **Sunny Beach**: south of the village
- Ten NPCs with JRPG-style dialogue boxes (typewriter text, NPCs turn to face you)
- Depth sorting so characters walk behind trees and roofs
- 2× pixel-art camera that follows the player and stays inside the map
- Debug tools in debug builds: world inspector and collision gizmos

## Controls

| Action | Keys |
| --- | --- |
| Move | WASD / arrow keys |
| Talk / advance text | Space, Enter, E or Z |
| Start game (title screen) | Enter / Space or click **New Game** |
| Toggle world inspector (debug) | F1 |
| Toggle collision gizmos (debug) | F2 |

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
cargo test                   # unit tests
```

## Project layout

```
assets/
  world.ldtk            LDtk project: all maps, NPCs, warps, props
  gfx/                  tileset and character sprite sheets (ow1..ow10)
  fonts/grumon.ttf      pixel font
src/
  lib.rs                game states and plugin setup
  level.rs              LDtk loading, wall grid, props, warps
  collision.rs          AABB collision and move-and-slide
  player.rs, npc.rs     characters
  dialogue.rs           dialogue box
  transition.rs         map changes with fades
  camera.rs, ysort.rs, animation.rs, main_menu/, debug.rs
docs/
  JRPG_BEST_PRACTICES.md   design and engineering guidelines
AGENTS.md               guide for AI agents and contributors
```

## Editing content

Open `assets/world.ldtk` in LDtk 1.5.x. NPCs (`Npc` entity) have `name`, `sprite` and `dialogue` fields. Map exits are `Warp` zones with `to_level`, `to_x` and `to_y`. See [AGENTS.md](AGENTS.md#editing-maps-assetsworldldtk) for details.

## For contributors and AI agents

- [AGENTS.md](AGENTS.md): commands, code map, conventions
- [docs/JRPG_BEST_PRACTICES.md](docs/JRPG_BEST_PRACTICES.md): how to build JRPG features in this project
- `.claude/agents/`: Claude Code subagents (`bevy-expert`, `level-designer`)

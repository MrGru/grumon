# CLAUDE.md

All project instructions for agents live in [AGENTS.md](AGENTS.md). Read it first.

@AGENTS.md

Claude Code specifics:

- Project subagents live in `.claude/agents/`:
  - `bevy-expert`: Bevy ECS/API questions, version migrations, reviewing systems for correctness and performance.
  - `level-designer`: LDtk map work (new levels, NPCs, warps, dialogue) and checking map data.
- JRPG design rules: [docs/JRPG_BEST_PRACTICES.md](docs/JRPG_BEST_PRACTICES.md).
- Bevy APIs change between releases. Check the source under `~/.cargo/registry/src/*/bevy_*-0.19.*` rather than relying on memory of older versions.

//! Pure, deterministic battle rules (no ECS). See `docs/game-systems.md`.

mod ai;
mod engine;
pub mod rng;
pub mod state;
#[cfg(test)]
mod tests;

pub use engine::{Command, TimelineEntry, Unavailable};
pub use state::*;

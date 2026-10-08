//! Simple AABB collision against the level's wall grid and solid entities.
//!
//! JRPG movement does not need a physics engine: characters only have to stop
//! at walls and slide along them. Every mover keeps a small box at its feet,
//! and each axis is resolved separately so diagonal input slides along walls.

use bevy::prelude::*;

use crate::{TILE_SIZE, level::LevelInfo};

/// Axis-aligned collision box, relative to the entity's translation.
///
/// Any entity with a `Collider` (other than the mover itself) blocks movement.
#[derive(Component, Reflect, Clone, Copy, Debug, Default)]
#[reflect(Component)]
pub struct Collider {
    pub half_size: Vec2,
    pub offset: Vec2,
}

impl Collider {
    pub const fn new(half_size: Vec2, offset: Vec2) -> Self {
        Self { half_size, offset }
    }

    /// Box hugging the bottom of a sprite of size `sprite` (centered origin):
    /// `width` x `height` pixels, sitting `lift` pixels above the sprite's bottom edge.
    pub fn feet(sprite: Vec2, width: f32, height: f32, lift: f32) -> Self {
        Self::new(
            Vec2::new(width, height) / 2.0,
            Vec2::new(0.0, -sprite.y / 2.0 + lift + height / 2.0),
        )
    }

    pub fn aabb(&self, position: Vec2) -> Rect {
        Rect::from_center_half_size(position + self.offset, self.half_size)
    }
}

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Collider>();
    }
}

/// Snapshot of everything that blocks movement this frame.
pub struct CollisionWorld<'a> {
    pub level: &'a LevelInfo,
    pub solids: Vec<Rect>,
}

impl CollisionWorld<'_> {
    pub fn is_blocked(&self, rect: Rect) -> bool {
        self.level.rect_hits_wall(rect) || self.solids.iter().any(|s| overlaps(*s, rect))
    }

    /// Moves `collider` from `position` by `delta`, one axis at a time, stopping
    /// flush against obstacles. Returns the new position.
    pub fn move_and_slide(&self, collider: &Collider, mut position: Vec2, delta: Vec2) -> Vec2 {
        for axis in [Vec2::X, Vec2::Y] {
            let step = delta * axis;
            let length = step.length();
            if length == 0.0 {
                continue;
            }
            // Sub-steps of at most 1px so fast movement can't tunnel through
            // thin colliders and we stop right at the contact point.
            let count = length.ceil();
            let sub_step = step / count;
            for _ in 0..count as u32 {
                let candidate = position + sub_step;
                if self.is_blocked(collider.aabb(candidate)) {
                    break;
                }
                position = candidate;
            }
        }
        position
    }
}

/// Strict overlap: boxes that merely touch do not collide.
pub fn overlaps(a: Rect, b: Rect) -> bool {
    !a.intersect(b).is_empty()
}

/// Grid cells (x, y; y up, origin bottom-left) touched by `rect`.
pub fn cells_in(rect: Rect) -> impl Iterator<Item = IVec2> {
    let min = (rect.min / TILE_SIZE).floor().as_ivec2();
    let max = (rect.max / TILE_SIZE).ceil().as_ivec2() - IVec2::ONE;
    (min.y..=max.y).flat_map(move |y| (min.x..=max.x).map(move |x| IVec2::new(x, y)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level_with_wall(cell: IVec2) -> LevelInfo {
        let mut level = LevelInfo::empty("Test", UVec2::new(10, 10));
        level.set_wall(cell, true);
        level
    }

    #[test]
    fn cells_in_ignores_touching_edges() {
        let cells: Vec<_> = cells_in(Rect::new(16.0, 16.0, 32.0, 32.0)).collect();
        assert_eq!(cells, vec![IVec2::new(1, 1)]);
    }

    #[test]
    fn stops_flush_against_wall() {
        let level = level_with_wall(IVec2::new(3, 0));
        let world = CollisionWorld {
            level: &level,
            solids: vec![],
        };
        let collider = Collider::new(Vec2::splat(4.0), Vec2::ZERO);
        let end = world.move_and_slide(&collider, Vec2::new(24.0, 8.0), Vec2::new(40.0, 0.0));
        // Wall starts at x = 48, box half width is 4.
        assert_eq!(end, Vec2::new(44.0, 8.0));
    }

    #[test]
    fn slides_along_wall_on_diagonal_input() {
        let level = level_with_wall(IVec2::new(3, 1));
        let world = CollisionWorld {
            level: &level,
            solids: vec![],
        };
        let collider = Collider::new(Vec2::splat(4.0), Vec2::ZERO);
        let end = world.move_and_slide(&collider, Vec2::new(40.0, 24.0), Vec2::new(5.0, 5.0));
        assert_eq!(end, Vec2::new(44.0, 29.0));
    }

    #[test]
    fn level_edges_block() {
        let level = LevelInfo::empty("Test", UVec2::new(4, 4));
        let world = CollisionWorld {
            level: &level,
            solids: vec![],
        };
        let collider = Collider::new(Vec2::splat(4.0), Vec2::ZERO);
        let end = world.move_and_slide(&collider, Vec2::new(8.0, 8.0), Vec2::new(-20.0, 0.0));
        assert_eq!(end, Vec2::new(4.0, 8.0));
    }

    #[test]
    fn solids_block() {
        let level = LevelInfo::empty("Test", UVec2::new(10, 10));
        let world = CollisionWorld {
            level: &level,
            solids: vec![Rect::new(30.0, 0.0, 40.0, 40.0)],
        };
        let collider = Collider::new(Vec2::splat(4.0), Vec2::ZERO);
        let end = world.move_and_slide(&collider, Vec2::new(8.0, 8.0), Vec2::new(40.0, 0.0));
        assert_eq!(end, Vec2::new(26.0, 8.0));
    }
}

use bevy::prelude::*;

use crate::GameState;

/// Inclusive range of atlas frames to loop through.
#[derive(Component, Reflect, Clone, Copy, Debug)]
#[reflect(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

/// Direction a character faces. Matches the row order of the `ow*.png` sheets.
#[derive(Component, Reflect, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[reflect(Component)]
pub enum Facing {
    #[default]
    Down,
    Left,
    Right,
    Up,
}

impl Facing {
    pub const FRAMES_PER_ROW: usize = 4;

    pub fn first_frame(self) -> usize {
        let row = match self {
            Facing::Down => 0,
            Facing::Left => 1,
            Facing::Right => 2,
            Facing::Up => 3,
        };
        row * Self::FRAMES_PER_ROW
    }

    /// Looping walk cycle for this direction.
    pub fn walk(self) -> AnimationIndices {
        let first = self.first_frame();
        AnimationIndices {
            first,
            last: first + Self::FRAMES_PER_ROW - 1,
        }
    }

    /// Single standing frame for this direction.
    pub fn idle(self) -> AnimationIndices {
        let first = self.first_frame();
        AnimationIndices { first, last: first }
    }

    pub fn as_vec2(self) -> Vec2 {
        match self {
            Facing::Down => Vec2::NEG_Y,
            Facing::Left => Vec2::NEG_X,
            Facing::Right => Vec2::X,
            Facing::Up => Vec2::Y,
        }
    }

    /// Direction that best matches `dir` (dominant axis wins).
    pub fn from_vec2(dir: Vec2) -> Self {
        if dir.x.abs() > dir.y.abs() {
            if dir.x < 0.0 {
                Facing::Left
            } else {
                Facing::Right
            }
        } else if dir.y < 0.0 {
            Facing::Down
        } else {
            Facing::Up
        }
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AnimationIndices>()
            .register_type::<Facing>()
            .add_systems(Update, animate_sprites.run_if(in_state(GameState::Playing)));
    }
}

fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        let Some(atlas) = &mut sprite.texture_atlas else {
            continue;
        };
        // Snap immediately when the range changes (e.g. turning around).
        if atlas.index < indices.first || atlas.index > indices.last {
            atlas.index = indices.first;
            timer.reset();
            continue;
        }
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index >= indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

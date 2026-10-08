use bevy::prelude::*;

use crate::{GAME_HEIGHT, GAME_WIDTH, GameState, level::LevelInfo, player::Player};

/// World pixels are drawn at 2x for a chunky pixel-art look.
const CAMERA_ZOOM: f32 = 2.0;
/// How quickly the camera catches up with the player (higher = snappier).
const FOLLOW_SHARPNESS: f32 = 10.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            PostUpdate,
            follow_player
                .run_if(in_state(GameState::Playing))
                .before(TransformSystems::Propagate),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0 / CAMERA_ZOOM,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Visible area in world pixels.
fn view_size() -> Vec2 {
    Vec2::new(GAME_WIDTH as f32, GAME_HEIGHT as f32) / CAMERA_ZOOM
}

/// Keeps the view inside the level; levels smaller than the view are centered.
fn clamp_to_level(target: Vec2, level_size: Vec2) -> Vec2 {
    let half_view = view_size() / 2.0;
    let clamp_axis = |value: f32, size: f32, half: f32| {
        if size <= half * 2.0 {
            size / 2.0
        } else {
            value.clamp(half, size - half)
        }
    };
    Vec2::new(
        clamp_axis(target.x, level_size.x, half_view.x),
        clamp_axis(target.y, level_size.y, half_view.y),
    )
}

fn follow_player(
    time: Res<Time>,
    level: Res<LevelInfo>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let target = clamp_to_level(player.translation.truncate(), level.size_px());
    let current = camera.translation.truncate();
    // Jump instantly on big moves (spawn, warp), otherwise ease toward the player.
    let next = if current.distance(target) > view_size().x {
        target
    } else {
        current.lerp(target, 1.0 - (-FOLLOW_SHARPNESS * time.delta_secs()).exp())
    };
    camera.translation = next.extend(camera.translation.z);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_inside_large_levels_and_centers_small_ones() {
        let level = Vec2::new(960.0, 200.0);
        let half = view_size() / 2.0;
        assert_eq!(clamp_to_level(Vec2::ZERO, level), Vec2::new(half.x, 100.0));
        assert_eq!(
            clamp_to_level(Vec2::new(5000.0, 0.0), level).x,
            960.0 - half.x
        );
    }
}

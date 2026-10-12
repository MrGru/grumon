//! Dev-only tools (debug builds):
//! - F1: toggle the egui world inspector
//! - F2: toggle collider / warp gizmos
//! - F3 / F4: open the Luyện khí / Luyện đan screen (before the story offers them)

use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{
    GameState, PlayState,
    collision::Collider,
    content::defs::Station,
    level::{LevelInfo, Warp},
    workshop::WorkshopRequest,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if !cfg!(debug_assertions) {
            return;
        }
        app.add_plugins(EguiPlugin::default())
            .add_plugins(
                WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F1)),
            )
            .add_systems(
                Update,
                draw_collision_gizmos.run_if(
                    in_state(GameState::Playing).and_then(input_toggle_active(false, KeyCode::F2)),
                ),
            )
            .add_systems(Update, open_station.run_if(in_state(PlayState::Exploring)));
    }
}

fn draw_collision_gizmos(
    mut gizmos: Gizmos,
    level: Res<LevelInfo>,
    colliders: Query<(&Collider, &GlobalTransform)>,
    warps: Query<(&Warp, &GlobalTransform)>,
) {
    let tile = Vec2::splat(crate::TILE_SIZE);
    for y in 0..level.grid_size.y as i32 {
        for x in 0..level.grid_size.x as i32 {
            let cell = IVec2::new(x, y);
            if level.is_wall(cell) {
                let center = (cell.as_vec2() + 0.5) * tile;
                gizmos.rect_2d(center, tile, Color::srgba(1.0, 0.2, 0.2, 0.6));
            }
        }
    }
    for (collider, gt) in &colliders {
        let aabb = collider.aabb(gt.translation().truncate());
        gizmos.rect_2d(aabb.center(), aabb.size(), Color::srgb(0.2, 1.0, 0.3));
    }
    for (warp, gt) in &warps {
        gizmos.rect_2d(
            gt.translation().truncate(),
            warp.size,
            Color::srgb(1.0, 0.9, 0.2),
        );
    }
}

fn open_station(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    let station = if keys.just_pressed(KeyCode::F3) {
        Station::Forge
    } else if keys.just_pressed(KeyCode::F4) {
        Station::Alchemy
    } else {
        return;
    };
    commands.insert_resource(WorkshopRequest::Craft(station));
    next_state.set(PlayState::Workshop);
}

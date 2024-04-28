use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    asset::GameAssets,
    GameState,
};

pub struct PlayerPlugin;

#[derive(Component, InspectorOptions, Reflect)]
pub struct Player {
    pub speed: f32,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player);
        app.add_systems(Update, player_movement.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_player(mut commands: Commands, game_assets: Res<GameAssets>) {
    let animation_indices = AnimationIndices { first: 0, last: 3 };

    commands
        .spawn((
            SpriteBundle {
                texture: game_assets.player_texture.clone(),
                transform: Transform::from_xyz(100.0, 100.0, 10.0),
                ..SpriteBundle::default()
            },
            TextureAtlas {
                layout: game_assets.player_layout.clone(),
                index: animation_indices.first,
            },
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
        ))
        .insert(Name::new("Player"))
        .insert(Player { speed: 120.0 });
}

fn player_movement(
    mut player_query: Query<(&Player, &mut Transform, &mut AnimationIndices)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (player, mut transform, mut animation_indices) = player_query.single_mut();

    if keyboard.pressed(KeyCode::KeyW) {
        transform.translation.y += player.speed * time.delta_seconds();
        animation_indices.first = 12;
        animation_indices.last = 15;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        transform.translation.y -= player.speed * time.delta_seconds();
        animation_indices.first = 0;
        animation_indices.last = 3;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        transform.translation.x -= player.speed * time.delta_seconds();
        animation_indices.first = 4;
        animation_indices.last = 7;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        transform.translation.x += player.speed * time.delta_seconds();
        animation_indices.first = 8;
        animation_indices.last = 11;
    }
}

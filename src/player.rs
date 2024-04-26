use bevy::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    asset::GameAssets,
    GameState,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), draw_the_player);
    }
}

fn draw_the_player(mut commands: Commands, game_assets: Res<GameAssets>) {
    let animation_indices = AnimationIndices { first: 0, last: 3 };

    commands.spawn((
        SpriteBundle {
            texture: game_assets.player_texture.clone(),
            transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
            ..SpriteBundle::default()
        },
        TextureAtlas {
            layout: game_assets.player_layout.clone(),
            index: animation_indices.first,
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    ));
}

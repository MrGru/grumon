use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::InspectorOptions;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    asset::GameAssets,
    components::ysort::YSort,
    GameState,
};

pub struct PlayerPlugin;

#[derive(Component, InspectorOptions, Reflect, Default)]
pub struct Player {
    pub speed: f32,
}

#[derive(Component, Default)]
pub struct Tree;

#[derive(Bundle, LdtkEntity, Default)]
pub struct TreeBundle {
    tree: Tree,
    #[sprite_sheet_bundle(no_grid)]
    sprite_sheet_bundle: SpriteSheetBundle,
    pub y_sort: YSort,
}

#[derive(Component, Default)]
pub struct House;

#[derive(Bundle, LdtkEntity, Default)]
pub struct HouseBundle {
    house: House,
    #[sprite_sheet_bundle(no_grid)]
    sprite_sheet_bundle: SpriteSheetBundle,
    pub y_sort: YSort,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player);
        app.register_ldtk_entity::<TreeBundle>("Tree");
        app.register_ldtk_entity::<HouseBundle>("House");
        app.add_systems(Update, player_movement.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_player(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    new_entity_instances: Query<(Entity, &EntityInstance, &Transform), Added<EntityInstance>>,
) {
    for (entity, entity_instance, transform) in new_entity_instances.iter() {
        if entity_instance.identifier == "Player".to_string() {
            let animation_indices = AnimationIndices { first: 0, last: 3 };

            commands
                .entity(entity)
                .insert((
                    SpriteBundle {
                        texture: game_assets.player_texture.clone(),
                        transform: *transform,
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
                .insert(Player { speed: 120.0 })
                .insert(YSort { z: 0.0 });
        }
    }
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

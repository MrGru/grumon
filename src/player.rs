use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    GameState, PlayState,
    animation::{AnimationIndices, AnimationTimer, Facing},
    asset::GameAssets,
    collision::{Collider, CollisionWorld},
    level::LevelInfo,
    ysort::YSort,
};

const PLAYER_SPEED: f32 = 90.0;
const PLAYER_SIZE: Vec2 = Vec2::splat(32.0);
/// Character sheet used for the hero (`ow1.png`).
const PLAYER_SHEET: usize = 1;
/// Cap the frame delta so a hitch can't push the player through a wall.
const MAX_STEP_SECS: f32 = 1.0 / 20.0;

pub struct PlayerPlugin;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub speed: f32,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_systems(Update, spawn_player.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                player_movement
                    .in_set(PlayerMovement)
                    .run_if(in_state(PlayState::Exploring)),
            )
            .add_systems(OnExit(PlayState::Exploring), stop_walking);
    }
}

/// Systems that read the player's position after it moved this frame should run after this.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerMovement;

/// Feet hitbox: narrower than the sprite so the player fits through 1-tile gaps.
pub fn player_collider() -> Collider {
    Collider::feet(PLAYER_SIZE, 12.0, 8.0, 1.0)
}

/// The LDtk "Player" entity only marks the start position. The real player is
/// a top-level entity so it survives level changes.
fn spawn_player(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    markers: Query<(Entity, &EntityInstance, &Transform), Added<EntityInstance>>,
    players: Query<(), With<Player>>,
) {
    for (entity, entity_instance, transform) in &markers {
        if entity_instance.identifier != "Player" {
            continue;
        }
        commands.entity(entity).despawn();
        if !players.is_empty() {
            continue;
        }
        info!("spawning player");
        commands.spawn((
            Name::new("Player"),
            Player {
                speed: PLAYER_SPEED,
            },
            game_assets.character_sprite(PLAYER_SHEET),
            Transform::from_translation(transform.translation.truncate().extend(0.0)),
            Facing::Down,
            Facing::Down.idle(),
            AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
            YSort::from_height(PLAYER_SIZE.y),
            player_collider(),
        ));
    }
}

fn movement_input(keyboard: &ButtonInput<KeyCode>) -> Vec2 {
    let pressed = |keys: [KeyCode; 2]| keyboard.any_pressed(keys) as i32 as f32;
    Vec2::new(
        pressed([KeyCode::KeyD, KeyCode::ArrowRight])
            - pressed([KeyCode::KeyA, KeyCode::ArrowLeft]),
        pressed([KeyCode::KeyW, KeyCode::ArrowUp]) - pressed([KeyCode::KeyS, KeyCode::ArrowDown]),
    )
}

fn player_movement(
    mut player: Query<
        (
            &Player,
            &Collider,
            &mut Transform,
            &mut Facing,
            &mut AnimationIndices,
        ),
        With<Player>,
    >,
    solids: Query<(&Collider, &GlobalTransform), Without<Player>>,
    level: Res<LevelInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((player, collider, mut transform, mut facing, mut animation)) = player.single_mut()
    else {
        return;
    };

    let input = movement_input(&keyboard);
    if input == Vec2::ZERO {
        *animation = facing.idle();
        return;
    }

    // Keep the current facing while it is still one of the pressed directions,
    // so walking diagonally doesn't flicker between sprites.
    if input.dot(facing.as_vec2()) <= 0.0 {
        *facing = Facing::from_vec2(input);
    }
    *animation = facing.walk();

    let world = CollisionWorld {
        level: &level,
        solids: solids
            .iter()
            .map(|(c, gt)| c.aabb(gt.translation().truncate()))
            .collect(),
    };
    let dt = time.delta_secs().min(MAX_STEP_SECS);
    let delta = input.normalize() * player.speed * dt;
    let position = world.move_and_slide(collider, transform.translation.truncate(), delta);
    transform.translation = position.extend(transform.translation.z);
}

fn stop_walking(mut player: Query<(&Facing, &mut AnimationIndices), With<Player>>) {
    for (facing, mut animation) in &mut player {
        *animation = facing.idle();
    }
}

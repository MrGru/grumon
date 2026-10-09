use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    GameState, PlayState,
    animation::{AnimationIndices, AnimationTimer, Facing},
    asset::GameAssets,
    collision::{Collider, CollisionWorld},
    flow::SessionSetup,
    input::MenuInput,
    level::LevelInfo,
    story::Progress,
    transition::PLAYER_FEET_TO_CENTER,
    ysort::YSort,
};

const PLAYER_SPEED: f32 = 90.0;
const PLAYER_SIZE: Vec2 = Vec2::splat(32.0);
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
            .add_systems(
                OnEnter(GameState::Playing),
                spawn_player.after(SessionSetup),
            )
            .add_systems(
                Update,
                remove_start_markers.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (player_movement, record_position)
                    .chain()
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

/// The player is a top-level entity so it survives level changes. It is
/// placed by the transition that loads the first level.
fn spawn_player(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    progress: Option<Res<Progress>>,
) {
    let sheet = progress.map_or(1, |p| p.profile.sheet);
    commands.spawn((
        Name::new("Player"),
        Player {
            speed: PLAYER_SPEED,
        },
        DespawnOnExit(GameState::Playing),
        game_assets.character_sprite(sheet),
        Transform::from_xyz(-10_000.0, -10_000.0, 0.0),
        Facing::Down,
        Facing::Down.idle(),
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
        YSort::from_height(PLAYER_SIZE.y),
        player_collider(),
    ));
}

/// The LDtk `Player` entity is only an editor reference for the start point.
fn remove_start_markers(
    mut commands: Commands,
    markers: Query<(Entity, &EntityInstance), Added<EntityInstance>>,
) {
    for (entity, instance) in &markers {
        if instance.identifier == "Player" {
            commands.entity(entity).despawn();
        }
    }
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
    input: Res<MenuInput>,
    time: Res<Time>,
) {
    let Ok((player, collider, mut transform, mut facing, mut animation)) = player.single_mut()
    else {
        return;
    };

    let input = input.axis;
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

/// Keeps the save position (feet, LDtk pixels) up to date.
fn record_position(
    player: Query<&Transform, With<Player>>,
    level: Res<LevelInfo>,
    progress: Option<ResMut<Progress>>,
) {
    let (Ok(transform), Some(mut progress)) = (player.single(), progress) else {
        return;
    };
    if level.identifier.is_empty() {
        return;
    }
    let feet = Vec2::new(
        transform.translation.x,
        level.size_px().y - (transform.translation.y - PLAYER_FEET_TO_CENTER),
    )
    .round()
    .as_ivec2();
    if progress.level != level.identifier {
        progress.level = level.identifier.clone();
    }
    if progress.feet != (feet.x, feet.y) {
        progress.feet = (feet.x, feet.y);
    }
}

fn stop_walking(mut player: Query<(&Facing, &mut AnimationIndices), With<Player>>) {
    for (facing, mut animation) in &mut player {
        *animation = facing.idle();
    }
}

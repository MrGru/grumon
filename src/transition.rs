//! Map changes: stepping on a [`Warp`] fades out, swaps the LDtk level,
//! moves the player to the arrival point and fades back in.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    GameState, PlayState,
    collision::{Collider, overlaps},
    level::{LevelInfo, LevelReady, Warp},
    player::{Player, PlayerMovement},
};

/// Seconds for a full fade in or out.
const FADE_SECS: f32 = 0.25;
/// The player's transform is the sprite center; arrival points are the feet.
const PLAYER_FEET_TO_CENTER: f32 = 16.0;

#[derive(Resource, Debug, Clone)]
struct PendingWarp {
    to_level: String,
    to: IVec2,
    level_requested: bool,
}

#[derive(Component)]
struct ScreenFade;

/// Current fade opacity (0 = clear, 1 = black).
#[derive(Resource, Default)]
struct FadeAlpha(f32);

pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FadeAlpha>()
            .add_systems(OnEnter(GameState::Playing), spawn_screen_fade)
            .add_systems(
                Update,
                check_warps
                    .after(PlayerMovement)
                    .run_if(in_state(PlayState::Exploring)),
            )
            .add_systems(
                Update,
                (fade_out_and_switch_level, arrive_in_level)
                    .chain()
                    .run_if(in_state(PlayState::Transition)),
            )
            .add_systems(
                Update,
                fade_in.run_if(
                    in_state(GameState::Playing).and_then(not(in_state(PlayState::Transition))),
                ),
            );
    }
}

fn spawn_screen_fade(mut commands: Commands) {
    commands.spawn((
        Name::new("ScreenFade"),
        ScreenFade,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.0)),
        GlobalZIndex(100),
        // Don't block clicks on UI underneath.
        Pickable::IGNORE,
    ));
}

fn check_warps(
    mut commands: Commands,
    player: Query<(&Transform, &Collider), With<Player>>,
    warps: Query<(&Warp, &GlobalTransform)>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    let Ok((transform, collider)) = player.single() else {
        return;
    };
    let feet = collider.aabb(transform.translation.truncate());
    let Some((warp, _)) = warps
        .iter()
        .find(|(warp, gt)| overlaps(warp.aabb(gt.translation().truncate()), feet))
    else {
        return;
    };
    info!("warping to {} at {}", warp.to_level, warp.to);
    commands.insert_resource(PendingWarp {
        to_level: warp.to_level.clone(),
        to: warp.to,
        level_requested: false,
    });
    next_state.set(PlayState::Transition);
}

fn set_fade(alpha: f32, fade: &mut FadeAlpha, overlay: &mut BackgroundColor) {
    fade.0 = alpha.clamp(0.0, 1.0);
    overlay.0 = Color::BLACK.with_alpha(fade.0);
}

fn fade_out_and_switch_level(
    time: Res<Time>,
    mut fade: ResMut<FadeAlpha>,
    mut overlay: Single<&mut BackgroundColor, With<ScreenFade>>,
    mut pending: ResMut<PendingWarp>,
    mut level_selection: ResMut<LevelSelection>,
) {
    if fade.0 < 1.0 {
        let alpha = fade.0 + time.delta_secs() / FADE_SECS;
        set_fade(alpha, &mut fade, &mut overlay);
        return;
    }
    if !pending.level_requested {
        *level_selection = LevelSelection::Identifier(pending.to_level.clone());
        pending.level_requested = true;
    }
}

fn arrive_in_level(
    mut commands: Commands,
    mut level_ready: MessageReader<LevelReady>,
    pending: Res<PendingWarp>,
    level: Res<LevelInfo>,
    mut player: Single<&mut Transform, With<Player>>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    // Always drain, so a message from an earlier level can't be mistaken for ours.
    let ready = level_ready.read().count() > 0;
    if !pending.level_requested || !ready {
        return;
    }
    if level.identifier != pending.to_level {
        warn!(
            "expected level {}, got {}",
            pending.to_level, level.identifier
        );
    }
    let feet = level.ldtk_to_world(pending.to);
    player.translation.x = feet.x;
    player.translation.y = feet.y + PLAYER_FEET_TO_CENTER;
    commands.remove_resource::<PendingWarp>();
    next_state.set(PlayState::Exploring);
}

fn fade_in(
    time: Res<Time>,
    mut fade: ResMut<FadeAlpha>,
    mut overlay: Single<&mut BackgroundColor, With<ScreenFade>>,
) {
    if fade.0 > 0.0 {
        let alpha = fade.0 - time.delta_secs() / FADE_SECS;
        set_fade(alpha, &mut fade, &mut overlay);
    }
}

//! Map changes: stepping on a [`Warp`] (or a scripted warp) fades out, swaps
//! the LDtk level, moves the player to the arrival point and fades back in.
//! Every session starts here too: the screen is black until the level loads.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    GameState, PlayState,
    collision::{Collider, overlaps},
    level::{LevelInfo, LevelReady, Warp},
    player::{Player, PlayerMovement},
};

/// Seconds for a full fade in or out.
const FADE_SECS: f32 = 0.3;
/// The player's transform is the sprite center; arrival points are the feet.
pub const PLAYER_FEET_TO_CENTER: f32 = 16.0;

/// A level change in progress.
#[derive(Resource, Debug, Clone)]
pub struct PendingWarp {
    to_level: String,
    /// Arrival of the player's feet, LDtk pixels.
    to: IVec2,
    level_requested: bool,
}

impl PendingWarp {
    pub fn level(&self) -> &str {
        &self.to_level
    }

    pub fn to(level: &str, feet: IVec2) -> Self {
        Self {
            to_level: level.to_string(),
            to: feet,
            level_requested: false,
        }
    }
}

#[derive(Component)]
struct ScreenFade;

/// Current fade opacity (0 = clear, 1 = black).
#[derive(Resource, Default)]
pub struct FadeAlpha(pub f32);

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
                (fade_out_and_switch_level, arrive_in_level).chain().run_if(
                    in_state(PlayState::Transition).and_then(resource_exists::<PendingWarp>),
                ),
            )
            .add_systems(
                Update,
                fade_in.run_if(
                    in_state(GameState::Playing).and_then(not(in_state(PlayState::Transition))),
                ),
            );
    }
}

fn spawn_screen_fade(mut commands: Commands, mut fade: ResMut<FadeAlpha>) {
    // Start black: the first level is still loading.
    fade.0 = 1.0;
    commands.spawn((
        Name::new("ScreenFade"),
        ScreenFade,
        DespawnOnExit(GameState::Playing),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::BLACK),
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
    commands.insert_resource(PendingWarp::to(&warp.to_level, warp.to));
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
    level: Res<LevelInfo>,
    mut level_ready: MessageWriter<LevelReady>,
) {
    if fade.0 < 1.0 {
        let alpha = fade.0 + time.delta_secs() / FADE_SECS;
        set_fade(alpha, &mut fade, &mut overlay);
        return;
    }
    if !pending.level_requested {
        pending.level_requested = true;
        if level.identifier == pending.to_level {
            // Same map (scripted warps): no reload, arrive right away.
            level_ready.write(LevelReady);
        } else {
            *level_selection = LevelSelection::Identifier(pending.to_level.clone());
        }
    }
}

fn arrive_in_level(
    mut commands: Commands,
    mut level_ready: MessageReader<LevelReady>,
    pending: Res<PendingWarp>,
    level: Res<LevelInfo>,
    mut player: Query<&mut Transform, With<Player>>,
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
    let Ok(mut transform) = player.single_mut() else {
        return;
    };
    let feet = level.ldtk_to_world(pending.to);
    transform.translation.x = feet.x;
    transform.translation.y = feet.y + PLAYER_FEET_TO_CENTER;
    commands.remove_resource::<PendingWarp>();
    next_state.set(PlayState::Exploring);
}

fn fade_in(
    time: Res<Time>,
    mut fade: ResMut<FadeAlpha>,
    mut overlay: Query<&mut BackgroundColor, With<ScreenFade>>,
) {
    let Ok(mut overlay) = overlay.single_mut() else {
        return;
    };
    if fade.0 > 0.0 {
        let alpha = fade.0 - time.delta_secs() / FADE_SECS;
        set_fade(alpha, &mut fade, &mut overlay);
    }
}

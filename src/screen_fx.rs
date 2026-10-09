//! Screen-wide mood effects: time-of-day tint, rain and fire glow for the
//! raid, and a blackout for narrated scenes (`fx.blackout` flag).

use bevy::prelude::*;

use crate::{GameState, PlayState, content::defs::TimeOfDay, story::Progress};

const RAIN_DROPS: usize = 90;

#[derive(Component)]
struct Tint;

#[derive(Component)]
struct Blackout;

#[derive(Component)]
struct RainDrop {
    speed: f32,
}

pub struct ScreenFxPlugin;

impl Plugin for ScreenFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_fx)
            .add_systems(
                Update,
                (update_tint, update_rain).run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_fx(mut commands: Commands) {
    let overlay = || Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    };
    commands.spawn((
        Name::new("TimeTint"),
        Tint,
        DespawnOnExit(GameState::Playing),
        overlay(),
        BackgroundColor(Color::NONE),
        GlobalZIndex(1),
        Pickable::IGNORE,
    ));
    commands.spawn((
        Name::new("Blackout"),
        Blackout,
        DespawnOnExit(GameState::Playing),
        overlay(),
        BackgroundColor(Color::BLACK),
        Visibility::Hidden,
        GlobalZIndex(15),
        Pickable::IGNORE,
    ));
    // Deterministic pseudo-random layout for the rain.
    let mut seed: u32 = 0x9e37_79b9;
    let mut rand = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed % 10_000) as f32 / 10_000.0
    };
    for _ in 0..RAIN_DROPS {
        commands.spawn((
            RainDrop {
                speed: 380.0 + rand() * 260.0,
            },
            DespawnOnExit(GameState::Playing),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(rand() * 100.0),
                top: Val::Percent(rand() * 100.0),
                width: Val::Px(1.5),
                height: Val::Px(10.0 + rand() * 10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.7, 0.75, 0.9, 0.35)),
            Visibility::Hidden,
            GlobalZIndex(2),
            Pickable::IGNORE,
        ));
    }
}

fn tint_for(time: TimeOfDay, t: f32) -> Color {
    match time {
        TimeOfDay::Day => Color::NONE,
        TimeOfDay::Dusk => Color::srgba(0.95, 0.45, 0.15, 0.22),
        TimeOfDay::Night => Color::srgba(0.02, 0.04, 0.16, 0.52),
        // Rain-dark blue with a flickering fire glow.
        TimeOfDay::Raid => {
            let flicker = (t * 7.3).sin() * 0.5 + (t * 13.1).sin() * 0.3;
            Color::srgba(0.30 + 0.06 * flicker, 0.04, 0.06, 0.46 + 0.04 * flicker)
        }
        TimeOfDay::Dawn => Color::srgba(0.75, 0.70, 0.85, 0.16),
    }
}

fn update_tint(
    time: Res<Time>,
    progress: Option<Res<Progress>>,
    state: Res<State<PlayState>>,
    mut tint: Query<&mut BackgroundColor, (With<Tint>, Without<Blackout>)>,
    mut blackout: Query<&mut Visibility, With<Blackout>>,
) {
    let Some(progress) = progress else {
        return;
    };
    let in_battle = *state.get() == PlayState::Battle;
    if let Ok(mut bg) = tint.single_mut() {
        bg.0 = if in_battle {
            Color::NONE
        } else {
            tint_for(progress.time, time.elapsed_secs())
        };
    }
    if let Ok(mut v) = blackout.single_mut() {
        let wanted = if progress.flag("fx.blackout") != 0 && !in_battle {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *v != wanted {
            *v = wanted;
        }
    }
}

fn update_rain(
    time: Res<Time>,
    progress: Option<Res<Progress>>,
    state: Res<State<PlayState>>,
    mut drops: Query<(&RainDrop, &mut Node, &mut Visibility)>,
) {
    let raining =
        progress.is_some_and(|p| p.time == TimeOfDay::Raid) && *state.get() != PlayState::Battle;
    for (drop, mut node, mut visibility) in &mut drops {
        let wanted = if raining {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
        if !raining {
            continue;
        }
        if let Val::Percent(top) = node.top {
            let mut next = top + drop.speed * time.delta_secs() / 6.4;
            if next > 100.0 {
                next -= 105.0;
            }
            node.top = Val::Percent(next);
        }
    }
}

//! Field HUD: map name banner, notifications, quest tracker, control hints,
//! and full-screen story cards ([`PlayState::Card`]).

use bevy::prelude::*;

use crate::{
    GameState, PlayState,
    asset::GameAssets,
    content::{Content, defs::QuestKind},
    flow::Notices,
    input::MenuInput,
    level::{LevelInfo, LevelReady},
    story::{Notice, Progress, QuestState},
    ui::{self, FontKind},
};

const NOTICE_SECS: f32 = 3.5;
const BANNER_SECS: f32 = 2.6;
const CARD_FADE_SECS: f32 = 0.8;
const CARD_MIN_SECS: f32 = 1.4;

/// Shows `card.<key>.title` / `card.<key>.subtitle` full screen.
#[derive(Resource, Debug, Clone)]
pub struct CardRequest(pub String);

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct NoticeList;

#[derive(Component)]
struct NoticeItem(Timer);

#[derive(Component)]
struct QuestTracker;

#[derive(Component)]
struct MapBanner(Timer);

#[derive(Component)]
struct CardScreen(Stopwatch);

use bevy::time::Stopwatch;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(
                Update,
                (show_notices, tick_notices, update_quest_tracker, map_banner)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, toggle_hud.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(PlayState::Card), spawn_card)
            .add_systems(Update, run_card.run_if(in_state(PlayState::Card)))
            .add_systems(OnExit(PlayState::Card), despawn_card);
    }
}

fn spawn_hud(mut commands: Commands, assets: Res<GameAssets>, content: Res<Content>) {
    commands.spawn((
        Name::new("Hud"),
        HudRoot,
        DespawnOnExit(GameState::Playing),
        ui::fullscreen(),
        GlobalZIndex(5),
        Pickable::IGNORE,
        children![
            (
                QuestTracker,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(14.0),
                    top: Val::Px(12.0),
                    max_width: Val::Px(360.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.05, 0.06, 0.72)),
                Visibility::Hidden,
                children![
                    (
                        Text::new(""),
                        ui::font(&assets, FontKind::Bold, 16.0),
                        TextColor(ui::GOLD)
                    ),
                    (
                        Text::new(""),
                        ui::font(&assets, FontKind::Body, 15.0),
                        TextColor(ui::TEXT)
                    ),
                ],
            ),
            (
                NoticeList,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(14.0),
                    top: Val::Px(12.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    bottom: Val::Px(6.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.7)),
                children![(
                    Text::new(content.ui("ui.hud.controls")),
                    ui::font(&assets, FontKind::Body, 14.0),
                    TextColor(ui::TEXT),
                )],
            ),
        ],
    ));
}

/// The HUD is only shown while exploring.
fn toggle_hud(state: Res<State<PlayState>>, mut hud: Query<&mut Visibility, With<HudRoot>>) {
    if !state.is_changed() {
        return;
    }
    for mut v in &mut hud {
        *v = if *state.get() == PlayState::Exploring {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Player-facing text of a notice.
pub fn notice_text(notice: &Notice, content: &Content, progress: &Progress) -> String {
    let item_name = |id: &str| {
        if content.db.artifacts.contains_key(id) {
            content.text(&format!("artifact.{id}.name"), progress)
        } else {
            content.text(&format!("item.{id}.name"), progress)
        }
    };
    let quest = |id: &str| content.text(&format!("quest.{id}.title"), progress);
    match notice {
        Notice::ItemGained(id, n) => content.format(
            "notice.item_gained",
            progress,
            &[("item", item_name(id)), ("count", n.to_string())],
        ),
        Notice::ItemLost(id, n) => content.format(
            "notice.item_lost",
            progress,
            &[("item", item_name(id)), ("count", n.to_string())],
        ),
        Notice::Money(n) if *n >= 0 => {
            content.format("notice.money_gained", progress, &[("count", n.to_string())])
        }
        Notice::Money(n) => content.format(
            "notice.money_lost",
            progress,
            &[("count", (-n).to_string())],
        ),
        Notice::QuestStarted(id) => {
            content.format("notice.quest_started", progress, &[("quest", quest(id))])
        }
        Notice::QuestDone(id) => {
            content.format("notice.quest_done", progress, &[("quest", quest(id))])
        }
        Notice::QuestFailed(id) => {
            content.format("notice.quest_failed", progress, &[("quest", quest(id))])
        }
        Notice::Joined(id) => content.format(
            "notice.joined",
            progress,
            &[("name", content.character_name(id, progress))],
        ),
        Notice::Left(id) => content.format(
            "notice.left",
            progress,
            &[("name", content.character_name(id, progress))],
        ),
        Notice::SkillLearned(member, skill) => content.format(
            "notice.skill_learned",
            progress,
            &[
                ("name", content.character_name(member, progress)),
                (
                    "skill",
                    content.text(&format!("skill.{skill}.name"), progress),
                ),
            ],
        ),
        Notice::ArtifactGained(id) => content.format(
            "notice.artifact_gained",
            progress,
            &[(
                "artifact",
                content.text(&format!("artifact.{id}.name"), progress),
            )],
        ),
        Notice::TuVi(n) => content.format("notice.tu_vi", progress, &[("count", n.to_string())]),
        Notice::StageUp(member, realm, stage) => content.format(
            "notice.stage_up",
            progress,
            &[
                ("name", content.character_name(member, progress)),
                ("realm", realm_text(content, progress, *realm, *stage)),
            ],
        ),
        Notice::Custom(key) => content.text(key, progress),
    }
}

/// “Luyện Khí sơ kỳ”, “Phàm Nhân”.
pub fn realm_text(
    content: &Content,
    progress: &Progress,
    realm: crate::content::defs::Realm,
    stage: u8,
) -> String {
    let name = content.text(realm.key(), progress);
    if realm == crate::content::defs::Realm::PhamNhan {
        name
    } else {
        format!(
            "{name} {}",
            content.text(&format!("stage.{}", stage.min(3)), progress)
        )
    }
}

fn show_notices(
    mut commands: Commands,
    assets: Res<GameAssets>,
    content: Res<Content>,
    progress: Option<Res<Progress>>,
    mut notices: ResMut<Notices>,
    list: Query<Entity, With<NoticeList>>,
) {
    let (Some(progress), Ok(list)) = (progress, list.single()) else {
        return;
    };
    while let Some(notice) = notices.0.pop_front() {
        // Tu vi on mortals is meaningless noise.
        if matches!(notice, Notice::TuVi(_))
            && progress
                .player()
                .is_some_and(|p| p.realm == crate::content::defs::Realm::PhamNhan)
        {
            continue;
        }
        let text = notice_text(&notice, &content, &progress);
        commands.entity(list).with_child((
            NoticeItem(Timer::from_seconds(NOTICE_SECS, TimerMode::Once)),
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                border: UiRect::left(Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                max_width: Val::Px(420.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.07, 0.08, 0.86)),
            BorderColor::all(ui::JADE),
            children![(
                Text::new(text),
                ui::font(&assets, FontKind::Body, 16.0),
                TextColor(ui::TEXT)
            )],
        ));
    }
}

fn tick_notices(
    mut commands: Commands,
    time: Res<Time>,
    mut items: Query<(Entity, &mut NoticeItem, &mut BackgroundColor)>,
) {
    for (entity, mut item, mut bg) in &mut items {
        item.0.tick(time.delta());
        let left = item.0.remaining_secs();
        if left < 0.5 {
            bg.0 = bg.0.with_alpha(0.86 * left / 0.5);
        }
        if item.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn update_quest_tracker(
    content: Res<Content>,
    progress: Option<Res<Progress>>,
    tracker: Query<(&Children, &mut Visibility), With<QuestTracker>>,
    mut texts: Query<&mut Text>,
) {
    let Some(progress) = progress else {
        return;
    };
    if !progress.is_changed() {
        return;
    }
    let Ok((children, mut visibility)) = tracker.single_inner() else {
        return;
    };
    // Most recently started active main quest, else any active quest.
    let active = progress
        .quest_order
        .iter()
        .rev()
        .filter(|id| progress.quest(id) == Some(QuestState::Active))
        .filter_map(|id| content.db.quests.get(id))
        .max_by_key(|q| q.kind == QuestKind::Main);
    let Some(quest) = active else {
        *visibility = Visibility::Hidden;
        return;
    };
    *visibility = Visibility::Inherited;
    let title = content.text(&format!("quest.{}.title", quest.id), &progress);
    let objective = progress
        .objectives(quest)
        .into_iter()
        .find(|(_, done)| !done)
        .map(|(o, _)| content.text(&format!("quest.{}.obj.{}", quest.id, o.id), &progress))
        .unwrap_or_default();
    if let Some(mut t) = children.first().and_then(|&c| texts.get_mut(c).ok()) {
        t.0 = title;
    }
    if let Some(mut t) = children.get(1).and_then(|&c| texts.get_mut(c).ok()) {
        t.0 = format!("• {objective}");
    }
}

#[allow(clippy::too_many_arguments)]
fn map_banner(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    content: Res<Content>,
    level: Res<LevelInfo>,
    mut ready: MessageReader<LevelReady>,
    mut banners: Query<(Entity, &mut MapBanner, &Children)>,
    mut colors: Query<&mut TextColor>,
    mut last: Local<String>,
) {
    if ready.read().count() > 0 && *last != level.identifier && !level.identifier.is_empty() {
        *last = level.identifier.clone();
        for (e, _, _) in &banners {
            commands.entity(e).despawn();
        }
        commands.spawn((
            MapBanner(Timer::from_seconds(BANNER_SECS, TimerMode::Once)),
            DespawnOnExit(GameState::Playing),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(70.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(6),
            Pickable::IGNORE,
            children![(
                Text::new(content.ui(&format!("map.{}.name", level.identifier))),
                ui::font(&assets, FontKind::Title, 34.0),
                TextColor(ui::GOLD),
            )],
        ));
        return;
    }
    for (entity, mut banner, children) in &mut banners {
        banner.0.tick(time.delta());
        let t = banner.0.fraction();
        let alpha = if t < 0.2 {
            t / 0.2
        } else if t > 0.7 {
            (1.0 - t) / 0.3
        } else {
            1.0
        };
        for &child in children {
            if let Ok(mut c) = colors.get_mut(child) {
                c.0 = ui::GOLD.with_alpha(alpha);
            }
        }
        if banner.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_card(
    mut commands: Commands,
    assets: Res<GameAssets>,
    content: Res<Content>,
    progress: Res<Progress>,
    request: Option<Res<CardRequest>>,
) {
    let key = request.map(|r| r.0.clone()).unwrap_or_default();
    let title = content.text(&format!("card.{key}.title"), &progress);
    let subtitle_key = format!("card.{key}.subtitle");
    let subtitle = if content.locale.has(&subtitle_key) {
        content.text(&subtitle_key, &progress)
    } else {
        String::new()
    };
    commands.remove_resource::<CardRequest>();
    commands.spawn((
        Name::new("Card"),
        CardScreen(Stopwatch::new()),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(18.0),
            padding: UiRect::horizontal(Val::Px(80.0)),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(120),
        children![
            (
                Text::new(title),
                ui::font(&assets, FontKind::Title, 44.0),
                TextColor(ui::GOLD.with_alpha(0.0)),
                TextLayout::justify(Justify::Center),
            ),
            (
                Text::new(subtitle),
                ui::font(&assets, FontKind::Body, 20.0),
                TextColor(ui::TEXT.with_alpha(0.0)),
                TextLayout::justify(Justify::Center),
            ),
        ],
    ));
}

fn run_card(
    time: Res<Time>,
    mut input: ResMut<MenuInput>,
    mut card: Query<(&mut CardScreen, &Children)>,
    mut colors: Query<&mut TextColor>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    let Ok((mut card, children)) = card.single_mut() else {
        next_state.set(PlayState::Exploring);
        return;
    };
    card.0.tick(time.delta());
    let alpha = (card.0.elapsed_secs() / CARD_FADE_SECS).min(1.0);
    for (i, child) in children.iter().enumerate() {
        if let Ok(mut c) = colors.get_mut(child) {
            let base = if i == 0 { ui::GOLD } else { ui::TEXT };
            c.0 = base.with_alpha(alpha);
        }
    }
    if card.0.elapsed_secs() >= CARD_MIN_SECS && input.take_confirm() {
        next_state.set(PlayState::Exploring);
    }
}

fn despawn_card(mut commands: Commands, cards: Query<Entity, With<CardScreen>>) {
    for e in &cards {
        commands.entity(e).despawn();
    }
}

//! Pause menu: party, inventory, quest journal, save/load, settings.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    GameState, PlayState,
    asset::GameAssets,
    content::{
        Content,
        defs::{ItemCategory, QuestKind},
    },
    hud::realm_text,
    input::MenuInput,
    main_menu::slot_rows,
    save::{self, LoadRequest, SaveRequest, SaveSlot},
    story::{Progress, QuestState},
    ui::{self, FontKind, MenuItem, Row},
};

/// Player preferences, stored next to the saves.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// Faster enemy turns and animations in battle.
    pub fast_battle: bool,
    /// Music volume, `0..=VOLUME_STEPS`.
    pub music_volume: u8,
    /// Sound effect volume, `0..=VOLUME_STEPS`.
    pub sfx_volume: u8,
}

/// Number of volume steps in the settings menu.
pub const VOLUME_STEPS: u8 = 10;

impl Default for Settings {
    fn default() -> Self {
        Self {
            fast_battle: false,
            music_volume: 7,
            sfx_volume: 8,
        }
    }
}

impl Settings {
    fn path() -> std::path::PathBuf {
        save::save_dir().join("settings.json")
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn store(&self) {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self)
            && let Err(e) = std::fs::write(&path, json)
        {
            warn!("could not store settings: {e}");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Party,
    Inventory,
    Journal,
    Save,
    Load,
    Settings,
    Title,
    Quit,
}

const TABS: [Tab; 8] = [
    Tab::Party,
    Tab::Inventory,
    Tab::Journal,
    Tab::Save,
    Tab::Load,
    Tab::Settings,
    Tab::Title,
    Tab::Quit,
];

impl Tab {
    fn key(self) -> &'static str {
        match self {
            Tab::Party => "ui.pause.party",
            Tab::Inventory => "ui.pause.inventory",
            Tab::Journal => "ui.pause.journal",
            Tab::Save => "ui.pause.save",
            Tab::Load => "ui.pause.load",
            Tab::Settings => "ui.pause.settings",
            Tab::Title => "ui.pause.title",
            Tab::Quit => "ui.pause.quit",
        }
    }
}

#[derive(Resource)]
struct PauseMenu {
    tab: usize,
    /// Inside the tab's list (`Some(row)`) or on the tab list.
    inner: Option<usize>,
    dirty: bool,
}

#[derive(Component)]
struct PauseRoot;

#[derive(Component)]
struct TabList;

#[derive(Component)]
struct DetailPanel;

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::load())
            .add_systems(Update, open_pause.run_if(in_state(PlayState::Exploring)))
            .add_systems(OnEnter(PlayState::Paused), spawn_pause)
            .add_systems(
                Update,
                (pause_input, redraw_pause)
                    .chain()
                    .run_if(in_state(PlayState::Paused)),
            )
            .add_systems(OnExit(PlayState::Paused), despawn_pause);
    }
}

fn open_pause(mut input: ResMut<MenuInput>, mut next_state: ResMut<NextState<PlayState>>) {
    if input.menu && !input.consumed {
        input.consumed = true;
        next_state.set(PlayState::Paused);
    }
}

fn spawn_pause(mut commands: Commands, assets: Res<GameAssets>, content: Res<Content>) {
    commands.insert_resource(PauseMenu {
        tab: 0,
        inner: None,
        dirty: true,
    });
    commands.spawn((
        Name::new("Pause"),
        PauseRoot,
        ui::fullscreen(),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        GlobalZIndex(30),
        children![(
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(24.0)),
                column_gap: Val::Px(16.0),
                ..default()
            },
            children![
                (
                    ui::panel(Node {
                        width: Val::Px(230.0),
                        flex_shrink: 0.0,
                        padding: UiRect::all(Val::Px(12.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    }),
                    children![
                        (
                            Text::new(content.ui("ui.pause.header")),
                            ui::font(&assets, FontKind::Title, 28.0),
                            TextColor(ui::GOLD),
                            Node {
                                margin: UiRect::bottom(Val::Px(8.0)),
                                ..default()
                            },
                        ),
                        (
                            TabList,
                            Node {
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                        ),
                    ],
                ),
                (
                    DetailPanel,
                    ui::panel(Node {
                        flex_grow: 1.0,
                        padding: UiRect::all(Val::Px(16.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        overflow: Overflow::clip(),
                        ..default()
                    }),
                ),
            ],
        )],
    ));
}

fn despawn_pause(mut commands: Commands, roots: Query<Entity, With<PauseRoot>>) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<PauseMenu>();
}

/// Inventory entries in display order: (id, count).
fn inventory_entries(content: &Content, progress: &Progress) -> Vec<(String, u32)> {
    let order = |id: &str| {
        content.db.items.get(id).map_or(9, |d| match d.category {
            ItemCategory::Medicine => 0,
            ItemCategory::Material => 1,
            ItemCategory::Artifact => 2,
            ItemCategory::Quest => 3,
        })
    };
    let mut entries: Vec<(String, u32)> = progress
        .items
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    entries.sort_by_key(|(id, _)| (order(id), id.clone()));
    entries
}

/// Quests in journal order: active first (main before side), then finished.
fn journal_entries(content: &Content, progress: &Progress) -> Vec<String> {
    let mut quests: Vec<&String> = progress.quest_order.iter().collect();
    quests.sort_by_key(|id| {
        let state = progress.quest(id);
        let main = content
            .db
            .quests
            .get(*id)
            .is_some_and(|q| q.kind == QuestKind::Main);
        (state != Some(QuestState::Active), !main)
    });
    quests.into_iter().cloned().collect()
}

fn inner_len(tab: Tab, content: &Content, progress: &Progress) -> usize {
    match tab {
        Tab::Party => progress.party.len(),
        Tab::Inventory => inventory_entries(content, progress).len(),
        Tab::Journal => journal_entries(content, progress).len(),
        Tab::Save => save::MANUAL_SLOTS as usize,
        Tab::Load => SaveSlot::all().len(),
        Tab::Settings => 3,
        Tab::Title | Tab::Quit => 0,
    }
}

#[allow(clippy::too_many_arguments)]
fn pause_input(
    mut input: ResMut<MenuInput>,
    mut menu: ResMut<PauseMenu>,
    content: Res<Content>,
    progress: Res<Progress>,
    mut settings: ResMut<Settings>,
    items: Query<(&Interaction, &MenuItem), Changed<Interaction>>,
    mut next_play: ResMut<NextState<PlayState>>,
    mut next_game: ResMut<NextState<GameState>>,
    mut saves: MessageWriter<SaveRequest>,
    mut loads: MessageWriter<LoadRequest>,
    mut exit: MessageWriter<AppExit>,
) {
    let tab = TABS[menu.tab];
    let (_, clicked) = ui::mouse_menu(&items);
    match menu.inner {
        None => {
            if input.menu || input.cancel {
                input.consumed = true;
                next_play.set(PlayState::Exploring);
                return;
            }
            let step = input.vertical();
            if step != 0 {
                menu.tab = (menu.tab as i32 + step).rem_euclid(TABS.len() as i32) as usize;
                menu.dirty = true;
            }
            let activate = clicked.map(|c| {
                menu.tab = c;
                true
            });
            if activate.is_some() || input.take_confirm() || input.right {
                let tab = TABS[menu.tab];
                match tab {
                    Tab::Title => next_game.set(GameState::Menu),
                    Tab::Quit => {
                        exit.write(AppExit::Success);
                    }
                    _ if inner_len(tab, &content, &progress) > 0 => {
                        menu.inner = Some(0);
                    }
                    _ => {}
                }
                menu.dirty = true;
            }
        }
        Some(row) => {
            // Volume rows use left/right to adjust.
            let adjusting = tab == Tab::Settings && row > 0;
            if adjusting && (input.left || input.right) {
                let volume = if row == 1 {
                    &mut settings.music_volume
                } else {
                    &mut settings.sfx_volume
                };
                *volume = if input.right {
                    (*volume + 1).min(VOLUME_STEPS)
                } else {
                    volume.saturating_sub(1)
                };
                settings.store();
                menu.dirty = true;
                return;
            }
            if input.cancel || input.left || input.menu {
                input.consumed = true;
                menu.inner = None;
                menu.dirty = true;
                return;
            }
            let n = inner_len(tab, &content, &progress).max(1);
            let step = input.vertical();
            if step != 0 {
                menu.inner = Some((row as i32 + step).rem_euclid(n as i32) as usize);
                menu.dirty = true;
            }
            if input.take_confirm() {
                match tab {
                    Tab::Save => {
                        saves.write(SaveRequest(SaveSlot::Manual(row as u8 + 1)));
                        menu.dirty = true;
                    }
                    Tab::Load => {
                        loads.write(LoadRequest(SaveSlot::all()[row]));
                    }
                    Tab::Settings => {
                        match row {
                            0 => settings.fast_battle = !settings.fast_battle,
                            1 => {
                                settings.music_volume =
                                    (settings.music_volume + 1) % (VOLUME_STEPS + 1)
                            }
                            _ => {
                                settings.sfx_volume = (settings.sfx_volume + 1) % (VOLUME_STEPS + 1)
                            }
                        }
                        settings.store();
                        menu.dirty = true;
                    }
                    _ => {}
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn redraw_pause(
    mut commands: Commands,
    assets: Res<GameAssets>,
    content: Res<Content>,
    progress: Res<Progress>,
    settings: Res<Settings>,
    mut menu: ResMut<PauseMenu>,
    tab_list: Query<Entity, With<TabList>>,
    detail: Query<Entity, With<DetailPanel>>,
    mut frames: Local<u32>,
) {
    // Saving happens a frame later; refresh the slot list once more.
    if *frames > 0 {
        *frames -= 1;
        if *frames == 0 {
            menu.dirty = true;
        }
    }
    if !menu.dirty {
        return;
    }
    menu.dirty = false;
    if TABS[menu.tab] == Tab::Save && menu.inner.is_some() {
        *frames = 3;
    }
    let (Ok(tab_list), Ok(detail)) = (tab_list.single(), detail.single()) else {
        return;
    };
    commands.entity(tab_list).despawn_children();
    let rows: Vec<Row> = TABS.iter().map(|t| Row::new(content.ui(t.key()))).collect();
    ui::menu_rows(&mut commands, tab_list, &assets, &rows, menu.tab, 19.0);

    commands.entity(detail).despawn_children();
    let tab = TABS[menu.tab];
    let heading = |commands: &mut Commands, text: String| {
        commands.entity(detail).with_child((
            Text::new(text),
            ui::font(&assets, FontKind::Title, 24.0),
            TextColor(ui::GOLD),
        ));
    };
    let line = |commands: &mut Commands, text: String, color: Color, size: f32| {
        commands.entity(detail).with_child((
            Text::new(text),
            ui::font(&assets, FontKind::Body, size),
            TextColor(color),
        ));
    };
    let cursor = menu.inner.unwrap_or(usize::MAX);
    heading(&mut commands, content.ui(tab.key()));
    match tab {
        Tab::Party => {
            for (i, member) in progress.party.iter().enumerate() {
                let Some(def) = content.db.characters.get(&member.id) else {
                    continue;
                };
                let stats = member.stats(def);
                let selected = i == cursor || progress.party.len() == 1;
                let name = content.character_name(&member.id, &progress);
                line(
                    &mut commands,
                    format!(
                        "{} {name} — {} · {}",
                        if i == cursor { "›" } else { " " },
                        realm_text(&content, &progress, member.realm, member.stage),
                        content.text(def.element.key(), &progress)
                    ),
                    if selected { ui::GOLD } else { ui::TEXT },
                    19.0,
                );
                if selected {
                    line(
                        &mut commands,
                        content.format(
                            "ui.party.stats",
                            &progress,
                            &[
                                ("hp", stats.hp.to_string()),
                                ("ll", stats.ll.to_string()),
                                ("atk", stats.atk.to_string()),
                                ("spi", stats.spi.to_string()),
                                ("def", stats.def.to_string()),
                                ("tp", stats.tp.to_string()),
                            ],
                        ),
                        ui::TEXT,
                        16.0,
                    );
                    let skills: Vec<String> = member
                        .skills
                        .iter()
                        .map(|s| content.text(&format!("skill.{s}.name"), &progress))
                        .collect();
                    line(
                        &mut commands,
                        content.format(
                            "ui.party.skills",
                            &progress,
                            &[("list", skills.join(", "))],
                        ),
                        ui::TEXT_DIM,
                        16.0,
                    );
                    let artifacts: Vec<String> = member
                        .artifacts
                        .iter()
                        .map(|a| content.text(&format!("artifact.{a}.name"), &progress))
                        .collect();
                    let list = if artifacts.is_empty() {
                        content.text("ui.common.none", &progress)
                    } else {
                        artifacts.join(", ")
                    };
                    line(
                        &mut commands,
                        content.format(
                            "ui.party.artifacts",
                            &progress,
                            &[
                                ("list", list),
                                ("slots", member.realm.artifact_slots().to_string()),
                            ],
                        ),
                        ui::TEXT_DIM,
                        16.0,
                    );
                    if member.id == crate::story::PLAYER_ID
                        && member.realm != crate::content::defs::Realm::PhamNhan
                    {
                        let thresholds = crate::story::stage_thresholds(member.realm);
                        let next = thresholds.get(member.stage as usize).copied();
                        let text = match next {
                            Some(n) => content.format(
                                "ui.party.tu_vi",
                                &progress,
                                &[("now", member.tu_vi.to_string()), ("next", n.to_string())],
                            ),
                            None => content.text("ui.party.tu_vi_peak", &progress),
                        };
                        line(&mut commands, text, ui::JADE, 16.0);
                    }
                }
            }
            line(
                &mut commands,
                content.format(
                    "ui.party.money",
                    &progress,
                    &[("count", progress.money.to_string())],
                ),
                ui::TEXT,
                16.0,
            );
        }
        Tab::Inventory => {
            let entries = inventory_entries(&content, &progress);
            if entries.is_empty() {
                line(
                    &mut commands,
                    content.text("ui.inventory.empty", &progress),
                    ui::TEXT_DIM,
                    17.0,
                );
            }
            for (i, (id, count)) in entries.iter().enumerate() {
                let def = content.db.items.get(id);
                let name = if content.db.artifacts.contains_key(id) {
                    content.text(&format!("artifact.{id}.name"), &progress)
                } else {
                    content.text(&format!("item.{id}.name"), &progress)
                };
                let category = def
                    .map(|d| content.text(d.category.key(), &progress))
                    .unwrap_or_default();
                let selected = i == cursor;
                line(
                    &mut commands,
                    format!(
                        "{} {name} ×{count}  · {category}",
                        if selected { "›" } else { " " }
                    ),
                    if selected { ui::GOLD } else { ui::TEXT },
                    18.0,
                );
                if selected {
                    let desc = if content.db.artifacts.contains_key(id) {
                        content.text(&format!("artifact.{id}.desc"), &progress)
                    } else {
                        content.text(&format!("item.{id}.desc"), &progress)
                    };
                    line(&mut commands, desc, ui::TEXT_DIM, 16.0);
                    if def.is_some_and(|d| !d.battle_use.is_empty()) {
                        line(
                            &mut commands,
                            content.text("ui.inventory.battle_use", &progress),
                            ui::JADE,
                            15.0,
                        );
                    }
                }
            }
        }
        Tab::Journal => {
            let quests = journal_entries(&content, &progress);
            if quests.is_empty() {
                line(
                    &mut commands,
                    content.text("ui.journal.empty", &progress),
                    ui::TEXT_DIM,
                    17.0,
                );
            }
            for (i, id) in quests.iter().enumerate() {
                let Some(def) = content.db.quests.get(id) else {
                    continue;
                };
                let state = progress.quest(id);
                let status = match state {
                    Some(QuestState::Active) => content.text("ui.journal.active", &progress),
                    Some(QuestState::Done) => content.text("ui.journal.done", &progress),
                    Some(QuestState::Failed) => content.text("ui.journal.failed", &progress),
                    None => String::new(),
                };
                let kind = if def.kind == QuestKind::Main {
                    content.text("ui.journal.main", &progress)
                } else {
                    content.text("ui.journal.side", &progress)
                };
                let selected = i == cursor;
                let active = state == Some(QuestState::Active);
                line(
                    &mut commands,
                    format!(
                        "{} {} · {kind} · {status}",
                        if selected { "›" } else { " " },
                        content.text(&format!("quest.{id}.title"), &progress)
                    ),
                    match (selected, active) {
                        (true, _) => ui::GOLD,
                        (false, true) => ui::TEXT,
                        (false, false) => ui::TEXT_DIM,
                    },
                    18.0,
                );
                if selected {
                    line(
                        &mut commands,
                        content.text(&format!("quest.{id}.desc"), &progress),
                        ui::TEXT_DIM,
                        16.0,
                    );
                    for (objective, done) in progress.objectives(def) {
                        let text =
                            content.text(&format!("quest.{id}.obj.{}", objective.id), &progress);
                        let text = if done {
                            content.format(
                                "ui.journal.objective_done",
                                &progress,
                                &[("text", text)],
                            )
                        } else {
                            format!("• {text}")
                        };
                        line(
                            &mut commands,
                            text,
                            if done { ui::TEXT_DIM } else { ui::TEXT },
                            16.0,
                        );
                    }
                }
            }
        }
        Tab::Save | Tab::Load => {
            let dir = save::save_dir();
            let all = save::list_saves(&dir);
            let shown: Vec<_> = if tab == Tab::Save {
                all.into_iter()
                    .filter(|(s, _)| matches!(s, SaveSlot::Manual(_)))
                    .collect()
            } else {
                all
            };
            let mut rows = slot_rows(&content, &shown);
            if tab == Tab::Save {
                for row in &mut rows {
                    row.enabled = true;
                }
            }
            ui::menu_rows(&mut commands, detail, &assets, &rows, cursor, 18.0);
            line(
                &mut commands,
                content.text(
                    if tab == Tab::Save {
                        "ui.save.hint_save"
                    } else {
                        "ui.save.hint_load"
                    },
                    &progress,
                ),
                ui::TEXT_DIM,
                15.0,
            );
        }
        Tab::Settings => {
            let value = if settings.fast_battle {
                content.text("ui.settings.fast", &progress)
            } else {
                content.text("ui.settings.normal", &progress)
            };
            let volume_row = |key: &str, volume: u8| {
                let bar: String = (0..VOLUME_STEPS)
                    .map(|i| if i < volume { '•' } else { '·' })
                    .collect();
                Row::new(content.format(
                    key,
                    &progress,
                    &[
                        ("bar", bar),
                        ("value", volume.to_string()),
                        ("max", VOLUME_STEPS.to_string()),
                    ],
                ))
            };
            let rows = vec![
                Row::new(content.format(
                    "ui.settings.battle_speed",
                    &progress,
                    &[("value", value)],
                )),
                volume_row("ui.settings.music", settings.music_volume),
                volume_row("ui.settings.sfx", settings.sfx_volume),
            ];
            ui::menu_rows(&mut commands, detail, &assets, &rows, cursor, 18.0);
            line(
                &mut commands,
                content.text("ui.settings.hint", &progress),
                ui::TEXT_DIM,
                15.0,
            );
        }
        Tab::Title => line(
            &mut commands,
            content.text("ui.pause.title_hint", &progress),
            ui::TEXT_DIM,
            16.0,
        ),
        Tab::Quit => line(
            &mut commands,
            content.text("ui.pause.quit_hint", &progress),
            ui::TEXT_DIM,
            16.0,
        ),
    }
}

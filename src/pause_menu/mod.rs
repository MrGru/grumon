//! Pause menu: party, inventory, quest journal, save/load, settings.

mod inventory_tab;
mod party_tab;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    GameState, PlayState,
    asset::GameAssets,
    content::{Content, defs::QuestKind},
    flow::Story,
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
    /// A list opened from a row (actions, targets, artifacts…).
    sub: Option<Sub>,
    /// Result of the last action, shown under the tab.
    message: Option<Message>,
    dirty: bool,
}

/// Second-level lists opened from a tab row.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Sub {
    /// Who to use an item on.
    UseOn { item: String, cursor: usize },
    /// Actions for a party member.
    Member { member: usize, cursor: usize },
    /// Equip or unequip a member's artifacts.
    Equip { member: usize, cursor: usize },
    /// Choose the party formation.
    Formation { cursor: usize },
}

impl Sub {
    fn cursor_mut(&mut self) -> &mut usize {
        match self {
            Sub::UseOn { cursor, .. }
            | Sub::Member { cursor, .. }
            | Sub::Equip { cursor, .. }
            | Sub::Formation { cursor } => cursor,
        }
    }

    fn len(&self, content: &Content, progress: &Progress) -> usize {
        match self {
            Sub::UseOn { .. } => progress.party.len(),
            Sub::Member { .. } => party_tab::MEMBER_ACTIONS.len(),
            Sub::Equip { member, .. } => party_tab::artifact_rows(content, progress, *member).len(),
            Sub::Formation { .. } => party_tab::formation_rows(progress).len(),
        }
    }

    /// Where Esc goes: the artifact list returns to the member's actions.
    fn back(&self) -> Option<Sub> {
        match self {
            Sub::Equip { member, .. } => Some(Sub::Member {
                member: *member,
                cursor: party_tab::MEMBER_ACTIONS.len() - 1,
            }),
            _ => None,
        }
    }
}

/// A locale key with parameters; errors are drawn in red.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Message {
    key: &'static str,
    params: Vec<(&'static str, String)>,
    error: bool,
}

impl Message {
    fn ok(key: &'static str, params: Vec<(&'static str, String)>) -> Self {
        Self {
            key,
            params,
            error: false,
        }
    }

    fn error(key: &'static str, params: Vec<(&'static str, String)>) -> Self {
        Self {
            key,
            params,
            error: true,
        }
    }
}

/// Spawns lines into the detail panel.
struct Detail<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    panel: Entity,
    assets: &'a GameAssets,
}

impl Detail<'_, '_, '_> {
    fn line(&mut self, text: String, color: Color, size: f32) {
        self.commands.entity(self.panel).with_child((
            Text::new(text),
            ui::font(self.assets, FontKind::Body, size),
            TextColor(color),
        ));
    }
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
        sub: None,
        message: None,
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
        Tab::Party => party_tab::len(progress),
        Tab::Inventory => inventory_tab::entries(content, progress).len(),
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
    mut story: Story,
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
    if let Some(mut sub) = menu.sub.clone() {
        if input.cancel || input.left || input.menu {
            input.consumed = true;
            menu.sub = sub.back();
            menu.message = None;
            menu.dirty = true;
            return;
        }
        let n = sub.len(&story.content, &story.progress).max(1);
        let step = input.vertical();
        if step != 0 {
            let cursor = sub.cursor_mut();
            *cursor = (*cursor as i32 + step).rem_euclid(n as i32) as usize;
            menu.sub = Some(sub);
            menu.message = None;
            menu.dirty = true;
        }
        if input.take_confirm() {
            match tab {
                Tab::Inventory => inventory_tab::confirm_use_on(&mut menu, &mut story),
                Tab::Party => party_tab::confirm(&mut menu, &mut story),
                _ => {}
            }
        }
        return;
    }
    let (content, progress) = (&*story.content, &*story.progress);
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
                    _ if inner_len(tab, content, progress) > 0 => {
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
            let n = inner_len(tab, content, progress).max(1);
            let step = input.vertical();
            if step != 0 {
                menu.inner = Some((row as i32 + step).rem_euclid(n as i32) as usize);
                menu.message = None;
                menu.dirty = true;
            }
            if input.take_confirm() {
                match tab {
                    Tab::Party => {
                        party_tab::activate(&mut menu, progress, row);
                        menu.dirty = true;
                    }
                    Tab::Inventory => inventory_tab::activate(&mut menu, &mut story, row),
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
    if let Some(message) = &menu.message {
        let params: Vec<(&str, String)> = message
            .params
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        line(
            &mut commands,
            content.format(message.key, &progress, &params),
            if message.error { ui::DANGER } else { ui::JADE },
            16.0,
        );
    }
    match tab {
        Tab::Party => {
            let mut d = Detail {
                commands: &mut commands,
                panel: detail,
                assets: &assets,
            };
            party_tab::draw(&mut d, &content, &progress, &menu);
        }
        Tab::Inventory => {
            let mut d = Detail {
                commands: &mut commands,
                panel: detail,
                assets: &assets,
            };
            inventory_tab::draw(&mut d, &content, &progress, &menu);
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

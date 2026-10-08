//! Title screen: continue, new journey, load, quit.

use bevy::prelude::*;

use crate::{
    GameState,
    asset::GameAssets,
    content::Content,
    hud::realm_text,
    input::MenuInput,
    save::{self, LoadRequest, SaveSlot, SaveSummary},
    story::Progress,
    ui::{self, FontKind, MenuItem, Row},
};

#[derive(Component)]
struct TitleRoot;

#[derive(Component)]
struct TitleMenuList;

#[derive(Component)]
struct TitleMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Main,
    Load,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainItem {
    Continue,
    NewGame,
    Load,
    Quit,
}

const MAIN_ITEMS: [MainItem; 4] = [
    MainItem::Continue,
    MainItem::NewGame,
    MainItem::Load,
    MainItem::Quit,
];

#[derive(Resource)]
struct TitleMenu {
    view: View,
    cursor: usize,
    saves: Vec<(SaveSlot, Result<SaveSummary, save::SaveError>)>,
    latest: Option<SaveSlot>,
    dirty: bool,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), spawn_title)
            .add_systems(
                Update,
                (title_input, redraw_menu)
                    .chain()
                    .run_if(in_state(GameState::Menu)),
            )
            .add_systems(OnExit(GameState::Menu), |mut commands: Commands| {
                commands.remove_resource::<TitleMenu>();
            });
    }
}

fn spawn_title(mut commands: Commands, assets: Res<GameAssets>, content: Res<Content>) {
    let dir = save::save_dir();
    let saves = save::list_saves(&dir);
    let latest = save::latest_save(&dir);
    commands.insert_resource(TitleMenu {
        view: View::Main,
        cursor: if latest.is_some() { 0 } else { 1 },
        saves,
        latest,
        dirty: true,
    });
    commands.spawn((
        Name::new("Title"),
        TitleRoot,
        DespawnOnExit(GameState::Menu),
        ui::fullscreen(),
        BackgroundColor(ui::INK),
        children![
            (
                ImageNode::new(assets.title_background.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ),
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::top(Val::Px(70.0)),
                    ..default()
                },
                children![
                    (
                        Text::new(content.ui("ui.title.name")),
                        ui::font(&assets, FontKind::Title, 74.0),
                        TextColor(ui::GOLD),
                        TextShadow::default(),
                    ),
                    (
                        Text::new(content.ui("ui.title.subtitle")),
                        ui::font(&assets, FontKind::Title, 38.0),
                        TextColor(ui::JADE),
                        TextShadow::default(),
                    ),
                    (
                        Text::new(content.ui("ui.title.tagline")),
                        ui::font(&assets, FontKind::Body, 17.0),
                        TextColor(ui::TEXT),
                        TextShadow::default(),
                        Node {
                            margin: UiRect::vertical(Val::Px(12.0)),
                            ..default()
                        },
                    ),
                    (
                        TitleMenuList,
                        ui::panel(Node {
                            margin: UiRect::top(Val::Px(26.0)),
                            min_width: Val::Px(360.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        }),
                    ),
                    (
                        TitleMessage,
                        Text::new(""),
                        ui::font(&assets, FontKind::Body, 16.0),
                        TextColor(ui::TEXT_DIM),
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                    ),
                ],
            ),
            (
                Text::new(content.ui("ui.title.footer")),
                ui::font(&assets, FontKind::Body, 14.0),
                TextColor(ui::TEXT_DIM),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(14.0),
                    ..default()
                },
            ),
        ],
    ));
}

fn main_rows(content: &Content, menu: &TitleMenu) -> Vec<Row> {
    MAIN_ITEMS
        .iter()
        .map(|item| match item {
            MainItem::Continue => Row {
                label: content.ui("ui.title.continue"),
                enabled: menu.latest.is_some(),
                detail: None,
            },
            MainItem::NewGame => Row::new(content.ui("ui.title.new_game")),
            MainItem::Load => Row {
                label: content.ui("ui.title.load"),
                enabled: menu.saves.iter().any(|(_, r)| r.is_ok()),
                detail: None,
            },
            MainItem::Quit => Row::new(content.ui("ui.title.quit")),
        })
        .collect()
}

/// Rows for save slots (shared with the pause menu).
pub fn slot_rows(
    content: &Content,
    saves: &[(SaveSlot, Result<SaveSummary, save::SaveError>)],
) -> Vec<Row> {
    saves
        .iter()
        .map(|(slot, result)| {
            let label = match slot {
                SaveSlot::Auto => content.ui("ui.save.slot_auto"),
                SaveSlot::Quick => content.ui("ui.save.slot_quick"),
                SaveSlot::Manual(n) => {
                    content.ui_format("ui.save.slot_manual", &[("n", n.to_string())])
                }
            };
            match result {
                Ok(summary) => {
                    let progress = Progress::default();
                    let detail = content.ui_format(
                        "ui.save.summary",
                        &[
                            ("name", summary.player_name.clone()),
                            ("chapter", summary.chapter.to_string()),
                            ("map", content.ui(&format!("map.{}.name", summary.level))),
                            (
                                "realm",
                                realm_text(content, &progress, summary.realm, summary.stage),
                            ),
                            ("time", ui::format_play_time(summary.play_time)),
                        ],
                    );
                    Row {
                        label,
                        enabled: true,
                        detail: Some(detail),
                    }
                }
                Err(e) => Row {
                    label,
                    enabled: false,
                    detail: Some(content.ui(e.key())),
                },
            }
        })
        .collect()
}

fn title_input(
    mut input: ResMut<MenuInput>,
    mut menu: ResMut<TitleMenu>,
    content: Res<Content>,
    items: Query<(&Interaction, &MenuItem), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut loads: MessageWriter<LoadRequest>,
    mut exit: MessageWriter<AppExit>,
) {
    let rows = match menu.view {
        View::Main => main_rows(&content, &menu),
        View::Load => slot_rows(&content, &menu.saves),
    };
    let n = rows.len();
    let (hovered, clicked) = ui::mouse_menu(&items);
    if let Some(h) = hovered.filter(|&h| h != menu.cursor && rows.get(h).is_some_and(|r| r.enabled))
    {
        menu.cursor = h;
        menu.dirty = true;
    }
    let step = input.vertical();
    if step != 0 {
        // Skip disabled rows (e.g. “Tiếp tục” without any save).
        let mut next = menu.cursor;
        for _ in 0..n {
            next = (next as i32 + step).rem_euclid(n as i32) as usize;
            if rows[next].enabled {
                break;
            }
        }
        menu.cursor = next;
        menu.dirty = true;
    }
    if input.cancel && menu.view == View::Load {
        menu.view = View::Main;
        menu.cursor = 2;
        menu.dirty = true;
        return;
    }
    let activate = clicked.or(input.take_confirm().then_some(menu.cursor));
    let Some(index) = activate else {
        return;
    };
    if !rows.get(index).is_some_and(|r| r.enabled) {
        return;
    }
    match menu.view {
        View::Main => match MAIN_ITEMS[index] {
            MainItem::Continue => {
                if let Some(slot) = menu.latest {
                    loads.write(LoadRequest(slot));
                }
            }
            MainItem::NewGame => next_state.set(GameState::CharacterCreation),
            MainItem::Load => {
                menu.view = View::Load;
                menu.cursor = menu.saves.iter().position(|(_, r)| r.is_ok()).unwrap_or(0);
                menu.dirty = true;
            }
            MainItem::Quit => {
                exit.write(AppExit::Success);
            }
        },
        View::Load => {
            let slot = menu.saves[index].0;
            loads.write(LoadRequest(slot));
        }
    }
}

fn redraw_menu(
    mut commands: Commands,
    assets: Res<GameAssets>,
    content: Res<Content>,
    mut menu: ResMut<TitleMenu>,
    list: Query<Entity, With<TitleMenuList>>,
    mut message: Query<&mut Text, With<TitleMessage>>,
) {
    if !menu.dirty {
        return;
    }
    menu.dirty = false;
    let Ok(list) = list.single() else {
        return;
    };
    commands.entity(list).despawn_children();
    let (rows, size) = match menu.view {
        View::Main => (main_rows(&content, &menu), 24.0),
        View::Load => (slot_rows(&content, &menu.saves), 20.0),
    };
    if menu.view == View::Load {
        commands.entity(list).with_child((
            Text::new(content.ui("ui.title.load_header")),
            ui::font(&assets, FontKind::Bold, 18.0),
            TextColor(ui::JADE),
        ));
    }
    ui::menu_rows(&mut commands, list, &assets, &rows, menu.cursor, size);
    if let Ok(mut text) = message.single_mut() {
        text.0 = if menu.view == View::Load {
            content.ui("ui.common.back_hint")
        } else {
            String::new()
        };
    }
}

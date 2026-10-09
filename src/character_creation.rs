//! New-game character creation: name (Unicode, built-in Telex or the OS input
//! method), how NPCs address the protagonist, appearance, confirmation.

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
    window::{Ime, PrimaryWindow},
};

use crate::{
    GameState,
    asset::GameAssets,
    content::{Content, locale::Addressing},
    flow::PendingStart,
    story::{PlayerProfile, Progress},
    telex,
    ui::{self, FontKind},
};

/// Names used to test font coverage and layout with complex diacritics.
#[cfg(test)]
pub const NAME_TEST_STRING: &str = "Lâm Vô Trần Nguyễn Thị Hường Đặng Quỳnh Ửng Ỷ Ỹ Ợ";

pub const DEFAULT_NAME: &str = "Lâm Vô Trần";
pub const NAME_MIN_CHARS: usize = 2;
pub const NAME_MAX_CHARS: usize = 24;
/// Character sheets offered as appearances.
pub const APPEARANCES: [usize; 4] = [1, 3, 8, 2];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    TooShort,
    TooLong,
    InvalidChar,
}

impl NameError {
    pub fn key(self) -> &'static str {
        match self {
            NameError::TooShort | NameError::TooLong => "ui.create.error_length",
            NameError::InvalidChar => "ui.create.error_chars",
        }
    }
}

fn allowed_char(c: char) -> bool {
    c.is_alphabetic() || c == ' ' || c == '-' || c == '\''
}

/// Trims, collapses inner whitespace and checks length and characters.
pub fn validate_name(raw: &str) -> Result<String, NameError> {
    let name = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if !name.chars().all(allowed_char) {
        return Err(NameError::InvalidChar);
    }
    let len = name.chars().count();
    if len < NAME_MIN_CHARS {
        return Err(NameError::TooShort);
    }
    if len > NAME_MAX_CHARS {
        return Err(NameError::TooLong);
    }
    Ok(name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Name,
    Addressing,
    Appearance,
    Start,
}

const FIELDS: [Field; 4] = [
    Field::Name,
    Field::Addressing,
    Field::Appearance,
    Field::Start,
];

#[derive(Resource)]
struct Creation {
    name: String,
    preedit: String,
    telex: bool,
    addressing: usize,
    appearance: usize,
    focus: usize,
    error: Option<NameError>,
    /// Confirmation dialog open; cursor 0 = agree, 1 = back.
    confirm: Option<usize>,
    dirty: bool,
}

#[derive(Component)]
struct CreationRoot;

#[derive(Component)]
struct CreationBody;

#[derive(Component)]
struct Preview;

#[derive(Component)]
struct PreviewTimer(Timer);

pub struct CharacterCreationPlugin;

impl Plugin for CharacterCreationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::CharacterCreation), spawn_creation)
            .add_systems(
                Update,
                (creation_input, redraw, animate_preview, sync_ime)
                    .chain()
                    .run_if(in_state(GameState::CharacterCreation)),
            )
            .add_systems(OnExit(GameState::CharacterCreation), cleanup);
    }
}

fn spawn_creation(mut commands: Commands, assets: Res<GameAssets>, content: Res<Content>) {
    commands.insert_resource(Creation {
        name: DEFAULT_NAME.to_string(),
        preedit: String::new(),
        telex: true,
        addressing: 0,
        appearance: 0,
        focus: 0,
        error: None,
        confirm: None,
        dirty: true,
    });
    commands.spawn((
        Name::new("CharacterCreation"),
        CreationRoot,
        DespawnOnExit(GameState::CharacterCreation),
        ui::fullscreen(),
        BackgroundColor(ui::INK),
        children![
            (
                ImageNode::new(assets.title_background.clone())
                    .with_color(Color::srgba(1.0, 1.0, 1.0, 0.35)),
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
                    padding: UiRect::top(Val::Px(36.0)),
                    row_gap: Val::Px(18.0),
                    ..default()
                },
                children![
                    (
                        Text::new(content.ui("ui.create.title")),
                        ui::font(&assets, FontKind::Title, 40.0),
                        TextColor(ui::GOLD),
                    ),
                    (
                        Node {
                            column_gap: Val::Px(28.0),
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        children![
                            (
                                ui::panel(Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(250.0),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                }),
                                children![(
                                    Preview,
                                    PreviewTimer(Timer::from_seconds(0.18, TimerMode::Repeating)),
                                    ImageNode::from_atlas_image(
                                        assets.character_image(APPEARANCES[0]),
                                        TextureAtlas {
                                            layout: assets.character_layout.clone(),
                                            index: 0,
                                        },
                                    ),
                                    Node {
                                        width: Val::Px(160.0),
                                        height: Val::Px(160.0),
                                        ..default()
                                    },
                                )],
                            ),
                            (
                                CreationBody,
                                ui::panel(Node {
                                    width: Val::Px(560.0),
                                    min_height: Val::Px(250.0),
                                    padding: UiRect::all(Val::Px(18.0)),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(10.0),
                                    ..default()
                                }),
                            ),
                        ],
                    ),
                    (
                        Text::new(content.ui("ui.create.hints")),
                        ui::font(&assets, FontKind::Body, 15.0),
                        TextColor(ui::TEXT_DIM),
                    ),
                ],
            ),
        ],
    ));
}

fn cleanup(mut commands: Commands, mut window: Query<&mut Window, With<PrimaryWindow>>) {
    commands.remove_resource::<Creation>();
    if let Ok(mut window) = window.single_mut() {
        window.ime_enabled = false;
    }
}

/// OS input method only when Telex is off (avoids double input).
fn sync_ime(creation: Res<Creation>, mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = window.single_mut() else {
        return;
    };
    let wanted =
        !creation.telex && FIELDS[creation.focus] == Field::Name && creation.confirm.is_none();
    if window.ime_enabled != wanted {
        window.ime_enabled = wanted;
        window.ime_position = Vec2::new(520.0, 200.0);
    }
}

fn push_text(creation: &mut Creation, text: &str) {
    for c in text.chars() {
        if creation.name.chars().count() >= NAME_MAX_CHARS {
            break;
        }
        if c.is_ascii_alphabetic() && creation.telex {
            let next = telex::apply_key(&creation.name, c);
            if next.chars().count() <= NAME_MAX_CHARS {
                creation.name = next;
            }
        } else if allowed_char(c) && !(c == ' ' && creation.name.ends_with(' ')) {
            creation.name.push(c);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn creation_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut key_events: MessageReader<KeyboardInput>,
    mut ime_events: MessageReader<Ime>,
    gamepads: Query<&Gamepad>,
    mut creation: ResMut<Creation>,
    content: Res<Content>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let pad = |b: GamepadButton| gamepads.iter().any(|g| g.just_pressed(b));
    let up = keyboard.just_pressed(KeyCode::ArrowUp) || pad(GamepadButton::DPadUp);
    let down = keyboard.just_pressed(KeyCode::ArrowDown) || pad(GamepadButton::DPadDown);
    let left = keyboard.just_pressed(KeyCode::ArrowLeft) || pad(GamepadButton::DPadLeft);
    let right = keyboard.just_pressed(KeyCode::ArrowRight) || pad(GamepadButton::DPadRight);
    let enter = keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter])
        || pad(GamepadButton::South);
    let escape = keyboard.just_pressed(KeyCode::Escape) || pad(GamepadButton::East);
    let field = FIELDS[creation.focus];
    let mut dirty = false;

    // Confirmation dialog.
    if let Some(cursor) = creation.confirm {
        let space = keyboard.just_pressed(KeyCode::Space);
        if left || right || up || down {
            creation.confirm = Some(1 - cursor);
            dirty = true;
        }
        if escape {
            creation.confirm = None;
            dirty = true;
        } else if enter || space {
            if cursor == 0 {
                let Ok(name) = validate_name(&creation.name) else {
                    creation.confirm = None;
                    creation.dirty = true;
                    return;
                };
                let profile = PlayerProfile {
                    name,
                    addressing: Addressing::ALL[creation.addressing],
                    sheet: APPEARANCES[creation.appearance],
                };
                let progress = Progress::new_game(&content.db, profile);
                commands.insert_resource(PendingStart::New(Box::new(progress)));
                next_state.set(GameState::Starting);
            } else {
                creation.confirm = None;
                dirty = true;
            }
        }
        key_events.clear();
        ime_events.clear();
        creation.dirty |= dirty;
        return;
    }

    if escape {
        next_state.set(GameState::Menu);
        return;
    }
    if keyboard.just_pressed(KeyCode::Tab) || pad(GamepadButton::North) {
        creation.telex = !creation.telex;
        creation.preedit.clear();
        dirty = true;
    }
    if up {
        creation.focus = (creation.focus + FIELDS.len() - 1) % FIELDS.len();
        dirty = true;
    }
    if down {
        creation.focus = (creation.focus + 1) % FIELDS.len();
        dirty = true;
    }

    match field {
        Field::Name => {
            for event in key_events.read() {
                if event.state != ButtonState::Pressed {
                    continue;
                }
                match &event.logical_key {
                    Key::Backspace => {
                        creation.name.pop();
                        dirty = true;
                    }
                    Key::Delete => {
                        creation.name.clear();
                        dirty = true;
                    }
                    _ => {
                        if let Some(text) = &event.text
                            && !text.chars().any(char::is_control)
                        {
                            push_text(&mut creation, text);
                            dirty = true;
                        }
                    }
                }
            }
            for event in ime_events.read() {
                match event {
                    Ime::Preedit { value, .. } => {
                        creation.preedit = value.clone();
                        dirty = true;
                    }
                    Ime::Commit { value, .. } => {
                        creation.preedit.clear();
                        push_text(&mut creation, value);
                        dirty = true;
                    }
                    _ => {}
                }
            }
            if dirty {
                creation.error = validate_name(&creation.name).err();
            }
            if enter {
                creation.focus = 1;
                dirty = true;
            }
        }
        Field::Addressing => {
            key_events.clear();
            let n = Addressing::ALL.len();
            if left {
                creation.addressing = (creation.addressing + n - 1) % n;
                dirty = true;
            }
            if right || enter {
                creation.addressing = (creation.addressing + 1) % n;
                dirty = true;
            }
        }
        Field::Appearance => {
            key_events.clear();
            let n = APPEARANCES.len();
            if left {
                creation.appearance = (creation.appearance + n - 1) % n;
                dirty = true;
            }
            if right || enter {
                creation.appearance = (creation.appearance + 1) % n;
                dirty = true;
            }
        }
        Field::Start => {
            key_events.clear();
            if enter || keyboard.just_pressed(KeyCode::Space) {
                match validate_name(&creation.name) {
                    Ok(_) => creation.confirm = Some(0),
                    Err(e) => {
                        creation.error = Some(e);
                        creation.focus = 0;
                    }
                }
                dirty = true;
            }
        }
    }
    creation.dirty |= dirty;
}

fn label_row(assets: &GameAssets, label: String, focused: bool) -> impl Bundle {
    (
        Text::new(format!("{} {label}", if focused { "›" } else { " " })),
        ui::font(assets, FontKind::Bold, 19.0),
        TextColor(if focused { ui::GOLD } else { ui::TEXT }),
    )
}

#[allow(clippy::too_many_arguments)]
fn redraw(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    content: Res<Content>,
    mut creation: ResMut<Creation>,
    body: Query<Entity, With<CreationBody>>,
    mut preview: Query<&mut ImageNode, With<Preview>>,
    mut caret_on: Local<bool>,
) {
    // Blink the caret.
    let blink = ((time.elapsed_secs() * 2.0) as u32).is_multiple_of(2);
    if blink != *caret_on {
        *caret_on = blink;
        creation.dirty = true;
    }
    if !creation.dirty {
        return;
    }
    creation.dirty = false;
    let Ok(body) = body.single() else {
        return;
    };
    if let Ok(mut image) = preview.single_mut() {
        image.image = assets.character_image(APPEARANCES[creation.appearance]);
    }
    commands.entity(body).despawn_children();
    let focus = FIELDS[creation.focus];

    if let Some(cursor) = creation.confirm {
        let addressing = content.ui(Addressing::ALL[creation.addressing].key());
        let name = validate_name(&creation.name).unwrap_or_default();
        commands.entity(body).with_children(|p| {
            p.spawn((
                Text::new(content.ui("ui.create.confirm_title")),
                ui::font(&assets, FontKind::Title, 26.0),
                TextColor(ui::GOLD),
            ));
            p.spawn((
                Text::new(content.ui_format(
                    "ui.create.confirm_body",
                    &[("name", name), ("addressing", addressing)],
                )),
                ui::font(&assets, FontKind::Body, 19.0),
                TextColor(ui::TEXT),
            ));
            for (i, key) in ["ui.common.agree", "ui.common.back"].iter().enumerate() {
                let selected = i == cursor;
                p.spawn((
                    Text::new(format!(
                        "{} {}",
                        if selected { "›" } else { " " },
                        content.ui(key)
                    )),
                    ui::font(
                        &assets,
                        if selected {
                            FontKind::Bold
                        } else {
                            FontKind::Body
                        },
                        20.0,
                    ),
                    TextColor(if selected { ui::GOLD } else { ui::TEXT }),
                ));
            }
        });
        return;
    }

    let caret = if focus == Field::Name && *caret_on {
        "|"
    } else {
        " "
    };
    let ime_label = if creation.telex {
        content.ui("ui.create.ime_telex")
    } else {
        content.ui("ui.create.ime_system")
    };
    let addressing = Addressing::ALL[creation.addressing];
    commands.entity(body).with_children(|p| {
        p.spawn(label_row(
            &assets,
            content.ui("ui.create.name"),
            focus == Field::Name,
        ));
        p.spawn((
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                margin: UiRect::left(Val::Px(18.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                min_width: Val::Px(380.0),
                ..default()
            },
            BackgroundColor(ui::INK),
            BorderColor::all(if focus == Field::Name {
                ui::GOLD
            } else {
                ui::BORDER_DIM
            }),
            children![
                (
                    Text::new(creation.name.clone()),
                    ui::font(&assets, FontKind::Bold, 24.0),
                    TextColor(ui::TEXT),
                ),
                (
                    Text::new(creation.preedit.clone()),
                    ui::font(&assets, FontKind::Body, 24.0),
                    TextColor(ui::JADE),
                ),
                (
                    Text::new(caret),
                    ui::font(&assets, FontKind::Body, 24.0),
                    TextColor(ui::GOLD)
                ),
            ],
        ));
        let (hint, color) = match creation.error {
            Some(e) => (content.ui(e.key()), ui::DANGER),
            None => (ime_label, ui::TEXT_DIM),
        };
        p.spawn((
            Text::new(hint),
            ui::font(&assets, FontKind::Body, 15.0),
            TextColor(color),
            Node {
                margin: UiRect::left(Val::Px(20.0)),
                ..default()
            },
        ));

        p.spawn(label_row(
            &assets,
            content.ui("ui.create.addressing"),
            focus == Field::Addressing,
        ));
        p.spawn((
            Text::new(format!("‹  {}  ›", content.ui(addressing.key()))),
            ui::font(&assets, FontKind::Body, 20.0),
            TextColor(ui::TEXT),
            Node {
                margin: UiRect::left(Val::Px(20.0)),
                ..default()
            },
        ));
        p.spawn((
            Text::new(content.ui(&format!("{}_desc", addressing.key()))),
            ui::font(&assets, FontKind::Body, 15.0),
            TextColor(ui::TEXT_DIM),
            Node {
                margin: UiRect::left(Val::Px(20.0)),
                ..default()
            },
        ));

        p.spawn(label_row(
            &assets,
            content.ui("ui.create.appearance"),
            focus == Field::Appearance,
        ));
        p.spawn((
            Text::new(format!(
                "‹  {}  ›",
                content.ui_format(
                    "ui.create.appearance_n",
                    &[
                        ("n", (creation.appearance + 1).to_string()),
                        ("total", APPEARANCES.len().to_string())
                    ]
                )
            )),
            ui::font(&assets, FontKind::Body, 20.0),
            TextColor(ui::TEXT),
            Node {
                margin: UiRect::left(Val::Px(20.0)),
                ..default()
            },
        ));

        let start_focused = focus == Field::Start;
        p.spawn((
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(if start_focused {
                ui::SELECTED
            } else {
                Color::NONE
            }),
            children![label_row(
                &assets,
                content.ui("ui.create.start"),
                start_focused
            )],
        ));
    });
}

fn animate_preview(time: Res<Time>, mut preview: Query<(&mut ImageNode, &mut PreviewTimer)>) {
    for (mut image, mut timer) in &mut preview {
        timer.0.tick(time.delta());
        if timer.0.just_finished()
            && let Some(atlas) = &mut image.texture_atlas
        {
            atlas.index = (atlas.index + 1) % 4;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_normalises_names() {
        assert_eq!(validate_name("  Lâm   Vô Trần "), Ok("Lâm Vô Trần".into()));
        assert_eq!(
            validate_name("Nguyễn Thị Hường"),
            Ok("Nguyễn Thị Hường".into())
        );
        assert_eq!(validate_name("Mai-Lan O'Neil"), Ok("Mai-Lan O'Neil".into()));
        assert_eq!(validate_name("A"), Err(NameError::TooShort));
        assert_eq!(validate_name("   "), Err(NameError::TooShort));
        assert_eq!(validate_name(&"a".repeat(25)), Err(NameError::TooLong));
        assert_eq!(validate_name("Trần 123"), Err(NameError::InvalidChar));
        assert_eq!(validate_name("Trần!"), Err(NameError::InvalidChar));
        assert!(validate_name(NAME_TEST_STRING).is_err());
    }

    #[test]
    fn typing_with_telex_respects_max_length() {
        let mut c = Creation {
            name: String::new(),
            preedit: String::new(),
            telex: true,
            addressing: 0,
            appearance: 0,
            focus: 0,
            error: None,
            confirm: None,
            dirty: false,
        };
        push_text(&mut c, "Laam Voo Traafn");
        assert_eq!(c.name, "Lâm Vô Trần");
        push_text(&mut c, "  !!123");
        assert_eq!(c.name, "Lâm Vô Trần ");
        push_text(&mut c, &"x".repeat(40));
        assert!(c.name.chars().count() <= NAME_MAX_CHARS);
        c.telex = false;
        c.name.clear();
        push_text(&mut c, "Hoaf");
        assert_eq!(c.name, "Hoaf");
    }
}

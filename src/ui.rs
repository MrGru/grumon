//! Shared UI look: palette, fonts and small node helpers.

use bevy::{prelude::*, text::FontWeight};

use crate::asset::GameAssets;

/// Symbols drawn by code (not from the locale); fonts must cover them too.
#[cfg(test)]
pub const FONT_COVERAGE_EXTRA: &str = "›‹»«•·×…—–%+-/:()[]0123456789";

// Ink-and-jade palette.
pub const INK: Color = Color::srgb(0.035, 0.05, 0.07);
pub const PANEL: Color = Color::srgba(0.05, 0.08, 0.10, 0.94);
pub const BORDER: Color = Color::srgb(0.72, 0.62, 0.38);
pub const BORDER_DIM: Color = Color::srgb(0.32, 0.30, 0.24);
pub const GOLD: Color = Color::srgb(0.96, 0.82, 0.45);
pub const JADE: Color = Color::srgb(0.45, 0.85, 0.72);
pub const TEXT: Color = Color::srgb(0.93, 0.92, 0.88);
pub const TEXT_DIM: Color = Color::srgb(0.62, 0.62, 0.58);
pub const DANGER: Color = Color::srgb(0.92, 0.36, 0.32);
pub const HP_COLOR: Color = Color::srgb(0.80, 0.26, 0.26);
pub const LL_COLOR: Color = Color::srgb(0.30, 0.62, 0.92);
pub const SELECTED: Color = Color::srgba(0.72, 0.62, 0.38, 0.22);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontKind {
    Body,
    Bold,
    Title,
}

pub fn font(assets: &GameAssets, kind: FontKind, size: f32) -> TextFont {
    let (handle, weight) = match kind {
        FontKind::Body => (&assets.font_body, FontWeight::NORMAL),
        FontKind::Bold => (&assets.font_bold, FontWeight::NORMAL),
        FontKind::Title => (&assets.font_title, FontWeight::BOLD),
    };
    TextFont {
        font: handle.clone().into(),
        font_size: FontSize::Px(size),
        weight,
        ..default()
    }
}

/// A bordered panel node.
pub fn panel(node: Node) -> (Node, BackgroundColor, BorderColor) {
    (
        Node {
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..node
        },
        BackgroundColor(PANEL),
        BorderColor::all(BORDER),
    )
}

/// Full-screen root node for a mode's UI.
pub fn fullscreen() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

/// A clickable row of a keyboard-driven menu.
#[derive(Component, Debug, Clone, Copy)]
pub struct MenuItem(pub usize);

/// One row to draw with [`menu_rows`].
pub struct Row {
    pub label: String,
    pub enabled: bool,
    pub detail: Option<String>,
}

impl Row {
    pub fn new(label: String) -> Self {
        Self {
            label,
            enabled: true,
            detail: None,
        }
    }
}

/// Spawns menu rows under `parent`; the selected one gets a cursor and highlight.
pub fn menu_rows(
    commands: &mut Commands,
    parent: Entity,
    assets: &GameAssets,
    rows: &[Row],
    cursor: usize,
    size: f32,
) {
    for (i, row) in rows.iter().enumerate() {
        let selected = i == cursor;
        let color = match (row.enabled, selected) {
            (false, _) => TEXT_DIM.with_alpha(0.55),
            (true, true) => GOLD,
            (true, false) => TEXT,
        };
        let item = commands
            .spawn((
                MenuItem(i),
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(5.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(if selected { SELECTED } else { Color::NONE }),
                children![(
                    Text::new(format!(
                        "{} {}",
                        if selected { "›" } else { " " },
                        row.label
                    )),
                    font(
                        assets,
                        if selected {
                            FontKind::Bold
                        } else {
                            FontKind::Body
                        },
                        size
                    ),
                    TextColor(color),
                )],
            ))
            .id();
        if let Some(detail) = &row.detail {
            commands.entity(item).with_child((
                Text::new(detail.clone()),
                font(assets, FontKind::Body, size * 0.72),
                TextColor(TEXT_DIM),
                Node {
                    margin: UiRect::left(Val::Px(18.0)),
                    ..default()
                },
            ));
        }
        commands.entity(parent).add_child(item);
    }
}

/// Mouse support for menus: (hovered row, clicked row).
pub fn mouse_menu(
    items: &Query<(&Interaction, &MenuItem), Changed<Interaction>>,
) -> (Option<usize>, Option<usize>) {
    let mut hovered = None;
    let mut clicked = None;
    for (interaction, item) in items {
        match interaction {
            Interaction::Pressed => clicked = Some(item.0),
            Interaction::Hovered => hovered = Some(item.0),
            Interaction::None => {}
        }
    }
    (hovered, clicked)
}

/// “1:02:03” style play time.
pub fn format_play_time(secs: u64) -> String {
    format!("{}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60)
}

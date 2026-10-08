use bevy::prelude::*;

pub const BACKGROUND_COLOR: Color = Color::srgb(0.06, 0.08, 0.2);
pub const TITLE_COLOR: Color = Color::srgb(1.0, 0.85, 0.35);
pub const TEXT_COLOR: Color = Color::WHITE;
pub const NORMAL_BUTTON_COLOR: Color = Color::srgb(0.15, 0.18, 0.4);
pub const HOVERED_BUTTON_COLOR: Color = Color::srgb(0.25, 0.3, 0.6);
pub const PRESSED_BUTTON_COLOR: Color = Color::srgb(0.35, 0.45, 0.8);
pub const BORDER_COLOR: Color = Color::srgb(0.95, 0.95, 1.0);

pub fn button_node() -> Node {
    Node {
        width: Val::Px(260.0),
        height: Val::Px(64.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border: UiRect::all(Val::Px(3.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default()
    }
}

use bevy::prelude::*;

use crate::{
    asset::GameAssets,
    main_menu::{
        components::{MainMenu, MenuButton},
        styles::*,
    },
};

pub fn spawn_main_menu(mut commands: Commands, game_assets: Res<GameAssets>) {
    let font = |size: f32| TextFont {
        font: game_assets.grumon_font.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    };
    let button = |kind: MenuButton, label: &str| {
        (
            kind,
            Button,
            button_node(),
            BackgroundColor(NORMAL_BUTTON_COLOR),
            BorderColor::all(BORDER_COLOR),
            children![(Text::new(label), font(24.0), TextColor(TEXT_COLOR))],
        )
    };

    commands.spawn((
        Name::new("MainMenu"),
        MainMenu,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(24.0),
            ..default()
        },
        BackgroundColor(BACKGROUND_COLOR),
        children![
            (Text::new("GRUMON"), font(72.0), TextColor(TITLE_COLOR)),
            (
                Text::new("Press Enter to start"),
                font(18.0),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
            ),
            button(MenuButton::Play, "New Game"),
            button(MenuButton::Quit, "Quit"),
        ],
    ));
}

pub fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

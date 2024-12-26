use bevy::{
    color::palettes::{self, css::BLUE},
    prelude::*,
};

use crate::{
    asset::GameAssets,
    main_menu::{
        components::{MainMenu, PlayButton},
        styles::{BUTTON_STYLE, NORMAL_BUTTON_COLOR},
    },
};

pub fn spawn_main_menu(mut commands: Commands, game_assets: Res<GameAssets>) {
    let main_menu_entity = build_main_menu(&mut commands, game_assets);
}

pub fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    if let Ok(main_menu_entity) = query.get_single() {
        commands.entity(main_menu_entity).despawn_recursive();
    }
}

pub fn build_main_menu(commands: &mut Commands, game_assets: Res<GameAssets>) -> Entity {
    let main_menu_entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            MainMenu {},
        ))
        .with_children(|parent| {
            // Title

            // Play Button
            parent
                .spawn((BUTTON_STYLE, PlayButton {}))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("hello world!"),
                        TextFont {
                            // font: font_handle.clone().into(),
                            font_size: 60.0,
                            ..Default::default()
                        },
                        TextColor(BLUE.into()),
                        TextLayout::new_with_justify(JustifyText::Center),
                    ));
                });
            // Quit Button
        })
        .id();

    main_menu_entity
}

use bevy::{color::palettes, prelude::*};

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
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: palettes::basic::RED.into(),
                ..default()
            },
            MainMenu {},
        ))
        .with_children(|parent| {
            // Title

            // Play Button
            parent
                .spawn((
                    ButtonBundle {
                        style: BUTTON_STYLE,
                        background_color: NORMAL_BUTTON_COLOR.into(),
                        ..default()
                    },
                    PlayButton {},
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text {
                            sections: vec![TextSection::new(
                                "Play",
                                TextStyle {
                                    font: game_assets.grumon_font.clone(),
                                    font_size: 32.0,
                                    color: Color::WHITE,
                                },
                            )],
                            justify: JustifyText::Center,
                            ..default()
                        },
                        ..default()
                    });
                });
            // Quit Button
        })
        .id();

    main_menu_entity
}

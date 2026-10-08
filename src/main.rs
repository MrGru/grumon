use bevy::{prelude::*, window::WindowResolution};
use grumon::{CLEAR, GAME_HEIGHT, GAME_WIDTH, GamePlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Grumon".to_string(),
                        resizable: false,
                        resolution: WindowResolution::new(GAME_WIDTH, GAME_HEIGHT),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(GamePlugin)
        .run();
}

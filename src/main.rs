use bevy::{prelude::*, window::WindowResolution};
use grumon::GamePlugin;

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Grumon".to_string(),
                resizable: false,
                resolution: WindowResolution::new(960.0, 640.0),
                ..default()
            }),
            ..Default::default()
        }))
        .add_plugins(GamePlugin)
        .run();
}

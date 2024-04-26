use bevy::prelude::*;

use crate::GameState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    info!("add camera plugins");
    commands.spawn(Camera2dBundle::default());
}

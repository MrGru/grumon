use bevy::prelude::*;

use crate::{GameState, GAME_HEIGHT, GAME_WIDTH};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    info!("add camera plugins");
    let mut camera = Camera2dBundle::default();
    camera.transform.translation.x += GAME_WIDTH / 2.0;
    camera.transform.translation.y += GAME_HEIGHT / 2.0;

    commands.spawn(camera);
}

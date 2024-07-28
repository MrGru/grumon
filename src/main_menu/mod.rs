mod components;
mod styles;
mod systems;

use bevy::prelude::*;

use crate::GameState;

use self::systems::layout::{despawn_main_menu, spawn_main_menu};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), spawn_main_menu);
        app.add_systems(OnExit(GameState::Menu), despawn_main_menu);
    }
}

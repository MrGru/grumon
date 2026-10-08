mod components;
mod styles;
mod systems;

use bevy::prelude::*;

use crate::GameState;

use self::systems::{
    interactions::{button_feedback, menu_actions, menu_keyboard},
    layout::{despawn_main_menu, spawn_main_menu},
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), spawn_main_menu)
            .add_systems(OnExit(GameState::Menu), despawn_main_menu)
            .add_systems(
                Update,
                (button_feedback, menu_actions, menu_keyboard).run_if(in_state(GameState::Menu)),
            );
    }
}

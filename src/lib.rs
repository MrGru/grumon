use bevy::prelude::*;
use bevy_asset_loader::loading_state::{
    LoadingState, LoadingStateAppExt, config::ConfigureLoadingState,
};

use animation::AnimationPlugin;
use asset::GameAssets;
use camera::CameraPlugin;
use collision::CollisionPlugin;
use debug::DebugPlugin;
use dialogue::DialoguePlugin;
use level::LevelPlugin;
use main_menu::MainMenuPlugin;
use npc::NpcPlugin;
use player::PlayerPlugin;
use transition::TransitionPlugin;
use ysort::YSortPlugin;

mod animation;
mod asset;
mod camera;
mod collision;
mod debug;
mod dialogue;
mod level;
mod main_menu;
mod npc;
mod player;
mod transition;
mod ysort;

pub const CLEAR: Color = Color::srgb(0.1, 0.1, 0.1);
pub const GAME_WIDTH: u32 = 960;
pub const GAME_HEIGHT: u32 = 640;
/// Size of one LDtk grid cell in pixels.
pub const TILE_SIZE: f32 = 16.0;

/// Top-level application flow.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    Playing,
}

/// What the player is doing while in [`GameState::Playing`].
///
/// Gameplay systems gate themselves on one of these instead of checking
/// ad-hoc flags, so e.g. the player can never walk while a dialogue is open.
#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[source(GameState = GameState::Playing)]
pub enum PlayState {
    /// Free movement on the map.
    #[default]
    Exploring,
    /// A dialogue box is open; input drives the conversation.
    Dialogue,
    /// Screen is fading while the next map loads.
    Transition,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<PlayState>()
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Menu)
                    .load_collection::<GameAssets>(),
            )
            .add_plugins((
                CameraPlugin,
                LevelPlugin,
                MainMenuPlugin,
                AnimationPlugin,
                YSortPlugin,
                CollisionPlugin,
                PlayerPlugin,
                NpcPlugin,
                DialoguePlugin,
                TransitionPlugin,
                DebugPlugin,
            ));
    }
}

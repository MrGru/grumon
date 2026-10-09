use bevy::prelude::*;
use bevy_asset_loader::loading_state::{
    LoadingState, LoadingStateAppExt, config::ConfigureLoadingState,
};

mod animation;
mod asset;
mod audio;
pub mod battle;
mod camera;
mod character_creation;
mod collision;
pub mod content;
mod debug;
mod dialogue;
mod flow;
mod hud;
mod input;
mod level;
mod main_menu;
mod map_events;
mod npc;
mod pause_menu;
mod player;
pub mod save;
mod screen_fx;
pub mod story;
#[cfg(test)]
mod story_tests;
pub mod telex;
mod transition;
mod ui;
mod ysort;

/// Window title (also the game's name).
pub const GAME_TITLE: &str = "THIÊN MỆNH: TÀN HỒN";
pub const CLEAR: Color = Color::srgb(0.02, 0.03, 0.04);
pub const GAME_WIDTH: u32 = 960;
pub const GAME_HEIGHT: u32 = 640;
/// Size of one LDtk grid cell in pixels.
pub const TILE_SIZE: f32 = 16.0;

/// Top-level application flow.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    /// Title screen.
    Menu,
    CharacterCreation,
    /// One-frame bridge so a new game or a loaded save re-enters `Playing` cleanly.
    Starting,
    Playing,
}

/// What the player is doing while in [`GameState::Playing`].
///
/// Gameplay systems gate themselves on one of these instead of checking
/// ad-hoc flags, so e.g. the player can never walk while a dialogue is open.
#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[source(GameState = GameState::Playing)]
pub enum PlayState {
    /// Screen is black/fading while a map loads. Every session starts here.
    #[default]
    Transition,
    /// Free movement on the map.
    Exploring,
    /// A dialogue box is open; input drives the conversation.
    Dialogue,
    /// Tactical battle screen.
    Battle,
    /// Pause menu (party, inventory, journal, saves).
    Paused,
    /// Full-screen chapter or story card.
    Card,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(content::ContentPlugin)
            .init_state::<GameState>()
            .add_sub_state::<PlayState>()
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Menu)
                    .load_collection::<asset::GameAssets>()
                    .finally_init_resource::<content::Content>(),
            )
            .add_plugins((
                input::InputPlugin,
                camera::CameraPlugin,
                level::LevelPlugin,
                main_menu::MainMenuPlugin,
                character_creation::CharacterCreationPlugin,
                animation::AnimationPlugin,
                ysort::YSortPlugin,
                collision::CollisionPlugin,
                player::PlayerPlugin,
                npc::NpcPlugin,
                map_events::MapEventsPlugin,
            ))
            .add_plugins((
                flow::FlowPlugin,
                dialogue::DialoguePlugin,
                transition::TransitionPlugin,
                battle::BattlePlugin,
                hud::HudPlugin,
                pause_menu::PauseMenuPlugin,
                save::SavePlugin,
                audio::GameAudioPlugin,
                screen_fx::ScreenFxPlugin,
                debug::DebugPlugin,
            ));
    }
}

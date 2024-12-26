use animation::AnimationPlugin;
use asset::GameAssets;
use bevy::prelude::*;
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_ecs_ldtk::{LdtkPlugin, LevelSelection};
use camera::CameraPlugin;
use debug::DebugPlugin;
use level::LevelPlugin;
use main_menu::MainMenuPlugin;
use player::PlayerPlugin;
use systems::z_order_system::y_sort;

mod animation;
mod asset;
mod camera;
mod components;
mod debug;
mod level;
mod main_menu;
mod player;
mod systems;

pub const CLEAR: Color = Color::srgb(0.1, 0.1, 0.1);
pub const GAME_WIDTH: f32 = 960.0;
pub const GAME_HEIGHT: f32 = 640.0;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Menu,
    Playing,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<GameState>();
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Playing)
                .load_collection::<GameAssets>(),
        );
        app.add_systems(Update, y_sort);
        app.add_plugins(CameraPlugin);
        app.add_plugins(LdtkPlugin);
        app.insert_resource(LevelSelection::index(0));
        app.add_plugins(LevelPlugin);
        app.add_plugins(MainMenuPlugin);
        app.add_plugins(AnimationPlugin);
        app.add_plugins(DebugPlugin);
        app.add_plugins(PlayerPlugin);
    }
}

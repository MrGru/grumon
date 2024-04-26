use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    App::new()
        .init_state::<MyStates>()
        .insert_resource(ClearColor(CLEAR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Grumon".to_string(),
                resizable: false,
                ..default()
            }),
            ..Default::default()
        }))
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>(),
        )
        // .add_systems(Startup, (spawn_camera, draw_the_player))
        .add_systems(OnEnter(MyStates::Next), (spawn_camera, draw_the_player))
        .add_systems(Update, animated_sprite.run_if(in_state(MyStates::Next)))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animated_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            }
        }
    }
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(texture_atlas_layout(
        tile_size_x = 128.0,
        tile_size_y = 128.0,
        columns = 4,
        rows = 4
    ))]
    player_layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler = nearest))]
    #[asset(path = "gfx/characters/player.png")]
    player_texture: Handle<Image>,
}

fn draw_the_player(mut commands: Commands, my_assets: Res<MyAssets>) {
    let animation_indices = AnimationIndices { first: 0, last: 3 };

    commands.spawn((
        SpriteBundle {
            texture: my_assets.player_texture.clone(),
            transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
            ..SpriteBundle::default()
        },
        TextureAtlas {
            layout: my_assets.player_layout.clone(),
            index: animation_indices.first,
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    ));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

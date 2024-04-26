use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(texture_atlas_layout(
        tile_size_x = 128.0,
        tile_size_y = 128.0,
        columns = 4,
        rows = 4
    ))]
    pub player_layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler = nearest))]
    #[asset(path = "gfx/characters/player.png")]
    pub player_texture: Handle<Image>,
}

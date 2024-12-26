use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 4, rows = 4))]
    pub player_layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler(filter = nearest)))]
    #[asset(path = "gfx/characters/ow1.png")]
    pub player_texture: Handle<Image>,
    #[asset(path = "fonts/grumon.ttf")]
    pub grumon_font: Handle<Font>,
}

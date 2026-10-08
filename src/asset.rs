use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

/// Every asset needed before leaving [`crate::GameState::Loading`].
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    /// Shared 4x4 layout of all overworld character sheets (32x32 frames).
    /// Rows: down, left, right, up.
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 4, rows = 4))]
    pub character_layout: Handle<TextureAtlasLayout>,
    /// `characters[n - 1]` is `gfx/characters/ow{n}.png`.
    #[asset(
        paths(
            "gfx/characters/ow1.png",
            "gfx/characters/ow2.png",
            "gfx/characters/ow3.png",
            "gfx/characters/ow4.png",
            "gfx/characters/ow5.png",
            "gfx/characters/ow6.png",
            "gfx/characters/ow7.png",
            "gfx/characters/ow8.png",
            "gfx/characters/ow9.png",
            "gfx/characters/ow10.png"
        ),
        collection(typed)
    )]
    pub characters: Vec<Handle<Image>>,
    #[asset(path = "fonts/grumon.ttf")]
    pub grumon_font: Handle<Font>,
}

impl GameAssets {
    /// Sprite for a character sheet (1-based like the file names), facing down.
    pub fn character_sprite(&self, sheet: usize) -> Sprite {
        let index = sheet.clamp(1, self.characters.len()) - 1;
        Sprite::from_atlas_image(
            self.characters[index].clone(),
            TextureAtlas {
                layout: self.character_layout.clone(),
                index: 0,
            },
        )
    }
}

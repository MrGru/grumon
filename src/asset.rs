use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::content::{DataFileAsset, LocaleFileAsset};

/// Every asset needed before leaving [`crate::GameState::Loading`].
/// Everything is bundled under `assets/`; nothing is fetched at runtime.
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
    #[asset(path = "gfx/tileset/tileset.png")]
    pub tileset: Handle<Image>,
    /// Map object sprites (herbs, kite, grave, shrine…).
    #[asset(path = "gfx/objects/objects.png")]
    pub objects: Handle<Image>,
    /// Battle backgrounds, keyed by path (`gfx/battle/<id>.png`).
    #[asset(path = "gfx/battle", collection(typed, mapped))]
    pub battle_backgrounds: HashMap<String, Handle<Image>>,
    /// Dedicated enemy sprites, keyed by path (`gfx/enemies/<id>.png`).
    #[asset(path = "gfx/enemies", collection(typed, mapped))]
    pub enemy_sprites: HashMap<String, Handle<Image>>,
    /// Music loops, keyed by path (`audio/music/<id>.ogg`).
    #[asset(path = "audio/music", collection(typed, mapped))]
    pub music: HashMap<String, Handle<AudioSource>>,
    /// Sound effects, keyed by path (`audio/sfx/<id>.ogg`).
    #[asset(path = "audio/sfx", collection(typed, mapped))]
    pub sfx: HashMap<String, Handle<AudioSource>>,
    /// Dialogue portraits, keyed by path (`gfx/portraits/<id>.png`).
    #[asset(path = "gfx/portraits", collection(typed, mapped))]
    pub portraits: HashMap<String, Handle<Image>>,
    #[asset(path = "gfx/ui/title.png")]
    pub title_background: Handle<Image>,
    /// Body and UI text (Vietnamese coverage).
    #[asset(path = "fonts/BeVietnamPro-Regular.ttf")]
    pub font_body: Handle<Font>,
    /// Names, headings, buttons.
    #[asset(path = "fonts/BeVietnamPro-SemiBold.ttf")]
    pub font_bold: Handle<Font>,
    /// Calligraphic-style titles (variable font).
    #[asset(path = "fonts/NotoSerifDisplay.ttf")]
    pub font_title: Handle<Font>,
    #[asset(path = "data", collection(typed))]
    pub data: Vec<Handle<DataFileAsset>>,
    #[asset(path = "locale/vi-VN", collection(typed))]
    pub locale: Vec<Handle<LocaleFileAsset>>,
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

    pub fn character_image(&self, sheet: usize) -> Handle<Image> {
        self.characters[sheet.clamp(1, self.characters.len()) - 1].clone()
    }

    pub fn battle_background(&self, id: &str) -> Option<Handle<Image>> {
        self.battle_backgrounds
            .get(&format!("gfx/battle/{id}.png"))
            .cloned()
    }

    /// Portrait for a dialogue speaker: a character id, or `player_<sheet>`.
    pub fn portrait(&self, id: &str) -> Option<Handle<Image>> {
        self.portraits
            .get(&format!("gfx/portraits/{id}.png"))
            .cloned()
    }

    pub fn music_track(&self, id: &str) -> Option<Handle<AudioSource>> {
        self.music.get(&format!("audio/music/{id}.ogg")).cloned()
    }

    pub fn sound(&self, id: &str) -> Option<Handle<AudioSource>> {
        self.sfx.get(&format!("audio/sfx/{id}.ogg")).cloned()
    }

    pub fn enemy_sprite(&self, path: &str) -> Option<Handle<Image>> {
        self.enemy_sprites.get(path).cloned()
    }
}

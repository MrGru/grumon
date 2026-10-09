//! Data-driven content: definitions (`assets/data`), strings (`assets/locale`)
//! and their validation. See `docs/content-schema.md`.

pub mod db;
pub mod defs;
pub mod locale;
pub mod validate;

use std::{collections::HashMap, path::Path};

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};

use self::{
    db::GameDb,
    defs::DataFile,
    locale::{Locale, TextContext},
};
use crate::{asset::GameAssets, story::Progress};

/// One `.data.ron` file.
#[derive(Asset, TypePath, Debug)]
pub struct DataFileAsset(pub DataFile);

/// One `.locale.ron` file: key → text.
#[derive(Asset, TypePath, Debug)]
pub struct LocaleFileAsset(pub HashMap<String, String>);

#[derive(Default, TypePath)]
struct DataFileLoader;

#[derive(Default, TypePath)]
struct LocaleFileLoader;

fn ron_error(path: &Path, e: ron::error::SpannedError) -> std::io::Error {
    std::io::Error::other(format!("{}: {e}", path.display()))
}

impl AssetLoader for DataFileLoader {
    type Asset = DataFileAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<DataFileAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let file =
            ron::de::from_bytes(&bytes).map_err(|e| ron_error(load_context.path().path(), e))?;
        Ok(DataFileAsset(file))
    }

    fn extensions(&self) -> &[&str] {
        &["data.ron"]
    }
}

impl AssetLoader for LocaleFileLoader {
    type Asset = LocaleFileAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<LocaleFileAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let file =
            ron::de::from_bytes(&bytes).map_err(|e| ron_error(load_context.path().path(), e))?;
        Ok(LocaleFileAsset(file))
    }

    fn extensions(&self) -> &[&str] {
        &["locale.ron"]
    }
}

/// All definitions and strings, ready to use. Built once after loading.
#[derive(Resource, Debug)]
pub struct Content {
    pub db: GameDb,
    pub locale: Locale,
}

impl FromWorld for Content {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<GameAssets>();
        let data_assets = world.resource::<Assets<DataFileAsset>>();
        let locale_assets = world.resource::<Assets<LocaleFileAsset>>();
        let files: Vec<DataFile> = assets
            .data
            .iter()
            .filter_map(|h| data_assets.get(h))
            .map(|a| a.0.clone())
            .collect();
        let (db, mut errors) = GameDb::from_files(files);
        let mut locale = Locale::default();
        for handle in &assets.locale {
            if let Some(file) = locale_assets.get(handle) {
                for key in locale.merge(file.0.clone()) {
                    errors.push(format!("duplicate locale key `{key}`"));
                }
            }
        }
        for error in &errors {
            error!("content: {error}");
        }
        info!(
            "content loaded: {} dialogues, {} quests, {} strings",
            db.dialogues.len(),
            db.quests.len(),
            locale.keys().count()
        );
        Content { db, locale }
    }
}

impl Content {
    /// Formats a key for the current player.
    pub fn text(&self, key: &str, progress: &Progress) -> String {
        self.locale.text(key, &text_context(progress))
    }

    pub fn format(&self, key: &str, progress: &Progress, params: &[(&str, String)]) -> String {
        self.locale.format(key, &text_context(progress), params)
    }

    /// Text that does not depend on the player (menus before a game exists).
    pub fn ui(&self, key: &str) -> String {
        self.locale.text(key, &TextContext::default())
    }

    pub fn ui_format(&self, key: &str, params: &[(&str, String)]) -> String {
        self.locale.format(key, &TextContext::default(), params)
    }

    /// Display name of a character ID (`player` resolves to the chosen name).
    pub fn character_name(&self, id: &str, progress: &Progress) -> String {
        if id == crate::story::PLAYER_ID {
            progress.profile.name.clone()
        } else {
            self.text(&format!("char.{id}.name"), progress)
        }
    }
}

pub fn text_context(progress: &Progress) -> TextContext {
    TextContext {
        player_name: progress.profile.name.clone(),
        addressing: progress.profile.addressing,
    }
}

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<DataFileAsset>()
            .init_asset::<LocaleFileAsset>()
            .init_asset_loader::<DataFileLoader>()
            .init_asset_loader::<LocaleFileLoader>();
    }
}

/// Loads content straight from disk (tests and tools).
pub fn load_from_dir(assets: &Path) -> Result<(GameDb, Locale, Vec<String>), String> {
    let mut errors = Vec::new();
    let mut files = Vec::new();
    for path in sorted_files(&assets.join("data"), ".data.ron")? {
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        match ron::from_str::<DataFile>(&text) {
            Ok(file) => files.push(file),
            Err(e) => errors.push(format!("{}: {e}", path.display())),
        }
    }
    let (db, dup_errors) = GameDb::from_files(files);
    errors.extend(dup_errors);
    let mut locale = Locale::default();
    for path in sorted_files(
        &assets.join("locale").join(locale::SOURCE_LOCALE),
        ".locale.ron",
    )? {
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        match ron::from_str::<HashMap<String, String>>(&text) {
            Ok(file) => {
                for key in locale.merge(file) {
                    errors.push(format!("duplicate locale key `{key}`"));
                }
            }
            Err(e) => errors.push(format!("{}: {e}", path.display())),
        }
    }
    Ok((db, locale, errors))
}

fn sorted_files(dir: &Path, suffix: &str) -> Result<Vec<std::path::PathBuf>, String> {
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.to_string_lossy().ends_with(suffix))
        .collect();
    paths.sort();
    Ok(paths)
}

/// IDs referenced from `world.ldtk` (levels and entity `id` fields).
pub fn ldtk_refs(world_ldtk: &Path) -> Result<validate::ExternalRefs, String> {
    let text = std::fs::read_to_string(world_ldtk).map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut refs = validate::ExternalRefs::default();
    for level in json["levels"].as_array().into_iter().flatten() {
        refs.levels
            .push(level["identifier"].as_str().unwrap_or_default().to_string());
        for layer in level["layerInstances"].as_array().into_iter().flatten() {
            for entity in layer["entityInstances"].as_array().into_iter().flatten() {
                let id = entity["fieldInstances"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|f| f["__identifier"] == "id")
                    .and_then(|f| f["__value"].as_str())
                    .map(str::to_string);
                let target = match entity["__identifier"].as_str() {
                    Some("Npc") => &mut refs.npc_ids,
                    Some("Trigger") => &mut refs.trigger_ids,
                    Some("Object") => &mut refs.object_ids,
                    _ => continue,
                };
                match id {
                    Some(id) => target.push(id),
                    None => target.push(format!(
                        "<missing id at {}>",
                        entity["iid"].as_str().unwrap_or("?")
                    )),
                }
            }
        }
    }
    Ok(refs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assets_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
    }

    fn backgrounds() -> Vec<String> {
        sorted_files(&assets_dir().join("gfx/battle"), ".png")
            .unwrap_or_default()
            .iter()
            .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
            .collect()
    }

    /// The full content validator over the shipped assets.
    #[test]
    fn shipped_content_is_valid() {
        let (db, locale, mut errors) = load_from_dir(&assets_dir()).expect("assets readable");
        // Guard against the validator silently checking nothing.
        assert!(
            db.dialogues.len() >= 50,
            "only {} dialogues loaded",
            db.dialogues.len()
        );
        assert!(db.quests.len() >= 8 && db.encounters.len() >= 5 && db.chapters.contains_key(&1));
        assert!(
            locale.keys().count() >= 600,
            "only {} strings",
            locale.keys().count()
        );
        let mut refs = ldtk_refs(&assets_dir().join("world.ldtk")).expect("world.ldtk readable");
        assert!(
            refs.npc_ids.len() >= 20 && refs.trigger_ids.len() >= 9 && refs.object_ids.len() >= 19
        );
        refs.backgrounds = backgrounds();
        refs.music = sorted_files(&assets_dir().join("audio/music"), ".ogg")
            .unwrap_or_default()
            .iter()
            .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
            .collect();
        assert!(refs.music.len() >= 8, "music tracks missing");
        errors.extend(validate::validate(&db, &locale, &refs));
        assert!(
            errors.is_empty(),
            "{} content errors:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }

    /// Every character of every string has a glyph in the bundled fonts.
    /// Every speaker has a portrait, and so does every protagonist look.
    #[test]
    fn every_speaker_has_a_portrait() {
        use crate::content::defs::DialogueNode;
        // Voices nobody can see.
        const NO_PORTRAIT: [&str; 1] = ["giong_noi"];
        let (db, _, _) = load_from_dir(&assets_dir()).expect("assets readable");
        let dir = assets_dir().join("gfx/portraits");
        let mut missing = std::collections::BTreeSet::new();
        for dialogue in db.dialogues.values() {
            for node in dialogue.nodes.values() {
                let speaker = match node {
                    DialogueNode::Line { speaker, .. } | DialogueNode::Choice { speaker, .. } => {
                        speaker.as_deref()
                    }
                    _ => None,
                };
                if let Some(id) = speaker
                    && id != crate::story::PLAYER_ID
                    && !NO_PORTRAIT.contains(&id)
                    && !dir.join(format!("{id}.png")).exists()
                {
                    missing.insert(id.to_string());
                }
            }
        }
        for sheet in crate::character_creation::APPEARANCES {
            if !dir.join(format!("player_{sheet}.png")).exists() {
                missing.insert(format!("player_{sheet}"));
            }
        }
        assert!(missing.is_empty(), "portraits missing: {missing:?}");
    }

    #[test]
    fn fonts_cover_every_character() {
        let (_, locale, _) = load_from_dir(&assets_dir()).expect("assets readable");
        let mut extra: String = crate::ui::FONT_COVERAGE_EXTRA.to_string();
        extra.push_str(crate::character_creation::NAME_TEST_STRING);
        for font in [
            "fonts/BeVietnamPro-Regular.ttf",
            "fonts/BeVietnamPro-SemiBold.ttf",
            "fonts/NotoSerifDisplay.ttf",
        ] {
            let data = std::fs::read(assets_dir().join(font)).expect("font file");
            let face = ttf_parser::Face::parse(&data, 0).expect("valid font");
            let mut missing: Vec<char> = locale
                .entries()
                .flat_map(|(_, v)| v.chars())
                .chain(extra.chars())
                .filter(|c| !c.is_control() && face.glyph_index(*c).is_none())
                .collect();
            missing.sort();
            missing.dedup();
            assert!(missing.is_empty(), "{font} is missing glyphs: {missing:?}");
        }
    }

    /// Every UI key literal used in the source code exists in the locale.
    #[test]
    fn code_keys_exist() {
        let (_, locale, _) = load_from_dir(&assets_dir()).expect("assets readable");
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut missing = Vec::new();
        let mut stack = vec![src];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("src readable").flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let text = std::fs::read_to_string(&path).expect("source readable");
                // Test modules use made-up keys.
                let text = text.split("#[cfg(test)]").next().unwrap_or_default();
                for literal in text.split('"').skip(1).step_by(2) {
                    let is_key = [
                        "ui.",
                        "log.",
                        "notice.",
                        "reason.",
                        "intent.",
                        "objective.",
                        "realm.",
                        "stage.",
                        "element.",
                        "slot.",
                        "rarity.",
                    ]
                    .iter()
                    .any(|ns| literal.starts_with(ns))
                        && literal.chars().all(|c| {
                            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_'
                        })
                        && !literal.ends_with('.');
                    if is_key && !locale.has(literal) {
                        missing.push(format!("{}: {literal}", path.display()));
                    }
                }
            }
        }
        missing.sort();
        missing.dedup();
        assert!(
            missing.is_empty(),
            "missing UI keys:\n{}",
            missing.join("\n")
        );
    }
}

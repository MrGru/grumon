//! Local save files: versioned JSON, atomic writes with a backup, migrations
//! (content-schema §7). Saving works while exploring, in dialogue and in battle.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    GameState,
    battle::{BattleSession, core::BattleState},
    content::defs::Realm,
    dialogue::DialogueSession,
    flow::{Notices, PendingStart},
    story::{Notice, Progress},
};

/// Current save format version. Bump and add a step to [`migrate`] on breaking changes.
pub const SAVE_VERSION: u32 = 1;
pub const MANUAL_SLOTS: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SaveSlot {
    Manual(u8),
    Auto,
    Quick,
}

impl SaveSlot {
    pub fn file_stem(self) -> String {
        match self {
            SaveSlot::Manual(n) => format!("slot{n}"),
            SaveSlot::Auto => "auto".into(),
            SaveSlot::Quick => "quick".into(),
        }
    }

    /// Every slot, in menu order.
    pub fn all() -> Vec<SaveSlot> {
        let mut slots = vec![SaveSlot::Auto, SaveSlot::Quick];
        slots.extend((1..=MANUAL_SLOTS).map(SaveSlot::Manual));
        slots
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveSummary {
    pub player_name: String,
    pub chapter: u8,
    pub level: String,
    pub play_time: u64,
    pub realm: Realm,
    pub stage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DialogueSnapshot {
    pub dialogue: String,
    pub node: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveFile {
    pub version: u32,
    pub saved_at: u64,
    pub summary: SaveSummary,
    pub progress: Progress,
    #[serde(default)]
    pub dialogue: Option<DialogueSnapshot>,
    #[serde(default)]
    pub battle: Option<Box<BattleState>>,
}

impl SaveFile {
    pub fn new(
        progress: &Progress,
        dialogue: Option<DialogueSnapshot>,
        battle: Option<BattleState>,
    ) -> Self {
        let (realm, stage) = progress
            .player()
            .map_or((Realm::PhamNhan, 0), |m| (m.realm, m.stage));
        SaveFile {
            version: SAVE_VERSION,
            saved_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
            summary: SaveSummary {
                player_name: progress.profile.name.clone(),
                chapter: progress.chapter,
                level: progress.level.clone(),
                play_time: progress.play_time as u64,
                realm,
                stage,
            },
            progress: progress.clone(),
            dialogue,
            battle: battle.map(Box::new),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    Io(String),
    Corrupt(String),
    TooNew(u32),
    Missing,
}

impl SaveError {
    /// Locale key for a player-facing message.
    pub fn key(&self) -> &'static str {
        match self {
            SaveError::Io(_) => "ui.save.error_io",
            SaveError::Corrupt(_) => "ui.save.error_corrupt",
            SaveError::TooNew(_) => "ui.save.error_too_new",
            SaveError::Missing => "ui.save.empty",
        }
    }
}

/// Directory holding save files.
pub fn save_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("THIEN_MENH_SAVE_DIR") {
        return PathBuf::from(dir);
    }
    let base = if cfg!(target_os = "windows") {
        std::env::var("APPDATA").map(PathBuf::from).ok()
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support"))
            .ok()
    } else {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .ok()
            .or_else(|| {
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join(".local/share"))
                    .ok()
            })
    };
    base.unwrap_or_else(|| PathBuf::from("."))
        .join("thien-menh-tan-hon")
        .join("saves")
}

fn slot_path(dir: &Path, slot: SaveSlot) -> PathBuf {
    dir.join(format!("{}.json", slot.file_stem()))
}

/// Upgrades an older save to [`SAVE_VERSION`], one step at a time.
pub fn migrate(mut value: Value) -> Result<Value, SaveError> {
    let mut version = value["version"].as_u64().unwrap_or(0) as u32;
    if version > SAVE_VERSION {
        return Err(SaveError::TooNew(version));
    }
    while version < SAVE_VERSION {
        match version {
            // v0 (pre-release test builds) stored flags as `story_flags`.
            0 => {
                if let Some(progress) = value.get_mut("progress").and_then(Value::as_object_mut)
                    && let Some(flags) = progress.remove("story_flags")
                {
                    progress.insert("flags".into(), flags);
                }
            }
            _ => unreachable!("missing migration step"),
        }
        version += 1;
        value["version"] = Value::from(version);
    }
    Ok(value)
}

fn parse(text: &str) -> Result<SaveFile, SaveError> {
    let value: Value = serde_json::from_str(text).map_err(|e| SaveError::Corrupt(e.to_string()))?;
    let value = migrate(value)?;
    serde_json::from_value(value).map_err(|e| SaveError::Corrupt(e.to_string()))
}

/// Writes a save atomically, keeping the previous file as `.bak`.
pub fn write_save(dir: &Path, slot: SaveSlot, save: &SaveFile) -> Result<(), SaveError> {
    let io = |e: std::io::Error| SaveError::Io(e.to_string());
    fs::create_dir_all(dir).map_err(io)?;
    let path = slot_path(dir, slot);
    let tmp = path.with_extension("json.tmp");
    let bak = path.with_extension("json.bak");
    let json = serde_json::to_string_pretty(save).map_err(|e| SaveError::Io(e.to_string()))?;
    {
        let mut file = fs::File::create(&tmp).map_err(io)?;
        file.write_all(json.as_bytes()).map_err(io)?;
        file.sync_all().map_err(io)?;
    }
    if path.exists() {
        fs::rename(&path, &bak).map_err(io)?;
    }
    fs::rename(&tmp, &path).map_err(io)?;
    Ok(())
}

/// Reads a save, falling back to the backup when the main file is missing or corrupt.
pub fn read_save(dir: &Path, slot: SaveSlot) -> Result<SaveFile, SaveError> {
    let path = slot_path(dir, slot);
    let main = match fs::read_to_string(&path) {
        Ok(text) => parse(&text),
        Err(_) => Err(SaveError::Missing),
    };
    match main {
        Ok(save) => Ok(save),
        Err(SaveError::TooNew(v)) => Err(SaveError::TooNew(v)),
        Err(first) => match fs::read_to_string(path.with_extension("json.bak")) {
            Ok(text) => parse(&text).map_err(|_| first),
            Err(_) => Err(first),
        },
    }
}

/// Summaries for every slot (menus).
pub fn list_saves(dir: &Path) -> Vec<(SaveSlot, Result<SaveSummary, SaveError>)> {
    SaveSlot::all()
        .into_iter()
        .map(|slot| (slot, read_save(dir, slot).map(|s| s.summary)))
        .collect()
}

pub fn any_save_exists(dir: &Path) -> bool {
    list_saves(dir).iter().any(|(_, r)| r.is_ok())
}

/// The most recently written readable save (title screen “Tiếp tục”).
pub fn latest_save(dir: &Path) -> Option<SaveSlot> {
    SaveSlot::all()
        .into_iter()
        .filter_map(|slot| read_save(dir, slot).ok().map(|s| (slot, s.saved_at)))
        .max_by_key(|(_, at)| *at)
        .map(|(slot, _)| slot)
}

#[derive(Message, Debug, Clone, Copy)]
pub struct SaveRequest(pub SaveSlot);

#[derive(Message, Debug, Clone, Copy)]
pub struct LoadRequest(pub SaveSlot);

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveRequest>()
            .add_message::<LoadRequest>()
            .add_systems(
                Update,
                (
                    handle_saves.run_if(in_state(GameState::Playing)),
                    handle_loads,
                ),
            );
    }
}

fn handle_saves(
    mut requests: MessageReader<SaveRequest>,
    progress: Option<Res<Progress>>,
    dialogue: Option<Res<DialogueSession>>,
    battle: Option<Res<BattleSession>>,
    mut notices: ResMut<Notices>,
) {
    for SaveRequest(slot) in requests.read() {
        let Some(progress) = &progress else {
            continue;
        };
        let save = SaveFile::new(
            progress,
            dialogue.as_ref().and_then(|d| d.snapshot()),
            battle.as_ref().and_then(|b| b.snapshot()),
        );
        let key = match write_save(&save_dir(), *slot, &save) {
            Ok(()) => match slot {
                SaveSlot::Auto => "notice.autosaved",
                SaveSlot::Quick => "notice.quick_saved",
                SaveSlot::Manual(_) => "notice.saved",
            },
            Err(e) => {
                error!("saving failed: {e:?}");
                "notice.save_failed"
            }
        };
        notices.0.push_back(Notice::Custom(key.into()));
    }
}

fn handle_loads(
    mut commands: Commands,
    mut requests: MessageReader<LoadRequest>,
    mut next_state: ResMut<NextState<GameState>>,
    notices: Option<ResMut<Notices>>,
) {
    let mut notices = notices;
    for LoadRequest(slot) in requests.read() {
        match read_save(&save_dir(), *slot) {
            Ok(save) => {
                commands.insert_resource(PendingStart::Load(Box::new(save)));
                next_state.set(GameState::Starting);
            }
            Err(e) => {
                warn!("loading {slot:?} failed: {e:?}");
                if let Some(n) = &mut notices {
                    n.0.push_back(Notice::Custom("notice.load_failed".into()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::PlayerProfile;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("tmth-save-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn sample() -> SaveFile {
        let mut progress = Progress {
            profile: PlayerProfile {
                name: "Nguyễn Thị Hường".into(),
                ..Default::default()
            },
            ..Default::default()
        };
        progress.set_flag("ch1.promise", 2);
        progress.level = "Forest".into();
        SaveFile::new(
            &progress,
            Some(DialogueSnapshot {
                dialogue: "ch1_lien_dusk".into(),
                node: "ask".into(),
            }),
            None,
        )
    }

    #[test]
    fn roundtrip_keeps_unicode_names_and_dialogue() {
        let dir = temp_dir("roundtrip");
        let save = sample();
        write_save(&dir, SaveSlot::Manual(1), &save).expect("write");
        let loaded = read_save(&dir, SaveSlot::Manual(1)).expect("read");
        assert_eq!(loaded, save);
        assert_eq!(loaded.summary.player_name, "Nguyễn Thị Hường");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_main_file_falls_back_to_backup() {
        let dir = temp_dir("backup");
        let first = sample();
        write_save(&dir, SaveSlot::Auto, &first).expect("write 1");
        let mut second = sample();
        second.progress.set_flag("x", 1);
        write_save(&dir, SaveSlot::Auto, &second).expect("write 2");
        // Simulate a crash that left a truncated main file.
        fs::write(slot_path(&dir, SaveSlot::Auto), "{\"version\": 1, \"sav").expect("truncate");
        let loaded = read_save(&dir, SaveSlot::Auto).expect("backup");
        assert_eq!(loaded.progress.flag("x"), 0, "previous save restored");
        assert!(read_save(&dir, SaveSlot::Manual(3)).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn migrates_old_versions_and_refuses_future_ones() {
        let mut value = serde_json::to_value(sample()).expect("value");
        value["version"] = Value::from(0);
        let flags = value["progress"]["flags"].take();
        value["progress"]
            .as_object_mut()
            .expect("object")
            .remove("flags");
        value["progress"]["story_flags"] = flags;
        let save = parse(&value.to_string()).expect("migrated");
        assert_eq!(save.version, SAVE_VERSION);
        assert_eq!(save.progress.flag("ch1.promise"), 2);

        value["version"] = Value::from(SAVE_VERSION + 1);
        assert_eq!(
            parse(&value.to_string()),
            Err(SaveError::TooNew(SAVE_VERSION + 1))
        );
    }
}

//! All definitions indexed by ID.

use std::collections::BTreeMap;

use super::defs::*;

/// Every definition from every data file, indexed by ID.
#[derive(Debug, Clone, Default)]
pub struct GameDb {
    pub items: BTreeMap<String, ItemDef>,
    pub skills: BTreeMap<String, SkillDef>,
    pub artifacts: BTreeMap<String, ArtifactDef>,
    pub formations: BTreeMap<String, FormationDef>,
    pub enemies: BTreeMap<String, EnemyDef>,
    pub encounters: BTreeMap<String, EncounterDef>,
    pub characters: BTreeMap<String, CharacterDef>,
    pub npcs: BTreeMap<String, NpcDef>,
    pub dialogues: BTreeMap<String, DialogueDef>,
    pub quests: BTreeMap<String, QuestDef>,
    pub triggers: BTreeMap<String, TriggerDef>,
    pub objects: BTreeMap<String, ObjectDef>,
    pub chapters: BTreeMap<u8, ChapterDef>,
    pub levels: BTreeMap<String, LevelDef>,
    pub summons: BTreeMap<String, SummonDef>,
    pub shops: BTreeMap<String, ShopDef>,
    pub recipes: BTreeMap<String, RecipeDef>,
}

fn insert_all<T>(
    kind: &str,
    target: &mut BTreeMap<String, T>,
    defs: Vec<T>,
    id: impl Fn(&T) -> &str,
    errors: &mut Vec<String>,
) {
    for def in defs {
        let key = id(&def).to_string();
        if target.contains_key(&key) {
            errors.push(format!("duplicate {kind} id `{key}`"));
        }
        target.insert(key, def);
    }
}

impl GameDb {
    /// Merges data files. Returns the database and duplicate-ID errors.
    pub fn from_files(files: impl IntoIterator<Item = DataFile>) -> (Self, Vec<String>) {
        let mut db = GameDb::default();
        let mut errors = Vec::new();
        for file in files {
            insert_all("item", &mut db.items, file.items, |d| &d.id, &mut errors);
            insert_all("skill", &mut db.skills, file.skills, |d| &d.id, &mut errors);
            insert_all(
                "artifact",
                &mut db.artifacts,
                file.artifacts,
                |d| &d.id,
                &mut errors,
            );
            insert_all(
                "formation",
                &mut db.formations,
                file.formations,
                |d| &d.id,
                &mut errors,
            );
            insert_all(
                "enemy",
                &mut db.enemies,
                file.enemies,
                |d| &d.id,
                &mut errors,
            );
            insert_all(
                "encounter",
                &mut db.encounters,
                file.encounters,
                |d| &d.id,
                &mut errors,
            );
            insert_all(
                "character",
                &mut db.characters,
                file.characters,
                |d| &d.id,
                &mut errors,
            );
            insert_all("npc", &mut db.npcs, file.npcs, |d| &d.id, &mut errors);
            insert_all(
                "dialogue",
                &mut db.dialogues,
                file.dialogues,
                |d| &d.id,
                &mut errors,
            );
            insert_all("quest", &mut db.quests, file.quests, |d| &d.id, &mut errors);
            insert_all(
                "trigger",
                &mut db.triggers,
                file.triggers,
                |d| &d.id,
                &mut errors,
            );
            insert_all(
                "object",
                &mut db.objects,
                file.objects,
                |d| &d.id,
                &mut errors,
            );
            insert_all("level", &mut db.levels, file.levels, |d| &d.id, &mut errors);
            insert_all("shop", &mut db.shops, file.shops, |d| &d.id, &mut errors);
            insert_all(
                "recipe",
                &mut db.recipes,
                file.recipes,
                |d| &d.id,
                &mut errors,
            );
            insert_all(
                "summon",
                &mut db.summons,
                file.summons,
                |d| &d.id,
                &mut errors,
            );
            for chapter in file.chapters {
                if db.chapters.contains_key(&chapter.number) {
                    errors.push(format!("duplicate chapter {}", chapter.number));
                }
                db.chapters.insert(chapter.number, chapter);
            }
        }
        (db, errors)
    }

    pub fn skill(&self, id: &str) -> Option<&SkillDef> {
        self.skills.get(id)
    }
}

//! Persistent story state (flags, inventory, quests, party) and the pure logic
//! that evaluates conditions and applies effects. No ECS here: everything is
//! unit tested and serialised into save files as-is.

use std::collections::BTreeMap;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::content::{db::GameDb, defs::*, locale::Addressing};

/// ID of the protagonist in party and character data.
pub const PLAYER_ID: &str = "player";

/// Tu vi needed to reach each minor stage of a realm, cumulative within the
/// realm (game-systems §9.1). Index 0 = to reach trung kỳ.
pub fn stage_thresholds(realm: Realm) -> [u32; 3] {
    match realm {
        Realm::PhamNhan => [u32::MAX; 3],
        Realm::LuyenKhi => [60, 180, 380],
        Realm::TrucCo => [300, 750, 1350],
        Realm::KetDan => [800, 1800, 3100],
        Realm::NguyenAnh => [1600, 3600, 6100],
        Realm::HoaThan => [3000, 6600, 10900],
    }
}

/// Number of stages reached in total, used for stat growth (Luyện Khí sơ kỳ = 1).
pub fn stage_index(realm: Realm, stage: u8) -> u32 {
    match realm {
        Realm::PhamNhan => 0,
        _ => (realm as u32 - 1) * 4 + stage as u32 + 1,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerProfile {
    pub name: String,
    pub addressing: Addressing,
    /// Character sheet number (`ow{n}.png`).
    pub sheet: usize,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            name: "Lâm Vô Trần".to_string(),
            addressing: Addressing::Male,
            sheet: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuestState {
    Active,
    Done,
    Failed,
}

/// A party member's growth state. Base stats come from `CharacterDef`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Member {
    pub id: String,
    pub realm: Realm,
    pub stage: u8,
    pub tu_vi: u32,
    pub skills: Vec<String>,
    /// Equipped artifacts (limited by realm).
    pub artifacts: Vec<String>,
    pub slot: Slot,
}

impl Member {
    pub fn new(def: &CharacterDef) -> Self {
        Self {
            id: def.id.clone(),
            realm: Realm::PhamNhan,
            stage: 0,
            tu_vi: 0,
            skills: def.skills.clone(),
            artifacts: def.artifacts.clone(),
            slot: def.slot,
        }
    }

    pub fn stats(&self, def: &CharacterDef) -> Stats {
        def.base
            .plus_scaled(def.growth, stage_index(self.realm, self.stage))
    }
}

/// Deferred action produced by an effect; executed by the ECS layer in order.
#[derive(Debug, Clone, PartialEq)]
pub enum Deferred {
    Dialogue(String),
    Battle(String),
    Warp { level: String, x: i32, y: i32 },
    Card(String),
    Autosave,
    Shop(String),
    Craft(Station),
}

/// Something worth telling the player about (rendered by the HUD).
#[derive(Debug, Clone, PartialEq)]
pub enum Notice {
    ItemGained(String, u32),
    ItemLost(String, u32),
    ItemUsed(String),
    Money(i64),
    QuestStarted(String),
    QuestDone(String),
    QuestFailed(String),
    Joined(String),
    Left(String),
    SkillLearned(String, String),
    ArtifactGained(String),
    TuVi(u32),
    StageUp(String, Realm, u8),
    Breakthrough(String, Realm),
    FormationLearned(String),
    RecipeLearned(String),
    Custom(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Outcome {
    pub deferred: Vec<Deferred>,
    pub notices: Vec<Notice>,
}

impl Outcome {
    pub fn extend(&mut self, other: Outcome) {
        self.deferred.extend(other.deferred);
        self.notices.extend(other.notices);
    }
}

/// Everything that persists between sessions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Resource)]
pub struct Progress {
    pub profile: PlayerProfile,
    pub flags: BTreeMap<String, i32>,
    pub items: BTreeMap<String, u32>,
    pub money: i64,
    pub quests: BTreeMap<String, QuestState>,
    /// Quest IDs in the order they started (journal order).
    pub quest_order: Vec<String>,
    pub party: Vec<Member>,
    pub formation: Option<String>,
    pub chapter: u8,
    pub time: TimeOfDay,
    /// Current LDtk level and the player's feet position in LDtk pixels.
    pub level: String,
    pub feet: (i32, i32),
    pub play_time: f64,
    pub battles_fought: u32,
    /// Story music overriding the map's track (`StoryEffect::Music`).
    #[serde(default)]
    pub music: Option<String>,
    /// Formations the party can choose in the pause menu.
    #[serde(default)]
    pub formations_known: Vec<String>,
    /// Luyện đan recipes the player can brew.
    #[serde(default)]
    pub recipes_known: Vec<String>,
    /// Units bought from limited shop stock, keyed `shop/item`.
    #[serde(default)]
    pub shop_bought: BTreeMap<String, u32>,
    /// Luyện khí level of each artifact (by id; artifacts are unique).
    #[serde(default)]
    pub artifact_levels: BTreeMap<String, u8>,
    /// Luyện đan experience.
    #[serde(default)]
    pub alchemy_xp: u32,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            profile: PlayerProfile::default(),
            flags: BTreeMap::new(),
            items: BTreeMap::new(),
            money: 0,
            quests: BTreeMap::new(),
            quest_order: Vec::new(),
            party: Vec::new(),
            formation: None,
            chapter: 1,
            time: TimeOfDay::Day,
            level: String::new(),
            feet: (0, 0),
            play_time: 0.0,
            battles_fought: 0,
            music: None,
            formations_known: Vec::new(),
            recipes_known: Vec::new(),
            shop_bought: BTreeMap::new(),
            artifact_levels: BTreeMap::new(),
            alchemy_xp: 0,
        }
    }
}

impl Progress {
    /// A fresh game with the protagonist in the party.
    pub fn new_game(db: &GameDb, profile: PlayerProfile) -> Self {
        let mut progress = Progress {
            profile,
            ..Default::default()
        };
        if let Some(def) = db.characters.get(PLAYER_ID) {
            progress.party.push(Member::new(def));
        }
        progress
    }

    pub fn flag(&self, name: &str) -> i32 {
        self.flags.get(name).copied().unwrap_or(0)
    }

    pub fn set_flag(&mut self, name: &str, value: i32) {
        if value == 0 {
            self.flags.remove(name);
        } else {
            self.flags.insert(name.to_string(), value);
        }
    }

    pub fn item_count(&self, id: &str) -> u32 {
        self.items.get(id).copied().unwrap_or(0)
    }

    pub fn member(&self, id: &str) -> Option<&Member> {
        self.party.iter().find(|m| m.id == id)
    }

    pub fn member_mut(&mut self, id: &str) -> Option<&mut Member> {
        self.party.iter_mut().find(|m| m.id == id)
    }

    pub fn player(&self) -> Option<&Member> {
        self.member(PLAYER_ID)
    }

    pub fn quest(&self, id: &str) -> Option<QuestState> {
        self.quests.get(id).copied()
    }

    pub fn eval(&self, condition: &Condition) -> bool {
        match condition {
            Condition::Flag(name) => self.flag(name) != 0,
            Condition::NotFlag(name) => self.flag(name) == 0,
            Condition::FlagAtLeast(name, n) => self.flag(name) >= *n,
            Condition::FlagEquals(name, n) => self.flag(name) == *n,
            Condition::HasItem(id, n) => self.item_count(id) >= *n,
            Condition::QuestActive(id) => self.quest(id) == Some(QuestState::Active),
            Condition::QuestDone(id) => self.quest(id) == Some(QuestState::Done),
            Condition::QuestNotStarted(id) => self.quest(id).is_none(),
            Condition::InParty(id) => self.member(id).is_some(),
            Condition::Chapter(n) => self.chapter == *n,
            Condition::TimeIs(time) => self.time == *time,
            Condition::All(list) => list.iter().all(|c| self.eval(c)),
            Condition::Any(list) => list.iter().any(|c| self.eval(c)),
            Condition::Not(inner) => !self.eval(inner),
        }
    }

    pub fn eval_opt(&self, condition: &Option<Condition>) -> bool {
        condition.as_ref().is_none_or(|c| self.eval(c))
    }

    pub(crate) fn add_item(&mut self, id: &str, n: u32) {
        *self.items.entry(id.to_string()).or_insert(0) += n;
    }

    /// Removes up to `n`; returns how many were removed.
    pub fn remove_item(&mut self, id: &str, n: u32) -> u32 {
        let Some(count) = self.items.get_mut(id) else {
            return 0;
        };
        let removed = n.min(*count);
        *count -= removed;
        if *count == 0 {
            self.items.remove(id);
        }
        removed
    }

    /// Adds tu vi to every party member and advances minor stages.
    pub fn gain_tu_vi(&mut self, amount: u32) -> Vec<Notice> {
        let mut notices = vec![Notice::TuVi(amount)];
        for i in 0..self.party.len() {
            notices.extend(self.member_tu_vi(i, amount));
        }
        notices
    }

    /// Tu vi for one member; returns the stage-up notices.
    pub fn member_tu_vi(&mut self, index: usize, amount: u32) -> Vec<Notice> {
        let mut notices = Vec::new();
        let Some(member) = self.party.get_mut(index) else {
            return notices;
        };
        if member.realm == Realm::PhamNhan {
            return notices;
        }
        member.tu_vi += amount;
        let thresholds = stage_thresholds(member.realm);
        while (member.stage as usize) < thresholds.len()
            && member.tu_vi >= thresholds[member.stage as usize]
        {
            member.stage += 1;
            notices.push(Notice::StageUp(
                member.id.clone(),
                member.realm,
                member.stage,
            ));
        }
        notices
    }

    /// Applies one effect. Quest completion is checked afterwards with
    /// [`Progress::refresh_quests`] (done automatically by [`Progress::apply_all`]).
    pub fn apply(&mut self, effect: &StoryEffect, db: &GameDb) -> Outcome {
        let mut out = Outcome::default();
        match effect {
            StoryEffect::SetFlag(name, value) => self.set_flag(name, *value),
            StoryEffect::AddFlag(name, delta) => {
                let value = self.flag(name) + delta;
                self.set_flag(name, value);
            }
            StoryEffect::GiveItem(id, n) => {
                self.add_item(id, *n);
                out.notices.push(Notice::ItemGained(id.clone(), *n));
            }
            StoryEffect::TakeItem(id, n) => {
                let removed = self.remove_item(id, *n);
                if removed > 0 {
                    out.notices.push(Notice::ItemLost(id.clone(), removed));
                }
            }
            StoryEffect::GiveMoney(n) => {
                self.money = (self.money + *n as i64).max(0);
                out.notices.push(Notice::Money(*n as i64));
            }
            StoryEffect::StartQuest(id) => {
                if self.quest(id).is_none() && db.quests.contains_key(id) {
                    self.quests.insert(id.clone(), QuestState::Active);
                    self.quest_order.push(id.clone());
                    out.notices.push(Notice::QuestStarted(id.clone()));
                }
            }
            StoryEffect::CompleteQuest(id) => {
                if self.quest(id) == Some(QuestState::Active) {
                    out.extend(self.finish_quest(id, db));
                }
            }
            StoryEffect::FailQuest(id) => {
                if self.quest(id) == Some(QuestState::Active) {
                    self.quests.insert(id.clone(), QuestState::Failed);
                    out.notices.push(Notice::QuestFailed(id.clone()));
                }
            }
            StoryEffect::Dialogue(id) => out.deferred.push(Deferred::Dialogue(id.clone())),
            StoryEffect::Battle(id) => out.deferred.push(Deferred::Battle(id.clone())),
            StoryEffect::Warp(level, x, y) => out.deferred.push(Deferred::Warp {
                level: level.clone(),
                x: *x,
                y: *y,
            }),
            StoryEffect::Card(key) => out.deferred.push(Deferred::Card(key.clone())),
            StoryEffect::JoinParty(id) => {
                if self.member(id).is_none()
                    && let Some(def) = db.characters.get(id)
                {
                    let mut member = Member::new(def);
                    // Companions join at the protagonist's realm.
                    if let Some(player) = self.player() {
                        member.realm = player.realm.max(Realm::LuyenKhi);
                        member.stage = player.stage;
                    }
                    self.party.push(member);
                    out.notices.push(Notice::Joined(id.clone()));
                }
            }
            StoryEffect::LeaveParty(id) => {
                if id != PLAYER_ID && self.member(id).is_some() {
                    self.party.retain(|m| &m.id != id);
                    out.notices.push(Notice::Left(id.clone()));
                }
            }
            StoryEffect::LearnSkill(member, skill) => {
                if let Some(m) = self.member_mut(member)
                    && !m.skills.contains(skill)
                {
                    m.skills.push(skill.clone());
                    out.notices
                        .push(Notice::SkillLearned(member.clone(), skill.clone()));
                }
            }
            StoryEffect::GiveArtifact(member, artifact) => {
                if let Some(m) = self.member_mut(member)
                    && !m.artifacts.contains(artifact)
                {
                    // Equip when a slot is free; otherwise keep it in the bag.
                    if m.artifacts.len() < m.realm.artifact_slots() {
                        m.artifacts.push(artifact.clone());
                    } else {
                        self.add_item(artifact, 1);
                    }
                    out.notices.push(Notice::ArtifactGained(artifact.clone()));
                }
            }
            StoryEffect::Trust(character, delta) => {
                let key = format!("trust.{character}");
                let value = (self.flag(&key) + delta).clamp(-5, 10);
                self.set_flag(&key, value);
            }
            StoryEffect::GainTuVi(n) => out.notices.extend(self.gain_tu_vi(*n)),
            StoryEffect::SetRealm(realm, stage) => {
                if let Some(player) = self.party.iter_mut().find(|m| m.id == PLAYER_ID) {
                    player.realm = *realm;
                    player.stage = *stage;
                    player.tu_vi = if *stage == 0 {
                        0
                    } else {
                        stage_thresholds(*realm)[*stage as usize - 1]
                    };
                    out.notices
                        .push(Notice::StageUp(PLAYER_ID.into(), *realm, *stage));
                }
            }
            StoryEffect::HealParty => {}
            StoryEffect::Autosave => out.deferred.push(Deferred::Autosave),
            StoryEffect::TimeOfDay(time) => self.time = *time,
            StoryEffect::Notify(key) => out.notices.push(Notice::Custom(key.clone())),
            StoryEffect::SetFormation(id) => self.formation = id.clone(),
            StoryEffect::LearnFormation(id) => {
                if !self.formations_known.contains(id) {
                    self.formations_known.push(id.clone());
                    out.notices.push(Notice::FormationLearned(id.clone()));
                }
            }
            StoryEffect::SetChapter(n) => self.chapter = *n,
            StoryEffect::Music(track) => {
                self.music = (!track.is_empty()).then(|| track.clone());
            }
            StoryEffect::OpenShop(id) => out.deferred.push(Deferred::Shop(id.clone())),
            StoryEffect::OpenCraft(station) => out.deferred.push(Deferred::Craft(*station)),
            StoryEffect::LearnRecipe(id) => {
                if !self.recipes_known.contains(id) {
                    self.recipes_known.push(id.clone());
                    out.notices.push(Notice::RecipeLearned(id.clone()));
                }
            }
        }
        out
    }

    /// Applies effects in order, then re-checks quests.
    pub fn apply_all(&mut self, effects: &[StoryEffect], db: &GameDb) -> Outcome {
        let mut out = Outcome::default();
        for effect in effects {
            out.extend(self.apply(effect, db));
        }
        out.extend(self.refresh_quests(db));
        out
    }

    fn finish_quest(&mut self, id: &str, db: &GameDb) -> Outcome {
        let mut out = Outcome::default();
        self.quests.insert(id.to_string(), QuestState::Done);
        out.notices.push(Notice::QuestDone(id.to_string()));
        if let Some(def) = db.quests.get(id) {
            for effect in def.rewards.iter().chain(&def.on_complete) {
                out.extend(self.apply(effect, db));
            }
        }
        out
    }

    /// Completes or fails active quests whose conditions now hold.
    pub fn refresh_quests(&mut self, db: &GameDb) -> Outcome {
        let mut out = Outcome::default();
        // Completing a quest can start or finish others; iterate to a fixed point.
        for _ in 0..16 {
            let mut changed = false;
            let active: Vec<String> = self
                .quests
                .iter()
                .filter(|(_, s)| **s == QuestState::Active)
                .map(|(id, _)| id.clone())
                .collect();
            for id in active {
                let Some(def) = db.quests.get(&id) else {
                    continue;
                };
                if def.fail_when.as_ref().is_some_and(|c| self.eval(c)) {
                    self.quests.insert(id.clone(), QuestState::Failed);
                    out.notices.push(Notice::QuestFailed(id));
                    changed = true;
                } else if def.objectives.iter().all(|o| self.eval(&o.done_when)) {
                    out.extend(self.finish_quest(&id, db));
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        out
    }

    /// Objectives of a quest with their completion state.
    pub fn objectives<'a>(&self, def: &'a QuestDef) -> Vec<(&'a QuestObjective, bool)> {
        def.objectives
            .iter()
            .map(|o| (o, self.eval(&o.done_when)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> GameDb {
        let ron = r#"(
            characters: [(id: "player", sheet: 1, base: (hp: 90, ll: 20, atk: 10, spi: 4, def: 5, tp: 32),
                          growth: (hp: 40, ll: 20, atk: 6, spi: 4, def: 3, tp: 6), skills: ["quet_gay"])],
            quests: [
              (id: "q1", chapter: 1, kind: Main, objectives: [
                  (id: "gather", done_when: HasItem("herb", 3)),
                  (id: "return", done_when: Flag("delivered")),
              ], rewards: [GiveMoney(20)], on_complete: [StartQuest("q2")]),
              (id: "q2", chapter: 1, kind: Main, objectives: [(id: "x", done_when: Flag("never"))],
               fail_when: Some(Flag("raid"))),
            ],
        )"#;
        let file: DataFile = ron::from_str(ron).expect("valid test data");
        let (db, errors) = GameDb::from_files([file]);
        assert!(errors.is_empty());
        db
    }

    #[test]
    fn quest_completes_and_chains() {
        let db = db();
        let mut p = Progress::new_game(&db, PlayerProfile::default());
        p.apply_all(&[StoryEffect::StartQuest("q1".into())], &db);
        assert_eq!(p.quest("q1"), Some(QuestState::Active));
        p.apply_all(&[StoryEffect::GiveItem("herb".into(), 3)], &db);
        assert_eq!(p.quest("q1"), Some(QuestState::Active));
        let out = p.apply_all(&[StoryEffect::SetFlag("delivered".into(), 1)], &db);
        assert_eq!(p.quest("q1"), Some(QuestState::Done));
        assert_eq!(p.quest("q2"), Some(QuestState::Active));
        assert_eq!(p.money, 20);
        assert!(out.notices.contains(&Notice::QuestDone("q1".into())));
        p.apply_all(&[StoryEffect::SetFlag("raid".into(), 1)], &db);
        assert_eq!(p.quest("q2"), Some(QuestState::Failed));
    }

    #[test]
    fn conditions_compose() {
        let db = db();
        let mut p = Progress::new_game(&db, PlayerProfile::default());
        p.set_flag("a", 2);
        let c = Condition::All(vec![
            Condition::FlagAtLeast("a".into(), 2),
            Condition::Not(Box::new(Condition::Flag("b".into()))),
            Condition::Any(vec![
                Condition::InParty("player".into()),
                Condition::Flag("zzz".into()),
            ]),
        ]);
        assert!(p.eval(&c));
        p.set_flag("b", 1);
        assert!(!p.eval(&c));
    }

    #[test]
    fn tu_vi_advances_stages_but_not_mortals() {
        let db = db();
        let mut p = Progress::new_game(&db, PlayerProfile::default());
        p.gain_tu_vi(500);
        assert_eq!(p.player().map(|m| m.stage), Some(0));
        p.apply_all(&[StoryEffect::SetRealm(Realm::LuyenKhi, 0)], &db);
        let notices = p.gain_tu_vi(200);
        let player = p.player().expect("player");
        assert_eq!(player.stage, 2);
        assert_eq!(
            notices
                .iter()
                .filter(|n| matches!(n, Notice::StageUp(..)))
                .count(),
            2
        );
        let def = &db.characters["player"];
        assert_eq!(player.stats(def).atk, 10 + 6 * 3);
    }

    #[test]
    fn items_never_go_negative() {
        let db = db();
        let mut p = Progress::new_game(&db, PlayerProfile::default());
        p.apply_all(&[StoryEffect::GiveItem("x".into(), 2)], &db);
        let out = p.apply_all(&[StoryEffect::TakeItem("x".into(), 5)], &db);
        assert_eq!(p.item_count("x"), 0);
        assert_eq!(out.notices, vec![Notice::ItemLost("x".into(), 2)]);
    }

    #[test]
    fn deferred_actions_keep_order() {
        let db = db();
        let mut p = Progress::new_game(&db, PlayerProfile::default());
        let out = p.apply_all(
            &[
                StoryEffect::Dialogue("d".into()),
                StoryEffect::Battle("b".into()),
                StoryEffect::Autosave,
            ],
            &db,
        );
        assert_eq!(
            out.deferred,
            vec![
                Deferred::Dialogue("d".into()),
                Deferred::Battle("b".into()),
                Deferred::Autosave
            ]
        );
    }
}

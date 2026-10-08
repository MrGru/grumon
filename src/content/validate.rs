//! Content validation (content-schema §5). Run by `cargo test` over the real
//! assets and at startup in debug builds.

use std::collections::{BTreeSet, HashSet};

use super::{
    db::GameDb,
    defs::*,
    locale::{Locale, all_variants, check_template},
};

/// English words that must never appear in player-facing text.
pub const FORBIDDEN_ENGLISH: &[&str] = &[
    "Start",
    "Continue",
    "Load",
    "Save",
    "Settings",
    "Quit",
    "Exit",
    "Loading",
    "Quest",
    "Quests",
    "Inventory",
    "Skill",
    "Skills",
    "Level",
    "Level Up",
    "HP",
    "MP",
    "EXP",
    "XP",
    "Boss",
    "Game Over",
    "Victory",
    "Defeat",
    "Attack",
    "Defense",
    "Menu",
    "Item",
    "Items",
    "OK",
    "Back",
    "Cancel",
    "Yes",
    "and",
    "you",
];

/// Key namespaces owned by code rather than by a data definition.
const CODE_NAMESPACES: &[&str] = &[
    "ui.",
    "log.",
    "notice.",
    "hint.",
    "card.",
    "realm.",
    "stage.",
    "element.",
    "slot.",
    "rarity.",
    "chapter.",
    "reason.",
    "intent.",
    "objective.",
    "tut.",
];

/// Inputs that live outside the data files.
#[derive(Debug, Default, Clone)]
pub struct ExternalRefs {
    /// LDtk level identifiers.
    pub levels: Vec<String>,
    /// `id` fields of LDtk `Npc` entities.
    pub npc_ids: Vec<String>,
    /// `id` fields of LDtk `Trigger` entities.
    pub trigger_ids: Vec<String>,
    /// `id` fields of LDtk `Object` entities.
    pub object_ids: Vec<String>,
    /// Battle background IDs that have an image.
    pub backgrounds: Vec<String>,
}

struct Checker<'a> {
    db: &'a GameDb,
    locale: &'a Locale,
    ext: &'a ExternalRefs,
    errors: Vec<String>,
    owned_keys: HashSet<String>,
}

impl Checker<'_> {
    fn err(&mut self, msg: String) {
        self.errors.push(msg);
    }

    fn need_key(&mut self, key: String, owner: &str) {
        if !self.locale.has(&key) {
            self.err(format!("missing locale key `{key}` (for {owner})"));
        }
        self.owned_keys.insert(key);
    }

    fn opt_key(&mut self, key: String) {
        self.owned_keys.insert(key);
    }

    fn item(&mut self, id: &str, ctx: &str) {
        if !self.db.items.contains_key(id) && !self.db.artifacts.contains_key(id) {
            self.err(format!("{ctx}: unknown item `{id}`"));
        }
    }

    fn skill(&mut self, id: &str, ctx: &str) {
        if !self.db.skills.contains_key(id) {
            self.err(format!("{ctx}: unknown skill `{id}`"));
        }
    }

    fn artifact(&mut self, id: &str, ctx: &str) {
        if !self.db.artifacts.contains_key(id) {
            self.err(format!("{ctx}: unknown artifact `{id}`"));
        }
    }

    fn character(&mut self, id: &str, ctx: &str) {
        if !self.db.characters.contains_key(id) {
            self.err(format!("{ctx}: unknown character `{id}`"));
        }
    }

    fn quest(&mut self, id: &str, ctx: &str) {
        if !self.db.quests.contains_key(id) {
            self.err(format!("{ctx}: unknown quest `{id}`"));
        }
    }

    fn condition(&mut self, cond: &Condition, ctx: &str) {
        match cond {
            Condition::HasItem(id, _) => self.item(id, ctx),
            Condition::QuestActive(id)
            | Condition::QuestDone(id)
            | Condition::QuestNotStarted(id) => self.quest(id, ctx),
            Condition::InParty(id) => self.character(id, ctx),
            Condition::All(list) | Condition::Any(list) => {
                for c in list {
                    self.condition(c, ctx);
                }
            }
            Condition::Not(inner) => self.condition(inner, ctx),
            Condition::Flag(_)
            | Condition::NotFlag(_)
            | Condition::FlagAtLeast(..)
            | Condition::FlagEquals(..)
            | Condition::Chapter(_)
            | Condition::TimeIs(_) => {}
        }
    }

    fn effects(&mut self, effects: &[StoryEffect], ctx: &str) {
        for effect in effects {
            match effect {
                StoryEffect::GiveItem(id, _) | StoryEffect::TakeItem(id, _) => self.item(id, ctx),
                StoryEffect::StartQuest(id)
                | StoryEffect::CompleteQuest(id)
                | StoryEffect::FailQuest(id) => self.quest(id, ctx),
                StoryEffect::Dialogue(id) => {
                    if !self.db.dialogues.contains_key(id) {
                        self.err(format!("{ctx}: unknown dialogue `{id}`"));
                    }
                }
                StoryEffect::Battle(id) => {
                    if !self.db.encounters.contains_key(id) {
                        self.err(format!("{ctx}: unknown encounter `{id}`"));
                    }
                }
                StoryEffect::Warp(level, _, _) => {
                    if !self.ext.levels.is_empty() && !self.ext.levels.contains(level) {
                        self.err(format!("{ctx}: unknown level `{level}`"));
                    }
                }
                StoryEffect::Card(key) => {
                    self.need_key(format!("card.{key}.title"), ctx);
                    self.opt_key(format!("card.{key}.subtitle"));
                }
                StoryEffect::Notify(key) => self.need_key(key.clone(), ctx),
                StoryEffect::JoinParty(id) | StoryEffect::LeaveParty(id) => self.character(id, ctx),
                StoryEffect::LearnSkill(member, skill) => {
                    self.character(member, ctx);
                    self.skill(skill, ctx);
                }
                StoryEffect::GiveArtifact(member, artifact) => {
                    self.character(member, ctx);
                    self.artifact(artifact, ctx);
                }
                StoryEffect::Trust(character, _) => {
                    if !self.db.characters.contains_key(character) {
                        self.need_key(format!("char.{character}.name"), ctx);
                    }
                }
                StoryEffect::SetFormation(Some(id)) => {
                    if !self.db.formations.contains_key(id) {
                        self.err(format!("{ctx}: unknown formation `{id}`"));
                    }
                }
                StoryEffect::SetFlag(..)
                | StoryEffect::AddFlag(..)
                | StoryEffect::GiveMoney(_)
                | StoryEffect::GainTuVi(_)
                | StoryEffect::SetRealm(..)
                | StoryEffect::HealParty
                | StoryEffect::Autosave
                | StoryEffect::TimeOfDay(_)
                | StoryEffect::SetFormation(None)
                | StoryEffect::SetChapter(_) => {}
            }
        }
    }

    fn battle_effects(&mut self, effects: &[BattleEffect], ctx: &str) {
        for effect in effects {
            if let BattleEffect::Damage { hits, .. } = effect
                && *hits == 0
            {
                self.err(format!("{ctx}: damage with 0 hits"));
            }
        }
    }

    fn speaker(&mut self, speaker: &Option<String>, ctx: &str) {
        if let Some(s) = speaker
            && s != "player"
            && !self.db.characters.contains_key(s)
        {
            self.need_key(format!("char.{s}.name"), ctx);
        }
    }

    fn dialogue(&mut self, def: &DialogueDef) {
        let ctx = format!("dialogue `{}`", def.id);
        if !def.nodes.contains_key("start") {
            self.err(format!("{ctx}: no `start` node"));
        }
        for (node_id, node) in &def.nodes {
            let node_ctx = format!("{ctx} node `{node_id}`");
            for next in node.successors() {
                if !def.nodes.contains_key(next) {
                    self.err(format!("{node_ctx}: next `{next}` does not exist"));
                }
            }
            if node.has_text() {
                self.need_key(format!("dlg.{}.{node_id}", def.id), &node_ctx);
            }
            match node {
                DialogueNode::Line {
                    speaker, effects, ..
                } => {
                    self.speaker(speaker, &node_ctx);
                    self.effects(effects, &node_ctx);
                }
                DialogueNode::Choice {
                    speaker,
                    options,
                    effects,
                } => {
                    self.speaker(speaker, &node_ctx);
                    self.effects(effects, &node_ctx);
                    if options.is_empty() {
                        self.err(format!("{node_ctx}: choice without options"));
                    }
                    if options.iter().all(|o| o.when.is_some()) {
                        self.err(format!(
                            "{node_ctx}: every option is conditional (possible dead end)"
                        ));
                    }
                    for option in options {
                        self.need_key(format!("dlg.{}.{node_id}.{}", def.id, option.id), &node_ctx);
                        self.effects(&option.effects, &node_ctx);
                        if let Some(c) = &option.when {
                            self.condition(c, &node_ctx);
                        }
                    }
                }
                DialogueNode::Branch { arms, .. } => {
                    for arm in arms {
                        self.condition(&arm.when, &node_ctx);
                    }
                }
                DialogueNode::Effects { effects, .. } => self.effects(effects, &node_ctx),
            }
        }
        // Reachability from `start`.
        let mut seen = BTreeSet::new();
        let mut stack = vec!["start"];
        while let Some(id) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            if let Some(node) = def.nodes.get(id) {
                stack.extend(node.successors());
            }
        }
        for node_id in def.nodes.keys() {
            if !seen.contains(node_id.as_str()) {
                self.err(format!("{ctx}: node `{node_id}` is unreachable"));
            }
        }
    }
}

/// Checks references, locale keys and dialogue graphs. Returns every problem found.
pub fn validate(db: &GameDb, locale: &Locale, ext: &ExternalRefs) -> Vec<String> {
    let mut c = Checker {
        db,
        locale,
        ext,
        errors: Vec::new(),
        owned_keys: HashSet::new(),
    };

    for item in db.items.values() {
        c.need_key(format!("item.{}.name", item.id), "item");
        c.need_key(format!("item.{}.desc", item.id), "item");
        c.battle_effects(&item.battle_use, &format!("item `{}`", item.id));
    }
    for skill in db.skills.values() {
        c.need_key(format!("skill.{}.name", skill.id), "skill");
        c.need_key(format!("skill.{}.desc", skill.id), "skill");
        let ctx = format!("skill `{}`", skill.id);
        c.battle_effects(&skill.effects, &ctx);
        for bonus in &skill.charge_bonus {
            c.battle_effects(&bonus.effects, &ctx);
        }
        if skill.effects.is_empty() {
            c.err(format!("{ctx}: no effects"));
        }
    }
    for artifact in db.artifacts.values() {
        let ctx = format!("artifact `{}`", artifact.id);
        c.need_key(format!("artifact.{}.name", artifact.id), &ctx);
        c.need_key(format!("artifact.{}.desc", artifact.id), &ctx);
        c.opt_key(format!("artifact.{}.lore", artifact.id));
        if let Some(active) = &artifact.active {
            c.skill(active, &ctx);
        }
    }
    for formation in db.formations.values() {
        let ctx = format!("formation `{}`", formation.id);
        c.need_key(format!("formation.{}.name", formation.id), &ctx);
        c.need_key(format!("formation.{}.desc", formation.id), &ctx);
        let thresholds: Vec<u32> = formation.phases.iter().map(|p| p.threshold).collect();
        if formation.phases.is_empty() || !thresholds.windows(2).all(|w| w[0] < w[1]) {
            c.err(format!(
                "{ctx}: phase thresholds must be ascending and non-empty"
            ));
        }
    }
    for status in StatusKind::ALL {
        c.need_key(format!("status.{}.name", status.id()), "status");
        c.need_key(format!("status.{}.desc", status.id()), "status");
    }
    for enemy in db.enemies.values() {
        let ctx = format!("enemy `{}`", enemy.id);
        c.need_key(format!("enemy.{}.name", enemy.id), &ctx);
        for skill in &enemy.skills {
            c.skill(skill, &ctx);
        }
        for rule in &enemy.ai {
            c.skill(&rule.skill, &ctx);
            if !enemy.skills.contains(&rule.skill) {
                c.err(format!(
                    "{ctx}: AI uses `{}` not in its skill list",
                    rule.skill
                ));
            }
        }
        if !enemy
            .ai
            .iter()
            .any(|r| !r.reactive && r.when == AiCond::Always)
        {
            c.err(format!(
                "{ctx}: needs a non-reactive `Always` AI rule as fallback"
            ));
        }
        for (item, _) in &enemy.drops {
            c.item(item, &ctx);
        }
    }
    for encounter in db.encounters.values() {
        let ctx = format!("encounter `{}`", encounter.id);
        c.need_key(format!("encounter.{}.objective", encounter.id), &ctx);
        if encounter.enemies.is_empty() {
            c.err(format!("{ctx}: no enemies"));
        }
        for (enemy, _) in &encounter.enemies {
            if !db.enemies.contains_key(enemy) {
                c.err(format!("{ctx}: unknown enemy `{enemy}`"));
            }
        }
        for guest in &encounter.guests {
            c.character(&guest.character, &ctx);
        }
        for p in &encounter.protect {
            c.character(p, &ctx);
        }
        if let Some(f) = &encounter.formation
            && !db.formations.contains_key(f)
        {
            c.err(format!("{ctx}: unknown formation `{f}`"));
        }
        if let Objective::DefeatTarget(target) = &encounter.objective
            && !encounter.enemies.iter().any(|(e, _)| e == target)
        {
            c.err(format!(
                "{ctx}: objective target `{target}` not in encounter"
            ));
        }
        if let Objective::FormationPhase(_) = encounter.objective
            && encounter.formation.is_none()
        {
            c.err(format!("{ctx}: formation objective without a formation"));
        }
        if matches!(
            encounter.objective,
            Objective::FormationPhase(_) | Objective::Survive(_)
        ) && encounter.can_flee
        {
            c.err(format!("{ctx}: story objectives should not allow fleeing"));
        }
        if !ext.backgrounds.is_empty() && !ext.backgrounds.contains(&encounter.background) {
            c.err(format!(
                "{ctx}: unknown background `{}`",
                encounter.background
            ));
        }
        for hint in &encounter.hints {
            c.need_key(hint.clone(), &ctx);
        }
        c.effects(&encounter.on_victory, &ctx);
        c.effects(&encounter.on_defeat, &ctx);
    }
    for character in db.characters.values() {
        let ctx = format!("character `{}`", character.id);
        if character.id != "player" {
            c.need_key(format!("char.{}.name", character.id), &ctx);
        }
        c.opt_key(format!("char.{}.title", character.id));
        for skill in &character.skills {
            c.skill(skill, &ctx);
        }
        for artifact in &character.artifacts {
            c.artifact(artifact, &ctx);
        }
    }
    for npc in db.npcs.values() {
        let ctx = format!("npc `{}`", npc.id);
        if !db.characters.contains_key(&npc.character) {
            c.need_key(format!("char.{}.name", npc.character), &ctx);
        }
        if let Some(cond) = &npc.visible {
            c.condition(cond, &ctx);
        }
        if npc.talk.is_empty() {
            c.err(format!("{ctx}: nothing to say"));
        }
        if npc.talk.last().is_some_and(|t| t.when.is_some()) {
            c.err(format!("{ctx}: last talk entry should be unconditional"));
        }
        for entry in &npc.talk {
            if !db.dialogues.contains_key(&entry.dialogue) {
                c.err(format!("{ctx}: unknown dialogue `{}`", entry.dialogue));
            }
            if let Some(cond) = &entry.when {
                c.condition(cond, &ctx);
            }
        }
    }
    for dialogue in db.dialogues.values() {
        c.dialogue(dialogue);
    }
    for quest in db.quests.values() {
        let ctx = format!("quest `{}`", quest.id);
        c.need_key(format!("quest.{}.title", quest.id), &ctx);
        c.need_key(format!("quest.{}.desc", quest.id), &ctx);
        if quest.objectives.is_empty() {
            c.err(format!("{ctx}: no objectives"));
        }
        for objective in &quest.objectives {
            c.need_key(format!("quest.{}.obj.{}", quest.id, objective.id), &ctx);
            c.condition(&objective.done_when, &ctx);
        }
        c.effects(&quest.rewards, &ctx);
        c.effects(&quest.on_complete, &ctx);
        if let Some(cond) = &quest.fail_when {
            c.condition(cond, &ctx);
        }
    }
    for trigger in db.triggers.values() {
        let ctx = format!("trigger `{}`", trigger.id);
        if let Some(cond) = &trigger.when {
            c.condition(cond, &ctx);
        }
        c.effects(&trigger.effects, &ctx);
    }
    for object in db.objects.values() {
        let ctx = format!("object `{}`", object.id);
        if let Some(cond) = &object.when {
            c.condition(cond, &ctx);
        }
        c.effects(&object.effects, &ctx);
    }
    for chapter in db.chapters.values() {
        let ctx = format!("chapter {}", chapter.number);
        c.need_key(format!("chapter.{}.title", chapter.number), &ctx);
        c.effects(&chapter.start_effects, &ctx);
        if !ext.levels.is_empty() && !ext.levels.contains(&chapter.start_level) {
            c.err(format!("{ctx}: unknown level `{}`", chapter.start_level));
        }
    }
    for level in &ext.levels {
        c.need_key(format!("map.{level}.name"), "LDtk level");
    }
    for id in &ext.npc_ids {
        if !db.npcs.contains_key(id) {
            c.err(format!("LDtk Npc `{id}` has no npc definition"));
        }
    }
    for id in &ext.trigger_ids {
        if !db.triggers.contains_key(id) {
            c.err(format!("LDtk Trigger `{id}` has no trigger definition"));
        }
    }
    for id in &ext.object_ids {
        if !db.objects.contains_key(id) {
            c.err(format!("LDtk Object `{id}` has no object definition"));
        }
    }

    // Orphan keys: every non-code key must belong to some definition.
    let mut orphans: Vec<&String> = locale
        .keys()
        .filter(|k| !CODE_NAMESPACES.iter().any(|ns| k.starts_with(ns)))
        .filter(|k| !c.owned_keys.contains(*k))
        .collect();
    orphans.sort();
    for key in orphans {
        c.errors.push(format!("locale key `{key}` has no owner"));
    }

    c.errors.extend(check_text(locale));
    c.errors
}

fn contains_word(text: &str, word: &str) -> bool {
    let is_word_char = |c: char| c.is_alphanumeric();
    text.match_indices(word).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + word.len()..].chars().next();
        !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
    })
}

/// Template syntax, English leakage, NFC and diacritics checks on every string.
pub fn check_text(locale: &Locale) -> Vec<String> {
    let mut errors = Vec::new();
    let mut entries: Vec<_> = locale.entries().collect();
    entries.sort();
    for (key, template) in entries {
        if let Err(e) = check_template(template) {
            errors.push(format!("`{key}`: broken template ({e:?})"));
            continue;
        }
        if template.trim().is_empty() {
            errors.push(format!("`{key}`: empty text"));
        }
        if template
            .chars()
            .any(|c| ('\u{0300}'..='\u{036f}').contains(&c))
        {
            errors.push(format!("`{key}`: not NFC (combining diacritics)"));
        }
        if template.contains("TODO") || template.contains("TBD") || template.contains("lorem") {
            errors.push(format!("`{key}`: placeholder text"));
        }
        for variant in all_variants(template) {
            // Strip `{param}` tokens so parameter names are not checked as prose.
            let mut prose = String::new();
            let mut depth = 0;
            for ch in variant.chars() {
                match ch {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ if depth == 0 => prose.push(ch),
                    _ => {}
                }
            }
            for word in FORBIDDEN_ENGLISH {
                if contains_word(&prose, word) {
                    errors.push(format!(
                        "`{key}`: English word `{word}` in player-facing text"
                    ));
                }
            }
            let words = prose
                .split_whitespace()
                .filter(|w| w.chars().any(char::is_alphabetic))
                .count();
            if words >= 4 && prose.is_ascii() {
                errors.push(format!(
                    "`{key}`: prose without any Vietnamese diacritics: “{prose}”"
                ));
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_matching_respects_boundaries() {
        assert!(contains_word("Press Start now", "Start"));
        assert!(!contains_word("Restart", "Start"));
        assert!(contains_word("HP: 10", "HP"));
        assert!(!contains_word("HPx", "HP"));
        assert!(contains_word("Mất HP", "HP"));
    }

    #[test]
    fn flags_english_and_unaccented_prose() {
        let mut locale = Locale::default();
        locale.merge(
            [
                ("ui.a".to_string(), "Start game".to_string()),
                ("ui.b".to_string(), "Toi la nguoi tot".to_string()),
                ("ui.c".to_string(), "Bắt đầu hành trình".to_string()),
                ("ui.d".to_string(), "{g:a|b}".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        let errors = check_text(&locale);
        assert!(errors.iter().any(|e| e.contains("`ui.a`")));
        assert!(errors.iter().any(|e| e.contains("`ui.b`")));
        assert!(!errors.iter().any(|e| e.contains("`ui.c`")));
        assert!(errors.iter().any(|e| e.contains("`ui.d`")));
    }
}

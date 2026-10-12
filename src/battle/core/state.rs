//! Battle state types, constants and setup from an encounter.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::rng::Rng;
use crate::{
    content::{db::GameDb, defs::*},
    story::{PLAYER_ID, Progress, stage_index},
};

// game-systems §2–§4
pub const TICKS_PER_CYCLE: u32 = 1000;
pub const MIN_RECOVERY: u32 = 400;
pub const AP_PER_ACTIVATION: u8 = 3;
pub const AP_CARRY_MAX: u8 = 1;
pub const AP_MAX: u8 = 4;
pub const LL_REGEN_PCT: u32 = 8;
pub const MAX_ACTION_DELAY: u32 = 600;
pub const PUSH_CAP_PER_CYCLE: u32 = 500;
pub const COMBO_WINDOW: u32 = 500;
pub const CHARGE_MULT: [u32; 5] = [100, 160, 240, 340, 450];
pub const CHARGE_BREAK_PCT: u32 = 25;
pub const CHARGE_LL_COST: u32 = 5;
pub const OVERCHARGE_HP_PCT: u32 = 10;
pub const BACKLASH_PCT: u32 = 8;
pub const STRIKE_LL_GAIN: u32 = 5;
pub const MEDITATE_PCT: u32 = 30;
pub const INTERRUPT_STAGGER: u32 = 200;
pub const MAX_CONSECUTIVE: u8 = 2;
pub const STUN_IMMUNITY_TURNS: u8 = 2;
pub const BOSS_PUSH_RESIST_MAX: u8 = 3;

/// `tempo × 100` for a given effective Thân pháp (diminishing returns).
pub fn tempo_x100(tp: u32) -> u32 {
    6000 + 14000 * tp / (tp + 60)
}

/// Ticks between two activations at a given effective Thân pháp.
pub fn recovery(tp: u32) -> u32 {
    (10_000_000 / tempo_x100(tp)).max(MIN_RECOVERY)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusInst {
    pub kind: StatusKind,
    /// Remaining own activations; 0 = until consumed or removed by a rule.
    pub turns: u8,
    /// Shield amount, overheat stacks…
    pub value: u32,
    /// Unit that applied it (for shields feeding the formation).
    pub source: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillSlot {
    pub id: String,
    pub cooldown: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactSlot {
    pub id: String,
    pub cooldown: u8,
    /// Remaining uses this battle (`None` = unlimited).
    pub charges: Option<u8>,
    /// Luyện khí: extra power of the active ability, in percent.
    #[serde(default)]
    pub power_pct: u32,
    /// Luyện khí: activations removed from the active ability's cooldown.
    #[serde(default)]
    pub cooldown_cut: u8,
}

/// Target of a command, relative to the actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Unit(usize),
    Opponents,
    Allies,
    Myself,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    pub skill: String,
    pub target: Target,
    pub resolve_at: u32,
    /// Power in percent when it resolves (refined artifacts exceed 100).
    #[serde(default = "full_power")]
    pub power: u32,
}

fn full_power() -> u32 {
    100
}

/// What an enemy will do on its next activation (shown to the player).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    pub skill: String,
    pub target: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    /// Character or enemy definition ID.
    pub def: String,
    pub side: Side,
    pub sheet: usize,
    pub tint: Option<(f32, f32, f32)>,
    pub sprite: Option<String>,
    pub stats: Stats,
    pub hp: u32,
    pub ll: u32,
    pub element: Element,
    pub element_override: Option<(Element, u8)>,
    pub slot: Slot,
    pub next_act: u32,
    pub ap: u8,
    pub leftover_ap: u8,
    pub pending_delay: u32,
    /// Haste received during the unit's own activation (shortens its next recovery).
    #[serde(default)]
    pub pending_haste: u32,
    pub charge: u8,
    pub max_charge: u8,
    pub charged_this_activation: bool,
    pub statuses: Vec<StatusInst>,
    pub skills: Vec<SkillSlot>,
    pub artifacts: Vec<ArtifactSlot>,
    pub stored_ll: u32,
    pub store_cap: u32,
    pub ll_regen_bonus: u32,
    pub tp_bonus: u32,
    pub channel: Option<Channel>,
    pub intent: Option<Intent>,
    pub own_activations: u32,
    pub pushed: (u32, u32),
    pub push_resist: u8,
    pub boss: bool,
    pub invulnerable: bool,
    pub guest: bool,
    pub node_broken: bool,
    pub nghich_menh: bool,
    pub shield_to_energy: bool,
    pub overheat: bool,
    /// A summoned spirit: activations left before it fades, and its summoner.
    #[serde(default)]
    pub summon: Option<SummonState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummonState {
    pub turns: u8,
    pub owner: usize,
}

impl Unit {
    pub fn alive(&self) -> bool {
        self.hp > 0
    }

    /// Alive and not a summon: the units that decide victory and defeat.
    pub fn fighting(&self) -> bool {
        self.alive() && self.summon.is_none()
    }

    pub fn status(&self, kind: StatusKind) -> Option<&StatusInst> {
        self.statuses.iter().find(|s| s.kind == kind)
    }

    pub fn has(&self, kind: StatusKind) -> bool {
        self.status(kind).is_some()
    }

    pub fn remove_status(&mut self, kind: StatusKind) -> Option<StatusInst> {
        let pos = self.statuses.iter().position(|s| s.kind == kind)?;
        Some(self.statuses.remove(pos))
    }

    pub fn shield(&self) -> u32 {
        self.status(StatusKind::Khien).map_or(0, |s| s.value)
    }

    pub fn current_element(&self) -> Element {
        self.element_override.map_or(self.element, |(e, _)| e)
    }

    pub fn hp_pct(&self) -> u32 {
        self.hp * 100 / self.stats.hp.max(1)
    }

    pub fn is_player(&self) -> bool {
        self.side == Side::Party && self.def == PLAYER_ID
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormationState {
    pub id: String,
    pub energy: u32,
    pub phase: u8,
    pub last_cycle: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Timeline advancing; call `step`.
    Running,
    /// Waiting for a command for this party unit.
    Command(usize),
    Victory,
    Defeat,
    Fled,
}

/// One line of the battle log; the UI turns it into Vietnamese text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogEntry {
    Turn {
        unit: usize,
    },
    Skill {
        unit: usize,
        skill: String,
        target: Option<usize>,
    },
    Strike {
        unit: usize,
        target: usize,
    },
    Damage {
        target: usize,
        amount: u32,
        absorbed: u32,
        effective: i8,
    },
    Miss {
        target: usize,
    },
    Heal {
        target: usize,
        amount: u32,
    },
    Shield {
        target: usize,
        amount: u32,
    },
    Status {
        target: usize,
        status: StatusKind,
    },
    StatusResisted {
        target: usize,
        status: StatusKind,
    },
    Cleansed {
        target: usize,
    },
    Charge {
        unit: usize,
        stage: u8,
    },
    ChargeBroken {
        unit: usize,
        lost: u8,
    },
    ChargeReleased {
        unit: usize,
        stage: u8,
    },
    Channel {
        unit: usize,
        skill: String,
        at: u32,
    },
    ChannelInterrupted {
        unit: usize,
    },
    Pushed {
        target: usize,
        ticks: u32,
    },
    PushResisted {
        target: usize,
    },
    Pulled {
        target: usize,
        ticks: u32,
    },
    Ko {
        unit: usize,
    },
    Summoned {
        unit: usize,
        owner: usize,
    },
    Dissipated {
        unit: usize,
    },
    Combo {
        from: Element,
        to: Element,
    },
    FormationPhase {
        phase: u8,
    },
    FormationEnergy {
        amount: u32,
    },
    NodeBroken {
        unit: usize,
    },
    NodeRestored {
        unit: usize,
    },
    FormationChaos,
    FormationReleased {
        phase: u8,
    },
    Guard {
        unit: usize,
    },
    Meditate {
        unit: usize,
        amount: u32,
    },
    LlGain {
        unit: usize,
        amount: u32,
    },
    Drain {
        target: usize,
        amount: u32,
    },
    Stunned {
        unit: usize,
    },
    Dot {
        unit: usize,
        status: StatusKind,
        amount: u32,
    },
    Backlash {
        unit: usize,
        amount: u32,
    },
    Overheat {
        unit: usize,
        amount: u32,
    },
    Item {
        unit: usize,
        item: String,
    },
    Artifact {
        unit: usize,
        artifact: String,
    },
    Swap {
        a: usize,
        b: usize,
    },
    ElementSet {
        target: usize,
        element: Element,
    },
    Fled,
    FleeFailed,
    Victory,
    Defeat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BattleState {
    pub encounter: String,
    pub clock: u32,
    pub units: Vec<Unit>,
    pub formation: Option<FormationState>,
    pub rng: Rng,
    pub phase: Phase,
    pub objective: Objective,
    pub protect: Vec<usize>,
    pub can_flee: bool,
    /// Battle-usable items carried in.
    pub items: BTreeMap<String, u32>,
    pub items_used: BTreeMap<String, u32>,
    pub last_party_element: Option<(Element, u32)>,
    pub last_actor: Option<usize>,
    pub streak: u8,
    pub log: Vec<LogEntry>,
    pub activations: u32,
    pub overcharge_releases: u32,
}

/// What the battle gives back to the story layer.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BattleResult {
    pub victory: bool,
    pub fled: bool,
    pub tu_vi: u32,
    pub drops: Vec<(String, u32)>,
    pub items_used: BTreeMap<String, u32>,
    pub overcharge_releases: u32,
}

#[allow(clippy::too_many_arguments)]
fn unit_from_character(
    def: &CharacterDef,
    db: &GameDb,
    stats: Stats,
    skills: &[String],
    artifacts: &[String],
    slot: Slot,
    sheet: usize,
    guest: bool,
) -> Unit {
    let mut unit = Unit {
        def: def.id.clone(),
        side: Side::Party,
        sheet,
        tint: None,
        sprite: None,
        stats,
        hp: stats.hp,
        ll: stats.ll,
        element: def.element,
        element_override: None,
        slot,
        next_act: 0,
        ap: 0,
        leftover_ap: 0,
        pending_delay: 0,
        pending_haste: 0,
        charge: 0,
        max_charge: if def.nghich_menh { 4 } else { 3 },
        charged_this_activation: false,
        statuses: Vec::new(),
        skills: skills
            .iter()
            .map(|id| SkillSlot {
                id: id.clone(),
                cooldown: 0,
            })
            .collect(),
        artifacts: Vec::new(),
        stored_ll: 0,
        store_cap: 0,
        ll_regen_bonus: 0,
        tp_bonus: 0,
        channel: None,
        intent: None,
        own_activations: 0,
        pushed: (0, 0),
        push_resist: 0,
        boss: false,
        invulnerable: false,
        guest,
        node_broken: false,
        nghich_menh: def.nghich_menh,
        shield_to_energy: false,
        overheat: false,
        summon: None,
    };
    for id in artifacts {
        let Some(art) = db.artifacts.get(id) else {
            continue;
        };
        unit.artifacts.push(ArtifactSlot {
            id: id.clone(),
            cooldown: 0,
            charges: art.charges_per_battle,
            power_pct: 0,
            cooldown_cut: 0,
        });
        for passive in &art.passives {
            match passive {
                ArtifactPassive::StoreLl(cap) => unit.store_cap += cap,
                ArtifactPassive::LlRegen(n) => unit.ll_regen_bonus += n,
                ArtifactPassive::SpeedBonus(n) => unit.tp_bonus += n,
                ArtifactPassive::ShieldToEnergy => unit.shield_to_energy = true,
                ArtifactPassive::Overheat => unit.overheat = true,
            }
        }
    }
    unit
}

fn unit_from_enemy(def: &EnemyDef, slot: Slot) -> Unit {
    Unit {
        def: def.id.clone(),
        side: Side::Enemy,
        sheet: def.sheet,
        tint: def.tint,
        sprite: def.sprite.clone(),
        stats: def.stats,
        hp: def.stats.hp,
        ll: def.stats.ll,
        element: def.element,
        element_override: None,
        slot,
        next_act: 0,
        ap: 0,
        leftover_ap: 0,
        pending_delay: 0,
        pending_haste: 0,
        charge: 0,
        max_charge: 3,
        charged_this_activation: false,
        statuses: Vec::new(),
        skills: def
            .skills
            .iter()
            .map(|id| SkillSlot {
                id: id.clone(),
                cooldown: 0,
            })
            .collect(),
        artifacts: Vec::new(),
        stored_ll: 0,
        store_cap: 0,
        ll_regen_bonus: 0,
        tp_bonus: 0,
        channel: None,
        intent: None,
        own_activations: 0,
        pushed: (0, 0),
        push_resist: 0,
        boss: def.boss,
        invulnerable: def.invulnerable,
        guest: false,
        node_broken: false,
        nghich_menh: false,
        shield_to_energy: false,
        overheat: false,
        summon: None,
    }
}

impl BattleState {
    /// Builds a battle for `encounter` with the current party.
    pub fn from_progress(
        db: &GameDb,
        progress: &Progress,
        encounter: &str,
        seed: u64,
    ) -> Result<Self, String> {
        let enc = db
            .encounters
            .get(encounter)
            .ok_or_else(|| format!("unknown encounter `{encounter}`"))?;
        let mut units = Vec::new();
        for member in &progress.party {
            let Some(def) = db.characters.get(&member.id) else {
                continue;
            };
            let sheet = if member.id == PLAYER_ID {
                progress.profile.sheet
            } else {
                def.sheet
            };
            let mut unit = unit_from_character(
                def,
                db,
                member.stats(def),
                &member.skills,
                &member.artifacts,
                member.slot,
                sheet,
                false,
            );
            // Mortals cannot gather qi yet (Tụ khí unlocks at Luyện Khí).
            if member.realm == Realm::PhamNhan {
                unit.max_charge = 0;
            }
            // Luyện khí gains of each equipped artifact.
            for slot in &mut unit.artifacts {
                let level = progress.artifact_levels.get(&slot.id).copied().unwrap_or(0);
                let Some(def) = db.artifacts.get(&slot.id) else {
                    continue;
                };
                for step in def.refine.iter().take(level as usize) {
                    match step.gain {
                        RefineGain::Power(pct) => slot.power_pct += pct,
                        RefineGain::Cooldown(n) => slot.cooldown_cut += n,
                        RefineGain::Charges(n) => {
                            if let Some(c) = &mut slot.charges {
                                *c += n;
                            }
                        }
                    }
                }
            }
            units.push(unit);
        }
        for guest in &enc.guests {
            let Some(def) = db.characters.get(&guest.character) else {
                continue;
            };
            let stats = def
                .base
                .plus_scaled(def.growth, stage_index(Realm::PhamNhan, 0));
            let mut unit = unit_from_character(
                def,
                db,
                stats,
                &def.skills,
                &def.artifacts,
                guest.slot,
                def.sheet,
                true,
            );
            let mut pct = guest.hp_pct.unwrap_or(100);
            if let Some((flag, alt)) = &guest.hp_pct_if
                && progress.flag(flag) != 0
            {
                pct = *alt;
            }
            unit.hp = (stats.hp * pct / 100).max(1);
            units.push(unit);
        }
        if units.is_empty() {
            return Err("party is empty".into());
        }
        let protect = enc
            .protect
            .iter()
            .filter_map(|id| units.iter().position(|u| &u.def == id))
            .collect();
        for (enemy_id, slot) in &enc.enemies {
            let def = db
                .enemies
                .get(enemy_id)
                .ok_or_else(|| format!("unknown enemy `{enemy_id}`"))?;
            units.push(unit_from_enemy(def, *slot));
        }

        let items = progress
            .items
            .iter()
            .filter(|(id, n)| {
                **n > 0 && db.items.get(*id).is_some_and(|d| !d.battle_use.is_empty())
            })
            .map(|(id, n)| (id.clone(), *n))
            .collect();

        let mut state = BattleState {
            encounter: encounter.to_string(),
            clock: 0,
            units,
            formation: None,
            rng: Rng::new(seed),
            phase: Phase::Running,
            objective: enc.objective.clone(),
            protect,
            can_flee: enc.can_flee,
            items,
            items_used: BTreeMap::new(),
            last_party_element: None,
            last_actor: None,
            streak: 0,
            log: Vec::new(),
            activations: 0,
            overcharge_releases: 0,
        };

        let formation_id = enc.formation.clone().or_else(|| progress.formation.clone());
        if let Some(def) = formation_id.and_then(|id| db.formations.get(&id)) {
            let party: Vec<&Unit> = state.party().map(|i| &state.units[i]).collect();
            let mut elements: Vec<Element> = party
                .iter()
                .map(|u| u.element)
                .filter(|e| *e != Element::Vo)
                .collect();
            elements.sort_by_key(|e| *e as u8);
            elements.dedup();
            if party.len() >= def.min_members as usize
                && elements.len() >= def.elements_required as usize
            {
                state.formation = Some(FormationState {
                    id: def.id.clone(),
                    energy: 0,
                    phase: 0,
                    last_cycle: 0,
                });
            }
        }

        for i in 0..state.units.len() {
            let rec = recovery(state.tp_eff(i));
            let side = state.units[i].side;
            state.units[i].next_act = match enc.ambush {
                Some(ambusher) if ambusher == side => rec / 4,
                Some(_) => rec,
                None => rec / 2,
            };
        }
        for i in state.enemies().collect::<Vec<_>>() {
            state.choose_intent(db, i);
        }
        Ok(state)
    }

    pub fn party(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.units.len()).filter(|&i| self.units[i].side == Side::Party)
    }

    pub fn enemies(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.units.len()).filter(|&i| self.units[i].side == Side::Enemy)
    }

    /// Living units on the same side as `actor` (including itself).
    pub fn allies_of(&self, actor: usize) -> Vec<usize> {
        let side = self.units[actor].side;
        (0..self.units.len())
            .filter(|&i| self.units[i].side == side && self.units[i].alive())
            .collect()
    }

    /// Living units opposing `actor`.
    pub fn opponents_of(&self, actor: usize) -> Vec<usize> {
        let side = self.units[actor].side;
        (0..self.units.len())
            .filter(|&i| self.units[i].side != side && self.units[i].alive())
            .collect()
    }

    pub fn is_over(&self) -> bool {
        matches!(self.phase, Phase::Victory | Phase::Defeat | Phase::Fled)
    }

    /// Effective Thân pháp (game-systems §2.2).
    pub fn tp_eff(&self, i: usize) -> u32 {
        let u = &self.units[i];
        let mut tp = u.stats.tp + u.tp_bonus;
        if u.has(StatusKind::TangToc) {
            tp = tp * 130 / 100;
        }
        if u.has(StatusKind::Cham) {
            tp = tp * 70 / 100;
        }
        tp
    }

    pub fn cycle(&self) -> u32 {
        self.clock / TICKS_PER_CYCLE
    }

    pub fn result(&self, db: &GameDb) -> BattleResult {
        let mut result = BattleResult {
            victory: self.phase == Phase::Victory,
            fled: self.phase == Phase::Fled,
            items_used: self.items_used.clone(),
            overcharge_releases: self.overcharge_releases,
            ..Default::default()
        };
        if result.victory {
            if let Some(enc) = db.encounters.get(&self.encounter) {
                result.tu_vi = enc.tu_vi;
            }
            for i in self.enemies() {
                if let Some(def) = db.enemies.get(&self.units[i].def) {
                    result.tu_vi += def.tu_vi;
                    if !self.units[i].alive() {
                        result.drops.extend(def.drops.iter().cloned());
                    }
                }
            }
        }
        result
    }
}

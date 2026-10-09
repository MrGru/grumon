//! Timeline, activations, commands, effects and formations.
//! Section numbers refer to `docs/game-systems.md`.

use serde::{Deserialize, Serialize};

use super::state::*;
use crate::content::{db::GameDb, defs::*};

/// A player command for the active party unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Strike(usize),
    Skill(String, Target),
    Guard,
    Meditate,
    Charge,
    Artifact(usize, Target),
    Item(String, Target),
    ReleaseFormation,
    Stabilize,
    Swap(usize),
    Flee,
    EndTurn,
}

/// Why a command cannot be used right now (shown in the UI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unavailable {
    NotYourTurn,
    Ap(u8),
    Ll(u32),
    Cooldown(u8),
    Sealed,
    NoCharges,
    NeedCharge,
    ChargedThisTurn,
    MaxCharge,
    NoCharge,
    NoTarget,
    NoFormation,
    FormationNotReady,
    NodeIntact,
    CannotFlee,
    NoItem,
    Unknown,
}

impl Unavailable {
    pub fn key(self) -> &'static str {
        match self {
            Unavailable::NotYourTurn => "reason.not_your_turn",
            Unavailable::Ap(_) => "reason.ap",
            Unavailable::Ll(_) => "reason.ll",
            Unavailable::Cooldown(_) => "reason.cooldown",
            Unavailable::Sealed => "reason.sealed",
            Unavailable::NoCharges => "reason.no_charges",
            Unavailable::NeedCharge => "reason.need_charge",
            Unavailable::ChargedThisTurn => "reason.charged_this_turn",
            Unavailable::MaxCharge => "reason.max_charge",
            Unavailable::NoCharge => "reason.no_charge",
            Unavailable::NoTarget => "reason.no_target",
            Unavailable::NoFormation => "reason.no_formation",
            Unavailable::FormationNotReady => "reason.formation_not_ready",
            Unavailable::NodeIntact => "reason.node_intact",
            Unavailable::CannotFlee => "reason.cannot_flee",
            Unavailable::NoItem => "reason.no_item",
            Unavailable::Unknown => "reason.unknown",
        }
    }
}

/// One upcoming entry on the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineEntry {
    pub unit: usize,
    pub tick: u32,
    pub channel: bool,
}

/// Parameters of one damage instance.
struct Hit {
    power: u32,
    scaling: Scaling,
    element: Element,
    melee: bool,
    mult: u32,
    combo: bool,
    interrupt: bool,
    single: bool,
}

const STRIKE_POWER: u32 = 100;
const GUARD_AP: u8 = 1;
const MEDITATE_AP: u8 = 2;
const CHARGE_AP: u8 = 1;
const ITEM_AP: u8 = 1;
const RELEASE_AP: u8 = 2;
const STABILIZE_AP: u8 = 1;
const SWAP_AP: u8 = 1;
const FLEE_AP: u8 = 2;

impl BattleState {
    // ------------------------------------------------------------------
    // Timeline (§2)
    // ------------------------------------------------------------------

    /// Unit whose entry comes next (§2.4 tie-breaking).
    fn next_entry(&self) -> Option<usize> {
        (0..self.units.len())
            .filter(|&i| self.units[i].alive())
            .min_by(|&a, &b| {
                let (ua, ub) = (&self.units[a], &self.units[b]);
                ua.next_act
                    .cmp(&ub.next_act)
                    .then(ub.channel.is_some().cmp(&ua.channel.is_some()))
                    .then(self.tp_eff(b).cmp(&self.tp_eff(a)))
                    .then((ua.side == Side::Enemy).cmp(&(ub.side == Side::Enemy)))
                    .then(a.cmp(&b))
            })
    }

    /// The next `n` timeline entries assuming nobody's actions change the timing.
    pub fn preview(&self, n: usize) -> Vec<TimelineEntry> {
        let mut sim: Vec<(usize, u32, bool)> = (0..self.units.len())
            .filter(|&i| self.units[i].alive())
            .map(|i| (i, self.units[i].next_act, self.units[i].channel.is_some()))
            .collect();
        let mut out = Vec::with_capacity(n);
        if let Phase::Command(active) = self.phase {
            out.push(TimelineEntry {
                unit: active,
                tick: self.clock,
                channel: false,
            });
            if let Some(entry) = sim.iter_mut().find(|e| e.0 == active) {
                entry.1 = self.predicted_next(active, 0);
            }
        }
        while out.len() < n && !sim.is_empty() {
            let pos = (0..sim.len())
                .min_by(|&a, &b| {
                    let (ea, eb) = (sim[a], sim[b]);
                    let enemy = |i: usize| self.units[i].side == Side::Enemy;
                    ea.1.cmp(&eb.1)
                        .then(eb.2.cmp(&ea.2))
                        .then(self.tp_eff(eb.0).cmp(&self.tp_eff(ea.0)))
                        .then(enemy(ea.0).cmp(&enemy(eb.0)))
                        .then(ea.0.cmp(&eb.0))
                })
                .unwrap_or(0);
            let (unit, tick, channel) = sim[pos];
            out.push(TimelineEntry {
                unit,
                tick,
                channel,
            });
            let rec = recovery(self.tp_eff(unit));
            sim[pos].1 = tick + if channel { rec / 2 } else { rec };
            sim[pos].2 = false;
        }
        out
    }

    /// Tick of `unit`'s next activation if it ended now with `extra_delay`.
    pub fn predicted_next(&self, unit: usize, extra_delay: u32) -> u32 {
        let u = &self.units[unit];
        if let Some(channel) = &u.channel {
            return channel.resolve_at;
        }
        let base = self.clock
            + recovery(self.tp_eff(unit))
            + (u.pending_delay + extra_delay).min(MAX_ACTION_DELAY);
        base.saturating_sub(u.pending_haste)
            .max(self.clock + MIN_RECOVERY / 2)
    }

    /// Advances the battle by one timeline entry. Enemy activations and channel
    /// resolutions are processed fully; a party activation stops in
    /// `Phase::Command`. Does nothing unless the phase is `Running`.
    pub fn step(&mut self, db: &GameDb) {
        if self.phase != Phase::Running {
            return;
        }
        if self.check_end() {
            return;
        }
        let Some(i) = self.next_entry() else {
            self.phase = Phase::Defeat;
            return;
        };

        // §2.7 anti-loop rule.
        if self.last_actor == Some(i) && self.streak >= MAX_CONSECUTIVE {
            let other = (0..self.units.len())
                .filter(|&j| j != i && self.units[j].alive())
                .map(|j| self.units[j].next_act)
                .min();
            if let Some(other) = other
                && other >= self.units[i].next_act
            {
                self.units[i].next_act = other + 1;
                return;
            }
        }

        let previous_cycle = self.cycle();
        self.clock = self.clock.max(self.units[i].next_act);
        if self.cycle() > previous_cycle {
            self.on_new_cycle(db);
            if self.check_end() {
                return;
            }
        }

        if self.units[i].channel.is_some() {
            self.resolve_channel(db, i);
            self.note_actor(i);
            self.check_end();
            return;
        }
        self.start_activation(db, i);
    }

    fn note_actor(&mut self, i: usize) {
        if self.last_actor == Some(i) {
            self.streak = self.streak.saturating_add(1);
        } else {
            self.streak = 1;
            self.last_actor = Some(i);
        }
    }

    fn on_new_cycle(&mut self, db: &GameDb) {
        let cycle = self.cycle();
        let Some(f) = &self.formation else {
            return;
        };
        if f.last_cycle == cycle {
            return;
        }
        let id = f.id.clone();
        let phase = f.phase;
        if let Some(fs) = &mut self.formation {
            fs.last_cycle = cycle;
        }
        if let Some(def) = db.formations.get(&id)
            && def.cycle_pulse
            && phase > 0
        {
            let pulse = def.phases[phase as usize - 1].pulse.clone();
            self.formation_effects(&pulse, 1);
        }
    }

    /// Victory / defeat checks (§8.3). Returns true if the battle ended.
    pub fn check_end(&mut self) -> bool {
        if self.is_over() {
            return true;
        }
        let party_alive = self.party().any(|i| self.units[i].fighting());
        let protected_down = self.protect.iter().any(|&i| !self.units[i].alive());
        if !party_alive || protected_down {
            self.phase = Phase::Defeat;
            self.log.push(LogEntry::Defeat);
            return true;
        }
        let won = match &self.objective {
            Objective::DefeatAll => self.enemies().all(|i| !self.units[i].fighting()),
            Objective::DefeatTarget(id) => self
                .enemies()
                .any(|i| &self.units[i].def == id && !self.units[i].alive()),
            Objective::Survive(cycles) => self.clock >= cycles * TICKS_PER_CYCLE,
            Objective::FormationPhase(n) => self.formation.as_ref().is_some_and(|f| f.phase >= *n),
        } || self.enemies().all(|i| !self.units[i].fighting());
        if won {
            self.phase = Phase::Victory;
            self.log.push(LogEntry::Victory);
            return true;
        }
        false
    }

    // ------------------------------------------------------------------
    // Activations (§3)
    // ------------------------------------------------------------------

    fn start_activation(&mut self, db: &GameDb, i: usize) {
        self.log.push(LogEntry::Turn { unit: i });
        // Summons only guard; they fade after their turns.
        if let Some(mut s) = self.units[i].summon {
            s.turns = s.turns.saturating_sub(1);
            self.units[i].summon = Some(s);
            if s.turns == 0 {
                self.units[i].hp = 0;
                self.units[i].statuses.clear();
                self.log.push(LogEntry::Dissipated { unit: i });
            }
            self.end_activation(db, i);
            return;
        }
        let wards = self.aura_active(db, |a| matches!(a, FormationAura::WardEachActivation));
        let aura_regen: u32 = self.aura_sum(db, |a| match a {
            FormationAura::LlRegen(n) => *n,
            _ => 0,
        });
        {
            let u = &mut self.units[i];
            u.remove_status(StatusKind::ThuThe);
            for s in &mut u.skills {
                s.cooldown = s.cooldown.saturating_sub(1);
            }
            for a in &mut u.artifacts {
                a.cooldown = a.cooldown.saturating_sub(1);
            }
            u.ap = (AP_PER_ACTIVATION + u.leftover_ap.min(AP_CARRY_MAX)).min(AP_MAX);
            u.leftover_ap = 0;
            u.pending_delay = 0;
            u.charged_this_activation = false;
        }
        let party_node = self.units[i].side == Side::Party && !self.units[i].node_broken;
        let regen = (self.units[i].stats.ll * LL_REGEN_PCT).div_ceil(100)
            + self.units[i].ll_regen_bonus
            + if party_node { aura_regen } else { 0 };
        self.gain_ll(i, regen, false);
        if wards && party_node && !self.units[i].has(StatusKind::HoTam) {
            self.add_status(i, StatusKind::HoTam, 0, 0, None);
        }

        // Damage over time and backlash (§3.1.5, §4.4).
        for (kind, pct) in [(StatusKind::Bong, 6), (StatusKind::Doc, 4)] {
            if self.units[i].has(kind) {
                let amount = (self.units[i].stats.hp * pct / 100).max(1);
                self.lose_hp(i, amount);
                self.log.push(LogEntry::Dot {
                    unit: i,
                    status: kind,
                    amount,
                });
            }
        }
        if self.units[i].charge >= 4 {
            let amount = (self.units[i].stats.hp * BACKLASH_PCT / 100).max(1);
            self.lose_hp(i, amount);
            self.log.push(LogEntry::Backlash { unit: i, amount });
        }
        if !self.units[i].alive() {
            self.ko(i);
            self.end_activation(db, i);
            return;
        }

        // §2.8 stun.
        if self.units[i].remove_status(StatusKind::Choang).is_some() {
            self.log.push(LogEntry::Stunned { unit: i });
            self.add_status(i, StatusKind::KienDinh, STUN_IMMUNITY_TURNS + 1, 0, None);
            self.units[i].ap = 0;
            self.end_activation(db, i);
            let half = recovery(self.tp_eff(i)) / 2;
            self.units[i].next_act = self.clock + half;
            return;
        }

        match self.units[i].side {
            Side::Party => self.phase = Phase::Command(i),
            Side::Enemy => {
                self.enemy_act(db, i);
                self.end_activation(db, i);
            }
        }
    }

    fn end_activation(&mut self, db: &GameDb, i: usize) {
        let party_node = self.units[i].side == Side::Party
            && !self.units[i].node_broken
            && self.units[i].fighting();
        {
            let u = &mut self.units[i];
            u.leftover_ap = u.ap.min(AP_CARRY_MAX);
            u.ap = 0;
            u.own_activations += 1;
            for s in &mut u.statuses {
                if s.turns > 0 && s.kind != StatusKind::ThuThe {
                    s.turns -= 1;
                }
            }
            u.statuses.retain(|s| {
                s.turns > 0
                    || matches!(
                        s.kind,
                        StatusKind::ThuThe
                            | StatusKind::Khien
                            | StatusKind::HoTam
                            | StatusKind::HuAnh
                            | StatusKind::QuaNhiet
                    )
            });
            if let Some((_, turns)) = &mut u.element_override {
                *turns = turns.saturating_sub(1);
                if *turns == 0 {
                    u.element_override = None;
                }
            }
        }
        if self.units[i].channel.is_none() {
            self.units[i].next_act = self.predicted_next(i, 0);
        }
        self.units[i].pending_delay = 0;
        self.units[i].pending_haste = 0;
        if party_node {
            self.add_energy(db, 1);
        }
        if self.units[i].side == Side::Enemy && self.units[i].alive() {
            self.choose_intent(db, i);
        }
        self.note_actor(i);
        self.activations += 1;
        if !self.check_end() {
            self.phase = Phase::Running;
        }
    }

    fn resolve_channel(&mut self, db: &GameDb, i: usize) {
        let Some(channel) = self.units[i].channel.take() else {
            return;
        };
        if let Some(skill) = db.skill(&channel.skill).cloned() {
            self.log.push(LogEntry::Skill {
                unit: i,
                skill: skill.id.clone(),
                target: match channel.target {
                    Target::Unit(t) => Some(t),
                    _ => None,
                },
            });
            self.resolve_skill(db, i, &skill, channel.target);
        }
        let half = recovery(self.tp_eff(i)) / 2;
        self.units[i].next_act = self.clock + half;
        if self.units[i].side == Side::Enemy && self.units[i].alive() {
            self.choose_intent(db, i);
        }
    }

    // ------------------------------------------------------------------
    // Commands (§3.2)
    // ------------------------------------------------------------------

    fn active(&self) -> Option<usize> {
        match self.phase {
            Phase::Command(i) => Some(i),
            _ => None,
        }
    }

    fn sealed(&self, i: usize) -> bool {
        self.units[i].has(StatusKind::PhongAn)
    }

    fn check_skill_cost(&self, i: usize, skill: &SkillDef) -> Result<(), Unavailable> {
        let u = &self.units[i];
        if u.ap < skill.ap {
            return Err(Unavailable::Ap(skill.ap));
        }
        if skill.ll > 0 && self.sealed(i) {
            return Err(Unavailable::Sealed);
        }
        if u.ll < skill.ll {
            return Err(Unavailable::Ll(skill.ll));
        }
        let consumes_charge = skill
            .effects
            .iter()
            .any(|e| matches!(e, BattleEffect::ConsumeChargeDamage { .. }));
        if consumes_charge && u.charge == 0 {
            return Err(Unavailable::NeedCharge);
        }
        Ok(())
    }

    /// Whether `command` is currently allowed for the active unit.
    pub fn can(&self, db: &GameDb, command: &Command) -> Result<(), Unavailable> {
        let i = self.active().ok_or(Unavailable::NotYourTurn)?;
        let u = &self.units[i];
        let need_ap = |ap: u8| {
            if u.ap < ap {
                Err(Unavailable::Ap(ap))
            } else {
                Ok(())
            }
        };
        match command {
            Command::Strike(t) => {
                need_ap(1)?;
                if !self.valid_targets(i, TargetKind::Enemy, true).contains(t) {
                    return Err(Unavailable::NoTarget);
                }
                Ok(())
            }
            Command::Skill(id, target) => {
                let slot = u
                    .skills
                    .iter()
                    .find(|s| &s.id == id)
                    .ok_or(Unavailable::Unknown)?;
                if slot.cooldown > 0 {
                    return Err(Unavailable::Cooldown(slot.cooldown));
                }
                let skill = db.skill(id).ok_or(Unavailable::Unknown)?;
                self.check_skill_cost(i, skill)?;
                self.check_target(i, skill.target, skill.melee, *target)
            }
            Command::Guard => need_ap(GUARD_AP),
            Command::Meditate => need_ap(MEDITATE_AP),
            Command::Charge => {
                need_ap(CHARGE_AP)?;
                if u.max_charge == 0 {
                    return Err(Unavailable::NoCharge);
                }
                if u.charged_this_activation {
                    return Err(Unavailable::ChargedThisTurn);
                }
                if u.charge >= u.max_charge {
                    return Err(Unavailable::MaxCharge);
                }
                if self.sealed(i) {
                    return Err(Unavailable::Sealed);
                }
                if u.ll < CHARGE_LL_COST {
                    return Err(Unavailable::Ll(CHARGE_LL_COST));
                }
                Ok(())
            }
            Command::Artifact(idx, target) => {
                let slot = u.artifacts.get(*idx).ok_or(Unavailable::Unknown)?;
                if slot.cooldown > 0 {
                    return Err(Unavailable::Cooldown(slot.cooldown));
                }
                if slot.charges == Some(0) {
                    return Err(Unavailable::NoCharges);
                }
                let skill = db
                    .artifacts
                    .get(&slot.id)
                    .and_then(|a| a.active.as_ref())
                    .and_then(|s| db.skill(s))
                    .ok_or(Unavailable::Unknown)?;
                self.check_skill_cost(i, skill)?;
                self.check_target(i, skill.target, skill.melee, *target)
            }
            Command::Item(id, target) => {
                need_ap(ITEM_AP)?;
                if self.items.get(id).copied().unwrap_or(0) == 0 {
                    return Err(Unavailable::NoItem);
                }
                let item = db.items.get(id).ok_or(Unavailable::Unknown)?;
                self.check_target(i, item.target, false, *target)
            }
            Command::ReleaseFormation => {
                need_ap(RELEASE_AP)?;
                let f = self.formation.as_ref().ok_or(Unavailable::NoFormation)?;
                if f.phase == 0 {
                    return Err(Unavailable::FormationNotReady);
                }
                Ok(())
            }
            Command::Stabilize => {
                need_ap(STABILIZE_AP)?;
                if self.formation.is_none() {
                    return Err(Unavailable::NoFormation);
                }
                if !u.node_broken {
                    return Err(Unavailable::NodeIntact);
                }
                Ok(())
            }
            Command::Swap(other) => {
                need_ap(self.swap_cost(db, i))?;
                let ok = *other != i
                    && self.units.get(*other).is_some_and(|o| {
                        o.side == Side::Party
                            && o.alive()
                            && o.slot.rank().abs_diff(u.slot.rank()) == 1
                    });
                if ok {
                    Ok(())
                } else {
                    Err(Unavailable::NoTarget)
                }
            }
            Command::Flee => {
                if !self.can_flee {
                    return Err(Unavailable::CannotFlee);
                }
                need_ap(FLEE_AP)
            }
            Command::EndTurn => Ok(()),
        }
    }

    fn check_target(
        &self,
        i: usize,
        kind: TargetKind,
        melee: bool,
        target: Target,
    ) -> Result<(), Unavailable> {
        let ok = match (kind, target) {
            (TargetKind::Enemy | TargetKind::Ally, Target::Unit(t)) => {
                self.valid_targets(i, kind, melee).contains(&t)
            }
            (TargetKind::AllEnemies, Target::Opponents) => !self.opponents_of(i).is_empty(),
            (TargetKind::AllAllies, Target::Allies) => true,
            (TargetKind::SelfOnly, Target::Myself) => true,
            _ => false,
        };
        if ok {
            Ok(())
        } else {
            Err(Unavailable::NoTarget)
        }
    }

    /// Units `actor` may pick for a single-target technique (§8.4 melee reach, taunt).
    pub fn valid_targets(&self, actor: usize, kind: TargetKind, melee: bool) -> Vec<usize> {
        match kind {
            TargetKind::Ally => self.allies_of(actor),
            TargetKind::SelfOnly => vec![actor],
            TargetKind::Enemy => {
                let foes = self.opponents_of(actor);
                let taunting: Vec<usize> = foes
                    .iter()
                    .copied()
                    .filter(|&f| self.units[f].has(StatusKind::KhieuKhich))
                    .collect();
                if !taunting.is_empty() {
                    return taunting;
                }
                if melee {
                    let front = foes.iter().map(|&f| self.units[f].slot.rank()).min();
                    foes.into_iter()
                        .filter(|&f| Some(self.units[f].slot.rank()) == front)
                        .collect()
                } else {
                    foes
                }
            }
            TargetKind::AllEnemies => self.opponents_of(actor),
            TargetKind::AllAllies => self.allies_of(actor),
        }
    }

    /// The default target for a technique kind (UI convenience).
    pub fn default_target(&self, actor: usize, kind: TargetKind, melee: bool) -> Option<Target> {
        match kind {
            TargetKind::Enemy | TargetKind::Ally => self
                .valid_targets(actor, kind, melee)
                .first()
                .map(|&t| Target::Unit(t)),
            TargetKind::AllEnemies => Some(Target::Opponents),
            TargetKind::AllAllies => Some(Target::Allies),
            TargetKind::SelfOnly => Some(Target::Myself),
        }
    }

    /// Executes a command for the active party unit.
    pub fn execute(&mut self, db: &GameDb, command: Command) -> Result<(), Unavailable> {
        self.can(db, &command)?;
        let i = self.active().ok_or(Unavailable::NotYourTurn)?;
        let mut end = false;
        match command {
            Command::Strike(t) => {
                self.units[i].ap -= 1;
                self.log.push(LogEntry::Strike { unit: i, target: t });
                let element = self.units[i].current_element();
                let combo = self.combo_check(db, i, element);
                self.deal_damage(
                    db,
                    i,
                    t,
                    &Hit {
                        power: STRIKE_POWER,
                        scaling: Scaling::Atk,
                        element,
                        melee: true,
                        mult: 100,
                        combo,
                        interrupt: false,
                        single: true,
                    },
                );
                self.gain_ll(i, STRIKE_LL_GAIN, true);
            }
            Command::Skill(id, target) => {
                let Some(skill) = db.skill(&id).cloned() else {
                    return Err(Unavailable::Unknown);
                };
                if let Some(slot) = self.units[i].skills.iter_mut().find(|s| s.id == id) {
                    slot.cooldown = skill.cooldown;
                }
                end = self.use_skill(db, i, &skill, target);
            }
            Command::Guard => {
                self.units[i].ap -= GUARD_AP;
                self.add_status(i, StatusKind::ThuThe, 0, 0, None);
                self.log.push(LogEntry::Guard { unit: i });
                end = true;
            }
            Command::Meditate => {
                self.units[i].ap -= MEDITATE_AP;
                let amount = self.units[i].stats.ll * MEDITATE_PCT / 100;
                let gained = self.gain_ll(i, amount, false);
                self.log.push(LogEntry::Meditate {
                    unit: i,
                    amount: gained,
                });
                end = true;
            }
            Command::Charge => {
                let u = &mut self.units[i];
                u.ap -= CHARGE_AP;
                u.ll -= CHARGE_LL_COST;
                u.charged_this_activation = true;
                if u.charge == 3 {
                    let cost = u.stats.hp * OVERCHARGE_HP_PCT / 100;
                    u.hp = u.hp.saturating_sub(cost).max(1);
                }
                u.charge += 1;
                let stage = u.charge;
                self.log.push(LogEntry::Charge { unit: i, stage });
            }
            Command::Artifact(idx, target) => {
                let slot = self.units[i].artifacts[idx].clone();
                let Some(skill) = db
                    .artifacts
                    .get(&slot.id)
                    .and_then(|a| a.active.as_ref())
                    .and_then(|s| db.skill(s))
                    .cloned()
                else {
                    return Err(Unavailable::Unknown);
                };
                let art = &mut self.units[i].artifacts[idx];
                art.cooldown = skill.cooldown;
                if let Some(c) = &mut art.charges {
                    *c -= 1;
                }
                self.log.push(LogEntry::Artifact {
                    unit: i,
                    artifact: slot.id.clone(),
                });
                end = self.use_skill(db, i, &skill, target);
            }
            Command::Item(id, target) => {
                self.units[i].ap -= ITEM_AP;
                if let Some(n) = self.items.get_mut(&id) {
                    *n -= 1;
                }
                *self.items_used.entry(id.clone()).or_insert(0) += 1;
                self.log.push(LogEntry::Item {
                    unit: i,
                    item: id.clone(),
                });
                if let Some(item) = db.items.get(&id) {
                    let effects = item.battle_use.clone();
                    for t in self.resolve_targets(i, target) {
                        for effect in &effects {
                            self.apply_effect(
                                db,
                                i,
                                t,
                                effect,
                                100,
                                Element::Vo,
                                false,
                                false,
                                true,
                            );
                        }
                    }
                }
            }
            Command::ReleaseFormation => {
                self.units[i].ap -= RELEASE_AP;
                self.release_formation(db);
            }
            Command::Stabilize => {
                self.units[i].ap -= STABILIZE_AP;
                self.units[i].node_broken = false;
                self.log.push(LogEntry::NodeRestored { unit: i });
            }
            Command::Swap(other) => {
                self.units[i].ap -= self.swap_cost(db, i);
                let (a, b) = (self.units[i].slot, self.units[other].slot);
                self.units[i].slot = b;
                self.units[other].slot = a;
                self.log.push(LogEntry::Swap { a: i, b: other });
            }
            Command::Flee => {
                self.units[i].ap -= FLEE_AP;
                let avg = |idx: Vec<usize>| {
                    let n = idx.len().max(1) as i64;
                    idx.iter().map(|&j| self.tp_eff(j) as i64).sum::<i64>() / n
                };
                let party: Vec<usize> = self.allies_of(i);
                let foes: Vec<usize> = self.opponents_of(i);
                let chance = (50 + avg(party) - avg(foes)).clamp(20, 90) as u32;
                if self.rng.chance(chance) {
                    self.phase = Phase::Fled;
                    self.log.push(LogEntry::Fled);
                    return Ok(());
                }
                self.log.push(LogEntry::FleeFailed);
                end = true;
            }
            Command::EndTurn => end = true,
        }
        if self.check_end() {
            return Ok(());
        }
        if !self.units[i].alive() || end || self.units[i].ap == 0 || self.units[i].channel.is_some()
        {
            self.end_activation(db, i);
        }
        Ok(())
    }

    /// Pays costs and resolves or starts channelling a technique.
    /// Returns true if the activation must end.
    pub(super) fn use_skill(
        &mut self,
        db: &GameDb,
        i: usize,
        skill: &SkillDef,
        target: Target,
    ) -> bool {
        {
            let u = &mut self.units[i];
            if u.side == Side::Party {
                u.ap = u.ap.saturating_sub(skill.ap);
            }
            u.ll = u.ll.saturating_sub(skill.ll);
            u.pending_delay += skill.delay;
        }
        if let Some(windup) = skill.windup {
            let at = self.clock + windup;
            self.units[i].channel = Some(Channel {
                skill: skill.id.clone(),
                target,
                resolve_at: at,
            });
            self.units[i].next_act = at;
            self.log.push(LogEntry::Channel {
                unit: i,
                skill: skill.id.clone(),
                at,
            });
            return true;
        }
        self.log.push(LogEntry::Skill {
            unit: i,
            skill: skill.id.clone(),
            target: match target {
                Target::Unit(t) => Some(t),
                _ => None,
            },
        });
        self.resolve_skill(db, i, skill, target);
        skill.ends_turn
    }

    fn resolve_targets(&self, actor: usize, target: Target) -> Vec<usize> {
        match target {
            Target::Unit(t) => {
                if self.units[t].alive() {
                    vec![t]
                } else {
                    // Re-target to the nearest valid unit on the same side.
                    let side = self.units[t].side;
                    (0..self.units.len())
                        .filter(|&j| self.units[j].side == side && self.units[j].alive())
                        .min_by_key(|&j| self.units[j].slot.rank())
                        .into_iter()
                        .collect()
                }
            }
            Target::Opponents => self.opponents_of(actor),
            Target::Allies => self.allies_of(actor),
            Target::Myself => vec![actor],
        }
    }

    fn resolve_skill(&mut self, db: &GameDb, i: usize, skill: &SkillDef, target: Target) {
        let stage = if skill.chargeable {
            std::mem::take(&mut self.units[i].charge)
        } else {
            0
        };
        if stage > 0 {
            self.log.push(LogEntry::ChargeReleased { unit: i, stage });
            if stage >= 4 {
                self.overcharge_releases += 1;
            }
        }
        let mult = CHARGE_MULT[stage as usize];
        let damages = skill.effects.iter().any(|e| {
            matches!(
                e,
                BattleEffect::Damage { .. } | BattleEffect::ConsumeChargeDamage { .. }
            )
        });
        let combo = damages && self.combo_check(db, i, skill.element);
        let single = matches!(target, Target::Unit(_));
        let targets = self.resolve_targets(i, target);
        let mut effects: Vec<&BattleEffect> = skill.effects.iter().collect();
        for bonus in &skill.charge_bonus {
            if stage >= bonus.stage {
                effects.extend(bonus.effects.iter());
            }
        }
        for t in targets {
            for effect in &effects {
                if !self.units[t].alive() && !matches!(effect, BattleEffect::ReleaseStoredLl) {
                    break;
                }
                self.apply_effect(
                    db,
                    i,
                    t,
                    effect,
                    mult,
                    skill.element,
                    skill.melee,
                    combo,
                    single,
                );
            }
        }
    }

    /// §7.2 Tương sinh: records the element and returns whether this action combos.
    fn combo_check(&mut self, db: &GameDb, i: usize, element: Element) -> bool {
        if self.units[i].side != Side::Party || element == Element::Vo {
            return false;
        }
        let combo = self.last_party_element.is_some_and(|(prev, at)| {
            self.clock.saturating_sub(at) <= COMBO_WINDOW && prev.generates() == Some(element)
        });
        if combo {
            let from = self.last_party_element.map(|(e, _)| e).unwrap_or_default();
            self.log.push(LogEntry::Combo { from, to: element });
            let bonus = self.aura_sum(db, |a| match a {
                FormationAura::ComboEnergy(n) => *n,
                _ => 0,
            });
            self.add_energy(db, 1 + bonus);
        }
        self.last_party_element = Some((element, self.clock));
        combo
    }

    // ------------------------------------------------------------------
    // Effects (§7)
    // ------------------------------------------------------------------

    fn stat(&self, db: &GameDb, i: usize, scaling: Scaling) -> u32 {
        let u = &self.units[i];
        let mut value = match scaling {
            Scaling::Atk => u.stats.atk,
            Scaling::Spi => u.stats.spi,
        };
        if u.has(StatusKind::CuongCong) {
            value = value * 130 / 100;
        }
        if u.side == Side::Party && !u.node_broken {
            let pct = self.aura_sum(db, |a| match a {
                FormationAura::AtkPct(n) => *n,
                _ => 0,
            });
            value = value * (100 + pct) / 100;
        }
        value
    }

    fn def_eff(&self, db: &GameDb, i: usize) -> u32 {
        let u = &self.units[i];
        let mut def = u.stats.def;
        if u.has(StatusKind::PhaGiap) {
            def = def * 60 / 100;
        }
        if u.side == Side::Party && !u.node_broken {
            let pct = self.aura_sum(db, |a| match a {
                FormationAura::DefPct(n) => *n,
                _ => 0,
            });
            def = def * (100 + pct) / 100;
        }
        def
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_effect(
        &mut self,
        db: &GameDb,
        actor: usize,
        t: usize,
        effect: &BattleEffect,
        mult: u32,
        element: Element,
        melee: bool,
        combo: bool,
        single: bool,
    ) {
        match effect {
            BattleEffect::Damage {
                power,
                scaling,
                interrupt,
                hits,
                bonus_hit_if_faster,
            } => {
                let mut n = *hits as u32;
                if *bonus_hit_if_faster && self.tp_eff(actor) > self.tp_eff(t) {
                    n += 1;
                }
                let hit = Hit {
                    power: *power,
                    scaling: *scaling,
                    element,
                    melee,
                    mult,
                    combo,
                    interrupt: *interrupt,
                    single,
                };
                for _ in 0..n {
                    if !self.units[t].alive() || !self.deal_damage(db, actor, t, &hit) {
                        break;
                    }
                }
            }
            BattleEffect::ConsumeChargeDamage {
                power_per_stage,
                scaling,
            } => {
                let stage = std::mem::take(&mut self.units[actor].charge).max(1) as u32;
                self.log.push(LogEntry::ChargeReleased {
                    unit: actor,
                    stage: stage as u8,
                });
                let hit = Hit {
                    power: power_per_stage * stage,
                    scaling: *scaling,
                    element,
                    melee,
                    mult: 100,
                    combo,
                    interrupt: false,
                    single,
                };
                self.deal_damage(db, actor, t, &hit);
                if self.units[actor].overheat {
                    self.overheat(actor);
                }
            }
            BattleEffect::Heal { power, scaling } => {
                let amount = power * self.stat(db, actor, *scaling) / 100 * mult / 100;
                self.heal(t, amount);
            }
            BattleEffect::HealPct(pct) => {
                let amount = self.units[t].stats.hp * pct / 100 * mult / 100;
                self.heal(t, amount);
            }
            BattleEffect::RestoreLl(pct) => {
                let amount = self.units[t].stats.ll * pct / 100;
                let gained = self.gain_ll(t, amount, false);
                self.log.push(LogEntry::LlGain {
                    unit: t,
                    amount: gained,
                });
            }
            BattleEffect::DrainLl(amount) => {
                let drained = (*amount).min(self.units[t].ll);
                self.units[t].ll -= drained;
                self.log.push(LogEntry::Drain {
                    target: t,
                    amount: drained,
                });
                let share = drained.div_ceil(2);
                for ally in self.allies_of(actor) {
                    self.gain_ll(ally, share, false);
                }
            }
            BattleEffect::Shield {
                power,
                scaling,
                flat,
            } => {
                let spi = self.stat(db, actor, Scaling::Spi);
                let amount = (power * self.stat(db, actor, *scaling) / 100 + flat) * mult / 100;
                let cap = 2 * spi + 60;
                let current = self.units[t].shield();
                let value = (current + amount).min(cap.max(amount));
                self.units[t].remove_status(StatusKind::Khien);
                self.add_status(t, StatusKind::Khien, 0, value, Some(actor));
                self.log.push(LogEntry::Shield {
                    target: t,
                    amount: value - current.min(value),
                });
            }
            BattleEffect::Status { id, turns, chance } => {
                if let Some(chance) = chance
                    && !self.rng.chance(*chance as u32)
                {
                    self.log.push(LogEntry::StatusResisted {
                        target: t,
                        status: *id,
                    });
                    return;
                }
                if *id == StatusKind::Choang && self.units[t].has(StatusKind::KienDinh) {
                    self.log.push(LogEntry::StatusResisted {
                        target: t,
                        status: *id,
                    });
                    return;
                }
                if *id == StatusKind::Choang {
                    self.break_charge(t, true);
                }
                self.add_status(t, *id, *turns, 0, Some(actor));
                self.log.push(LogEntry::Status {
                    target: t,
                    status: *id,
                });
            }
            BattleEffect::Cleanse => {
                self.units[t].statuses.retain(|s| !s.kind.is_debuff());
                self.log.push(LogEntry::Cleansed { target: t });
            }
            BattleEffect::Delay(ticks) => self.push(t, *ticks),
            BattleEffect::Haste(ticks) => {
                let clock = self.clock;
                let acting = self.phase == Phase::Command(t)
                    || (t == actor && self.units[t].next_act <= clock);
                let u = &mut self.units[t];
                let pulled = if acting {
                    // The acting unit's next turn is computed when it ends: shorten that.
                    u.pending_haste += ticks;
                    *ticks
                } else if u.channel.is_none() {
                    let before = u.next_act;
                    u.next_act = u.next_act.saturating_sub(*ticks).max(clock + 1).min(before);
                    before - u.next_act
                } else {
                    0
                };
                self.log.push(LogEntry::Pulled {
                    target: t,
                    ticks: pulled,
                });
            }
            BattleEffect::Interrupt => self.interrupt(t),
            BattleEffect::GainCharge(n) => {
                let u = &mut self.units[t];
                u.charge = (u.charge + n).min(u.max_charge.min(3).max(u.charge));
                let stage = u.charge;
                self.log.push(LogEntry::Charge { unit: t, stage });
            }
            BattleEffect::BreakNode => {
                if self.units[t].side == Side::Party
                    && self.units[t].summon.is_none()
                    && self.formation.is_some()
                    && !self.units[t].node_broken
                {
                    self.units[t].node_broken = true;
                    self.log.push(LogEntry::NodeBroken { unit: t });
                    if let Some(f) = &mut self.formation {
                        f.energy = f.energy.saturating_sub(2);
                    }
                    self.recompute_phase(db, false);
                    if self.formation_chaos() {
                        self.log.push(LogEntry::FormationChaos);
                    }
                }
            }
            BattleEffect::FormationEnergy(n) => self.add_energy(db, *n),
            BattleEffect::SetElement { element, turns } => {
                self.units[t].element_override = Some((*element, *turns));
                self.log.push(LogEntry::ElementSet {
                    target: t,
                    element: *element,
                });
            }
            BattleEffect::StunIfChanneling { else_power } => {
                if self.units[t].channel.is_some() {
                    self.interrupt(t);
                    if self.units[t].has(StatusKind::KienDinh) {
                        self.log.push(LogEntry::StatusResisted {
                            target: t,
                            status: StatusKind::Choang,
                        });
                    } else {
                        self.add_status(t, StatusKind::Choang, 1, 0, Some(actor));
                        self.log.push(LogEntry::Status {
                            target: t,
                            status: StatusKind::Choang,
                        });
                    }
                } else {
                    let hit = Hit {
                        power: *else_power,
                        scaling: Scaling::Atk,
                        element: Element::Tho,
                        melee,
                        mult,
                        combo,
                        interrupt: false,
                        single,
                    };
                    self.deal_damage(db, actor, t, &hit);
                }
            }
            BattleEffect::ReleaseStoredLl => {
                let stored = std::mem::take(&mut self.units[actor].stored_ll);
                let gained = self.gain_ll(actor, stored, false);
                self.log.push(LogEntry::LlGain {
                    unit: actor,
                    amount: gained,
                });
            }
            BattleEffect::Summon(id) => {
                if let Some(def) = db.summons.get(id).cloned() {
                    self.summon(db, actor, &def);
                }
            }
        }
    }

    /// Calls a spirit onto `owner`'s side, replacing its previous summon.
    fn summon(&mut self, db: &GameDb, owner: usize, def: &SummonDef) {
        for j in 0..self.units.len() {
            if self.units[j].summon.is_some_and(|s| s.owner == owner) && self.units[j].alive() {
                self.units[j].hp = 0;
                self.log.push(LogEntry::Dissipated { unit: j });
            }
        }
        // Start from the summoner so every field has a sane value.
        let mut u = self.units[owner].clone();
        u.def = def.id.clone();
        u.sheet = def.sheet;
        u.sprite = def.sprite.clone();
        u.tint = def.tint;
        u.stats = Stats {
            hp: (u.stats.hp * def.hp_pct / 100).max(1),
            ll: 0,
            atk: 0,
            spi: 0,
            def: u.stats.def * def.def_pct / 100,
            tp: u.stats.tp,
        };
        u.hp = u.stats.hp;
        u.ll = 0;
        u.element = Element::Vo;
        u.element_override = None;
        u.slot = Slot::Front;
        u.charge = 0;
        u.max_charge = 0;
        u.statuses.clear();
        u.skills.clear();
        u.artifacts.clear();
        u.stored_ll = 0;
        u.store_cap = 0;
        u.ll_regen_bonus = 0;
        u.tp_bonus = 0;
        u.channel = None;
        u.intent = None;
        u.own_activations = 0;
        u.pushed = (0, 0);
        u.push_resist = 0;
        u.boss = false;
        u.invulnerable = false;
        u.guest = false;
        u.node_broken = false;
        u.shield_to_energy = false;
        u.overheat = false;
        u.pending_delay = 0;
        u.pending_haste = 0;
        u.ap = 0;
        u.leftover_ap = 0;
        u.summon = Some(SummonState {
            turns: def.turns,
            owner,
        });
        u.next_act = self.clock + recovery(u.stats.tp) / 2;
        let index = self.units.len();
        self.units.push(u);
        if def.taunt {
            // Outlasts the spirit so it guards until it fades.
            self.add_status(index, StatusKind::KhieuKhich, def.turns + 1, 0, None);
        }
        self.log.push(LogEntry::Summoned { unit: index, owner });
        // Telegraphed attacks that can no longer reach their target pick a new one.
        let side = self.units[owner].side;
        for f in 0..self.units.len() {
            if self.units[f].side == side
                || self.units[f].side != Side::Enemy
                || !self.units[f].alive()
            {
                continue;
            }
            let stale = self.units[f].intent.as_ref().is_some_and(|it| {
                it.target.is_some_and(|t| {
                    db.skill(&it.skill)
                        .is_some_and(|s| !self.valid_targets(f, s.target, s.melee).contains(&t))
                })
            });
            if stale {
                self.choose_intent(db, f);
            }
        }
    }

    /// One damage instance (§7.1). Returns false if the attack was dodged.
    fn deal_damage(&mut self, db: &GameDb, a: usize, t: usize, hit: &Hit) -> bool {
        if hit.single && self.units[t].remove_status(StatusKind::HuAnh).is_some() {
            self.log.push(LogEntry::Miss { target: t });
            return false;
        }
        let raw = hit.power as i64 * self.stat(db, a, hit.scaling) as i64 / 100;
        let mut dmg = raw * 100 / (100 + self.def_eff(db, t) as i64);
        dmg = dmg * hit.mult as i64 / 100;
        let (ae, te) = (hit.element, self.units[t].current_element());
        let effective: i8 = if ae != Element::Vo && ae.overcomes() == Some(te) {
            dmg = dmg * 130 / 100;
            1
        } else if te != Element::Vo && te.overcomes() == Some(ae) {
            dmg = dmg * 80 / 100;
            -1
        } else {
            0
        };
        if hit.combo {
            dmg = dmg * 120 / 100;
        }
        if self.units[t].has(StatusKind::ThuThe) {
            dmg = dmg * 50 / 100;
        }
        if hit.melee && self.units[a].slot == Slot::Back {
            dmg = dmg * 75 / 100;
        }
        dmg = dmg * self.rng.range(95, 105) as i64 / 100;
        let mut dmg = dmg.max(1) as u32;

        // Shields absorb first, except interrupt-type hits (§6.2 Huyền Quy Thuẫn weakness).
        let mut absorbed = 0;
        if !hit.interrupt
            && let Some(pos) = self.units[t]
                .statuses
                .iter()
                .position(|s| s.kind == StatusKind::Khien)
        {
            let shield = &mut self.units[t].statuses[pos];
            absorbed = shield.value.min(dmg);
            shield.value -= absorbed;
            let source = shield.source;
            if shield.value == 0 {
                self.units[t].statuses.remove(pos);
            }
            dmg -= absorbed;
            if let Some(src) = source
                && self.units[src].shield_to_energy
                && absorbed > 0
            {
                self.add_energy(db, (absorbed / 10).min(3));
            }
        }
        let max_hp = self.units[t].stats.hp;
        self.lose_hp(t, dmg);
        self.log.push(LogEntry::Damage {
            target: t,
            amount: dmg,
            absorbed,
            effective,
        });
        if self.units[t].charge > 0 && (hit.interrupt || dmg * 100 >= max_hp * CHARGE_BREAK_PCT) {
            self.break_charge(t, false);
        }
        if hit.interrupt && self.units[t].channel.is_some() {
            self.cancel_channel(t);
        }
        if !self.units[t].alive() {
            self.ko(t);
        }
        true
    }

    fn lose_hp(&mut self, t: usize, amount: u32) {
        let u = &mut self.units[t];
        let floor = if u.invulnerable { 1 } else { 0 };
        u.hp = u.hp.saturating_sub(amount).max(floor);
    }

    fn heal(&mut self, t: usize, amount: u32) {
        let mut amount = amount;
        if self.units[t].has(StatusKind::Doc) {
            amount /= 2;
        }
        let u = &mut self.units[t];
        let before = u.hp;
        u.hp = (u.hp + amount).min(u.stats.hp);
        let healed = u.hp - before;
        self.log.push(LogEntry::Heal {
            target: t,
            amount: healed,
        });
    }

    /// Adds Linh lực; overflow goes into a storing artifact. Returns the amount gained.
    fn gain_ll(&mut self, t: usize, amount: u32, log: bool) -> u32 {
        let u = &mut self.units[t];
        let room = u.stats.ll - u.ll.min(u.stats.ll);
        let gained = amount.min(room);
        u.ll += gained;
        let overflow = amount - gained;
        if overflow > 0 && u.store_cap > 0 {
            u.stored_ll = (u.stored_ll + overflow).min(u.store_cap);
        }
        if log && gained > 0 {
            self.log.push(LogEntry::LlGain {
                unit: t,
                amount: gained,
            });
        }
        gained
    }

    fn add_status(
        &mut self,
        t: usize,
        kind: StatusKind,
        turns: u8,
        value: u32,
        source: Option<usize>,
    ) {
        let u = &mut self.units[t];
        if let Some(existing) = u.statuses.iter_mut().find(|s| s.kind == kind) {
            existing.turns = existing.turns.max(turns);
            existing.value = existing.value.max(value);
            existing.source = source.or(existing.source);
        } else {
            u.statuses.push(StatusInst {
                kind,
                turns,
                value,
                source,
            });
        }
    }

    /// §4.3 charge break.
    fn break_charge(&mut self, t: usize, full: bool) {
        let u = &mut self.units[t];
        if u.charge == 0 {
            return;
        }
        let protected =
            !full && (u.remove_status(StatusKind::HoTam).is_some() || u.has(StatusKind::ThuThe));
        let lost = if protected { 1 } else { u.charge };
        u.charge -= lost;
        self.log.push(LogEntry::ChargeBroken { unit: t, lost });
    }

    fn cancel_channel(&mut self, t: usize) {
        if self.units[t].channel.take().is_some() {
            self.log.push(LogEntry::ChannelInterrupted { unit: t });
            let rec = recovery(self.tp_eff(t));
            self.units[t].next_act = self.clock + rec / 2;
            self.push(t, INTERRUPT_STAGGER);
            if self.units[t].side == Side::Enemy {
                self.choose_intent_after_interrupt(t);
            }
        }
    }

    fn choose_intent_after_interrupt(&mut self, t: usize) {
        // Keep the intent visible but drop the cancelled channel's target hint.
        if let Some(intent) = &mut self.units[t].intent {
            intent.target = None;
        }
    }

    fn interrupt(&mut self, t: usize) {
        self.cancel_channel(t);
        self.break_charge(t, false);
    }

    /// §2.6 push with per-cycle cap and boss resistance.
    fn push(&mut self, t: usize, ticks: u32) {
        let cycle = self.cycle();
        let u = &mut self.units[t];
        let mut amount = ticks;
        if u.boss {
            amount >>= u.push_resist;
            u.push_resist = (u.push_resist + 1).min(BOSS_PUSH_RESIST_MAX);
        }
        if u.pushed.0 != cycle {
            u.pushed = (cycle, 0);
        }
        let allowed = PUSH_CAP_PER_CYCLE.saturating_sub(u.pushed.1).min(amount);
        if allowed == 0 {
            self.log.push(LogEntry::PushResisted { target: t });
            return;
        }
        u.pushed.1 += allowed;
        u.next_act += allowed;
        if let Some(channel) = &mut u.channel {
            channel.resolve_at = u.next_act;
        }
        self.log.push(LogEntry::Pushed {
            target: t,
            ticks: allowed,
        });
    }

    fn overheat(&mut self, t: usize) {
        self.add_status(t, StatusKind::QuaNhiet, 0, 0, None);
        let stacks = {
            let s = self.units[t]
                .statuses
                .iter_mut()
                .find(|s| s.kind == StatusKind::QuaNhiet)
                .expect("just added");
            s.value += 1;
            s.value
        };
        if stacks >= 3 {
            self.units[t].remove_status(StatusKind::QuaNhiet);
            let amount = self.units[t].stats.hp * 15 / 100;
            self.lose_hp(t, amount);
            self.log.push(LogEntry::Overheat { unit: t, amount });
            if !self.units[t].alive() {
                self.ko(t);
            }
        }
    }

    fn ko(&mut self, t: usize) {
        let u = &mut self.units[t];
        u.hp = 0;
        u.charge = 0;
        u.channel = None;
        u.intent = None;
        u.statuses.clear();
        if !self
            .log
            .iter()
            .rev()
            .take(4)
            .any(|e| *e == LogEntry::Ko { unit: t })
        {
            self.log.push(LogEntry::Ko { unit: t });
        }
    }

    // ------------------------------------------------------------------
    // Formations (§5)
    // ------------------------------------------------------------------

    /// Đổi vị trí is free for formation nodes under a `FreeSwap` aura.
    pub fn swap_cost(&self, db: &GameDb, i: usize) -> u8 {
        let node = self.units[i].side == Side::Party && !self.units[i].node_broken;
        if node && self.aura_active(db, |a| matches!(a, FormationAura::FreeSwap)) {
            0
        } else {
            SWAP_AP
        }
    }

    fn formation_def<'a>(&self, db: &'a GameDb) -> Option<&'a FormationDef> {
        self.formation
            .as_ref()
            .and_then(|f| db.formations.get(&f.id))
    }

    /// Half or more of the living nodes broken (§5.4).
    pub fn formation_chaos(&self) -> bool {
        let nodes: Vec<usize> = self.party().filter(|&i| self.units[i].fighting()).collect();
        let broken = nodes.iter().filter(|&&i| self.units[i].node_broken).count();
        !nodes.is_empty() && broken * 2 >= nodes.len().max(1) && broken > 0
    }

    fn auras<'a>(&self, db: &'a GameDb) -> Vec<&'a FormationAura> {
        let (Some(f), Some(def)) = (&self.formation, self.formation_def(db)) else {
            return Vec::new();
        };
        if self.formation_chaos() {
            return Vec::new();
        }
        def.phases
            .iter()
            .take(f.phase as usize)
            .flat_map(|p| p.aura.iter())
            .collect()
    }

    fn aura_active(&self, db: &GameDb, pred: impl Fn(&FormationAura) -> bool) -> bool {
        self.auras(db).into_iter().any(pred)
    }

    fn aura_sum(&self, db: &GameDb, f: impl Fn(&FormationAura) -> u32) -> u32 {
        self.auras(db).into_iter().map(f).sum()
    }

    fn add_energy(&mut self, db: &GameDb, n: u32) {
        if n == 0 || self.formation.is_none() || self.formation_chaos() {
            return;
        }
        let Some(def) = self.formation_def(db) else {
            return;
        };
        let cap = def.phases.last().map_or(0, |p| p.threshold);
        if let Some(f) = &mut self.formation {
            let before = f.energy;
            f.energy = (f.energy + n).min(cap);
            if f.energy > before {
                self.log.push(LogEntry::FormationEnergy {
                    amount: f.energy - before,
                });
            }
        }
        self.recompute_phase(db, true);
    }

    fn recompute_phase(&mut self, db: &GameDb, pulse: bool) {
        let Some(def) = self.formation_def(db).cloned() else {
            return;
        };
        let Some(f) = &self.formation else {
            return;
        };
        let target = def
            .phases
            .iter()
            .filter(|p| f.energy >= p.threshold)
            .count() as u8;
        let current = f.phase;
        if let Some(f) = &mut self.formation {
            f.phase = target;
        }
        if target > current && pulse {
            for phase in current..target {
                self.log.push(LogEntry::FormationPhase { phase: phase + 1 });
                let effects = def.phases[phase as usize].pulse.clone();
                self.formation_effects(&effects, 1);
            }
        }
    }

    fn release_formation(&mut self, db: &GameDb) {
        let Some(def) = self.formation_def(db).cloned() else {
            return;
        };
        let phase = self.formation.as_ref().map_or(0, |f| f.phase);
        self.log.push(LogEntry::FormationReleased { phase });
        self.formation_effects(&def.release, phase as u32);
        if let Some(f) = &mut self.formation {
            f.energy = 0;
            f.phase = 0;
        }
    }

    fn formation_effects(&mut self, effects: &[FormationEffect], scale: u32) {
        // Summons are not formation nodes.
        let party: Vec<usize> = self.party().filter(|&i| self.units[i].fighting()).collect();
        let enemies: Vec<usize> = self.enemies().filter(|&i| self.units[i].alive()).collect();
        for effect in effects {
            match effect {
                FormationEffect::ShieldAllPct(pct) => {
                    for &i in &party {
                        let amount = self.units[i].stats.hp * pct * scale / 100;
                        let value = self.units[i].shield() + amount;
                        self.units[i].remove_status(StatusKind::Khien);
                        self.add_status(i, StatusKind::Khien, 0, value, None);
                        self.log.push(LogEntry::Shield { target: i, amount });
                    }
                }
                FormationEffect::HealAllPct(pct) => {
                    for &i in &party {
                        let amount = self.units[i].stats.hp * pct * scale / 100;
                        self.heal(i, amount);
                    }
                }
                FormationEffect::CleanseAll => {
                    for &i in &party {
                        self.units[i].statuses.retain(|s| !s.kind.is_debuff());
                        self.log.push(LogEntry::Cleansed { target: i });
                    }
                }
                FormationEffect::RestoreLlAllPct(pct) => {
                    for &i in &party {
                        let amount = self.units[i].stats.ll * pct * scale / 100;
                        let gained = self.gain_ll(i, amount, false);
                        self.log.push(LogEntry::LlGain {
                            unit: i,
                            amount: gained,
                        });
                    }
                }
                FormationEffect::StrikeAll(power) => {
                    for &i in &party {
                        if self.units[i].node_broken {
                            continue;
                        }
                        let Some(&t) = self.valid_targets(i, TargetKind::Enemy, false).first()
                        else {
                            break;
                        };
                        let hit = Hit {
                            power: power * scale,
                            scaling: Scaling::Atk,
                            element: self.units[i].current_element(),
                            melee: false,
                            mult: 100,
                            combo: false,
                            interrupt: false,
                            single: false,
                        };
                        // Formation strikes ignore formation-energy feedback loops.
                        self.deal_damage_raw(i, t, &hit);
                    }
                }
                FormationEffect::InterruptAllEnemies => {
                    for &e in &enemies {
                        self.interrupt(e);
                    }
                }
                FormationEffect::StatusAllEnemies(kind, turns) => {
                    for &e in &enemies {
                        self.add_status(e, *kind, *turns, 0, None);
                        self.log.push(LogEntry::Status {
                            target: e,
                            status: *kind,
                        });
                    }
                }
                FormationEffect::StatusRow(slot, kind, turns) => {
                    for &i in &party {
                        if self.units[i].slot != *slot {
                            continue;
                        }
                        self.add_status(i, *kind, *turns, 0, None);
                        self.log.push(LogEntry::Status {
                            target: i,
                            status: *kind,
                        });
                    }
                }
                FormationEffect::StatusAllAllies(kind, turns) => {
                    for &i in &party {
                        self.add_status(i, *kind, *turns, 0, None);
                        self.log.push(LogEntry::Status {
                            target: i,
                            status: *kind,
                        });
                    }
                }
                FormationEffect::HasteAll(ticks) => {
                    for &i in &party {
                        let clock = self.clock;
                        let u = &mut self.units[i];
                        let before = u.next_act;
                        if u.channel.is_none() && before > clock {
                            u.next_act = before.saturating_sub(ticks * scale).max(clock + 1);
                        }
                        let pulled = before - u.next_act;
                        self.log.push(LogEntry::Pulled {
                            target: i,
                            ticks: pulled,
                        });
                    }
                }
                FormationEffect::DelayAllEnemies(ticks) => {
                    for &e in &enemies {
                        self.push(e, ticks * scale);
                    }
                }
            }
        }
    }

    /// Damage without auras/shield feedback (formation strikes).
    fn deal_damage_raw(&mut self, a: usize, t: usize, hit: &Hit) {
        let u = &self.units[a];
        let raw = hit.power as i64 * u.stats.atk as i64 / 100;
        let mut dmg = raw * 100 / (100 + self.units[t].stats.def as i64);
        dmg = dmg * self.rng.range(95, 105) as i64 / 100;
        let dmg = dmg.max(1) as u32;
        self.lose_hp(t, dmg);
        self.log.push(LogEntry::Damage {
            target: t,
            amount: dmg,
            absorbed: 0,
            effective: 0,
        });
        if !self.units[t].alive() {
            self.ko(t);
        }
    }
}

//! Enemy intents and behaviour (game-systems §8.2).

use super::state::*;
use crate::content::{db::GameDb, defs::*};

impl BattleState {
    fn ai_rules<'a>(&self, db: &'a GameDb, i: usize) -> &'a [AiRule] {
        db.enemies
            .get(&self.units[i].def)
            .map_or(&[], |d| d.ai.as_slice())
    }

    fn cond_holds(&self, i: usize, cond: &AiCond) -> bool {
        let foes = self.opponents_of(i);
        match cond {
            AiCond::Always => true,
            AiCond::SelfHpBelow(pct) => self.units[i].hp_pct() < *pct,
            AiCond::FoeCharging(stage) => foes.iter().any(|&f| self.units[f].charge >= *stage),
            AiCond::FoeChanneling => foes.iter().any(|&f| self.units[f].channel.is_some()),
            AiCond::FormationPhaseAtLeast(n) => {
                self.formation.as_ref().is_some_and(|f| f.phase >= *n)
            }
            AiCond::EveryNth(n, offset) => self.units[i].own_activations % (*n).max(1) == *offset,
            AiCond::AllyHpBelow(pct) => self
                .allies_of(i)
                .iter()
                .any(|&a| self.units[a].hp_pct() < *pct),
            AiCond::FoeHasShield => foes.iter().any(|&f| self.units[f].shield() > 0),
            AiCond::AllyLacks(kind) => self.allies_of(i).iter().any(|&a| !self.units[a].has(*kind)),
        }
    }

    fn rule_usable(&self, db: &GameDb, i: usize, rule: &AiRule, cooldown_slack: u8) -> bool {
        let Some(skill) = db.skill(&rule.skill) else {
            return false;
        };
        let cooldown = self.units[i]
            .skills
            .iter()
            .find(|s| s.id == rule.skill)
            .map_or(0, |s| s.cooldown);
        cooldown <= cooldown_slack
            && self.units[i].ll >= skill.ll
            && !(skill.ll > 0 && self.units[i].has(StatusKind::PhongAn))
            && self.cond_holds(i, &rule.when)
    }

    /// Picks the intent shown until the enemy's next activation.
    pub(crate) fn choose_intent(&mut self, db: &GameDb, i: usize) {
        let rules = self.ai_rules(db, i);
        // Cooldowns tick at activation start, so a cooldown of 1 is ready next time.
        let rule = rules
            .iter()
            .find(|r| !r.reactive && self.rule_usable(db, i, r, 1))
            .cloned();
        self.units[i].intent = rule.map(|r| {
            let target = self.pick_target(db, i, &r);
            Intent {
                skill: r.skill.clone(),
                target: match target {
                    Some(Target::Unit(t)) => Some(t),
                    _ => None,
                },
            }
        });
    }

    fn lowest_hp(&self, candidates: &[usize]) -> Option<usize> {
        candidates
            .iter()
            .copied()
            .min_by_key(|&c| (self.units[c].hp_pct(), self.units[c].hp, c))
    }

    fn pick_target(&mut self, db: &GameDb, i: usize, rule: &AiRule) -> Option<Target> {
        let skill = db.skill(&rule.skill)?;
        match skill.target {
            TargetKind::AllEnemies => return Some(Target::Opponents),
            TargetKind::AllAllies => return Some(Target::Allies),
            TargetKind::SelfOnly => return Some(Target::Myself),
            TargetKind::Ally => {
                let mut allies = self.allies_of(i);
                // A rule about a missing status targets an ally that lacks it.
                if let AiCond::AllyLacks(kind) = rule.when {
                    allies.retain(|&a| !self.units[a].has(kind));
                }
                return self.lowest_hp(&allies).map(Target::Unit);
            }
            TargetKind::Enemy => {}
        }
        let valid = self.valid_targets(i, TargetKind::Enemy, skill.melee);
        if valid.is_empty() {
            return None;
        }
        let front = valid[0];
        let chosen = match rule.target {
            AiTarget::Front => front,
            AiTarget::LowestHp => self.lowest_hp(&valid).unwrap_or(front),
            AiTarget::Charging => valid
                .iter()
                .copied()
                .filter(|&v| self.units[v].charge > 0)
                .max_by_key(|&v| (self.units[v].charge, std::cmp::Reverse(v)))
                .unwrap_or(front),
            AiTarget::Channeling => valid
                .iter()
                .copied()
                .find(|&v| self.units[v].channel.is_some())
                .unwrap_or(front),
            AiTarget::FormationNode => valid
                .iter()
                .copied()
                .find(|&v| !self.units[v].node_broken)
                .unwrap_or(front),
            AiTarget::Random => valid[self.rng.range(0, valid.len() as u32 - 1) as usize],
            AiTarget::SelfUnit => i,
            AiTarget::AllFoes => return Some(Target::Opponents),
            AiTarget::LowestHpAlly => {
                let allies = self.allies_of(i);
                self.lowest_hp(&allies).unwrap_or(i)
            }
        };
        Some(Target::Unit(chosen))
    }

    /// Runs one enemy activation: a reactive rule may override the intent.
    pub(crate) fn enemy_act(&mut self, db: &GameDb, i: usize) {
        let rules = self.ai_rules(db, i);
        let reactive = rules
            .iter()
            .find(|r| r.reactive && self.rule_usable(db, i, r, 0))
            .cloned();
        let planned = self.units[i].intent.clone().and_then(|intent| {
            rules
                .iter()
                .find(|r| r.skill == intent.skill && !r.reactive && self.rule_usable(db, i, r, 0))
                .cloned()
        });
        let fallback = || {
            rules
                .iter()
                .find(|r| !r.reactive && self.rule_usable(db, i, r, 0))
                .cloned()
        };
        let Some(rule) = reactive.or(planned).or_else(fallback) else {
            return;
        };
        let Some(skill) = db.skill(&rule.skill).cloned() else {
            return;
        };
        let target = match self.units[i].intent.as_ref().and_then(|t| t.target) {
            // Keep the telegraphed target if it is still valid and the rule wasn't overridden.
            Some(t)
                if !rule.reactive
                    && self.units[t].alive()
                    && self
                        .valid_targets(i, skill.target, skill.melee)
                        .contains(&t)
                    && matches!(
                        rule.target,
                        AiTarget::Front
                            | AiTarget::LowestHp
                            | AiTarget::Random
                            | AiTarget::FormationNode
                    ) =>
            {
                Some(Target::Unit(t))
            }
            _ => self.pick_target(db, i, &rule),
        };
        let Some(target) = target else {
            return;
        };
        if let Some(slot) = self.units[i].skills.iter_mut().find(|s| s.id == rule.skill) {
            slot.cooldown = skill.cooldown;
        }
        self.use_skill(db, i, &skill, target);
    }
}

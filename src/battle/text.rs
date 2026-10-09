//! Turns battle log entries and unit data into Vietnamese text.

use super::core::{BattleState, LogEntry};
use crate::{
    content::{Content, defs::Side},
    story::Progress,
};

/// Display name; duplicates of the same enemy get “A”, “B”…
pub fn unit_name(state: &BattleState, i: usize, content: &Content, progress: &Progress) -> String {
    let u = &state.units[i];
    match u.side {
        Side::Party => content.character_name(&u.def, progress),
        Side::Enemy => {
            let base = content.text(&format!("enemy.{}.name", u.def), progress);
            let same: Vec<usize> = state
                .enemies()
                .filter(|&e| state.units[e].def == u.def)
                .collect();
            if same.len() > 1 {
                let letter = (b'A' + same.iter().position(|&e| e == i).unwrap_or(0) as u8) as char;
                format!("{base} {letter}")
            } else {
                base
            }
        }
    }
}

/// Compact name for tight spaces: the protagonist's given name (last word, as
/// Vietnamese names are written family name first), others unchanged.
pub fn short_name(state: &BattleState, i: usize, content: &Content, progress: &Progress) -> String {
    let name = unit_name(state, i, content, progress);
    if state.units[i].is_player() {
        name.split_whitespace().last().unwrap_or(&name).to_string()
    } else {
        name
    }
}

pub fn log_line(
    entry: &LogEntry,
    state: &BattleState,
    content: &Content,
    progress: &Progress,
) -> Option<String> {
    let name = |i: usize| unit_name(state, i, content, progress);
    let skill = |id: &str| content.text(&format!("skill.{id}.name"), progress);
    let status = |s: crate::content::defs::StatusKind| {
        content.text(&format!("status.{}.name", s.id()), progress)
    };
    let f = |key: &str, params: &[(&str, String)]| content.format(key, progress, params);
    Some(match entry {
        LogEntry::Turn { .. } => return None,
        LogEntry::Skill {
            unit,
            skill: id,
            target,
        } => match target {
            Some(t) if t != unit => f(
                "log.skill_target",
                &[
                    ("unit", name(*unit)),
                    ("skill", skill(id)),
                    ("target", name(*t)),
                ],
            ),
            _ => f("log.skill", &[("unit", name(*unit)), ("skill", skill(id))]),
        },
        LogEntry::Strike { unit, target } => f(
            "log.strike",
            &[("unit", name(*unit)), ("target", name(*target))],
        ),
        LogEntry::Damage {
            target,
            amount,
            absorbed,
            effective,
        } => {
            let mut line = if *absorbed > 0 {
                f(
                    "log.damage_absorbed",
                    &[
                        ("target", name(*target)),
                        ("damage", amount.to_string()),
                        ("absorbed", absorbed.to_string()),
                    ],
                )
            } else {
                f(
                    "log.damage",
                    &[("target", name(*target)), ("damage", amount.to_string())],
                )
            };
            match effective {
                1 => line.push_str(&content.text("log.effective", progress)),
                -1 => line.push_str(&content.text("log.resisted_element", progress)),
                _ => {}
            }
            line
        }
        LogEntry::Miss { target } => f("log.miss", &[("target", name(*target))]),
        LogEntry::Heal { target, amount } => f(
            "log.heal",
            &[("target", name(*target)), ("amount", amount.to_string())],
        ),
        LogEntry::Shield { target, amount } => f(
            "log.shield",
            &[("target", name(*target)), ("amount", amount.to_string())],
        ),
        LogEntry::Status { target, status: s } => f(
            "log.status",
            &[("target", name(*target)), ("status", status(*s))],
        ),
        LogEntry::StatusResisted { target, status: s } => f(
            "log.status_resisted",
            &[("target", name(*target)), ("status", status(*s))],
        ),
        LogEntry::Cleansed { target } => f("log.cleansed", &[("target", name(*target))]),
        LogEntry::Charge { unit, stage } => f(
            "log.charge",
            &[("unit", name(*unit)), ("stage", stage.to_string())],
        ),
        LogEntry::ChargeBroken { unit, lost } => f(
            "log.charge_broken",
            &[("unit", name(*unit)), ("lost", lost.to_string())],
        ),
        LogEntry::ChargeReleased { unit, stage } => f(
            "log.charge_released",
            &[("unit", name(*unit)), ("stage", stage.to_string())],
        ),
        LogEntry::Channel {
            unit, skill: id, ..
        } => f(
            "log.channel",
            &[("unit", name(*unit)), ("skill", skill(id))],
        ),
        LogEntry::ChannelInterrupted { unit } => {
            f("log.channel_interrupted", &[("unit", name(*unit))])
        }
        LogEntry::Pushed { target, ticks } => f(
            "log.pushed",
            &[("target", name(*target)), ("ticks", ticks.to_string())],
        ),
        LogEntry::PushResisted { target } => f("log.push_resisted", &[("target", name(*target))]),
        LogEntry::Pulled { target, ticks } => f(
            "log.pulled",
            &[("target", name(*target)), ("ticks", ticks.to_string())],
        ),
        LogEntry::Ko { unit } => f("log.ko", &[("unit", name(*unit))]),
        LogEntry::Combo { from, to } => f(
            "log.combo",
            &[
                ("from", content.text(from.key(), progress)),
                ("to", content.text(to.key(), progress)),
            ],
        ),
        LogEntry::FormationPhase { phase } => {
            f("log.formation_phase", &[("phase", phase.to_string())])
        }
        LogEntry::FormationEnergy { .. } => return None,
        LogEntry::NodeBroken { unit } => f("log.node_broken", &[("unit", name(*unit))]),
        LogEntry::NodeRestored { unit } => f("log.node_restored", &[("unit", name(*unit))]),
        LogEntry::FormationChaos => content.text("log.formation_chaos", progress),
        LogEntry::FormationReleased { phase } => {
            f("log.formation_released", &[("phase", phase.to_string())])
        }
        LogEntry::Guard { unit } => f("log.guard", &[("unit", name(*unit))]),
        LogEntry::Meditate { unit, amount } => f(
            "log.meditate",
            &[("unit", name(*unit)), ("amount", amount.to_string())],
        ),
        LogEntry::LlGain { unit, amount } => {
            if *amount == 0 {
                return None;
            }
            f(
                "log.ll_gain",
                &[("unit", name(*unit)), ("amount", amount.to_string())],
            )
        }
        LogEntry::Drain { target, amount } => f(
            "log.drain",
            &[("target", name(*target)), ("amount", amount.to_string())],
        ),
        LogEntry::Stunned { unit } => f("log.stunned", &[("unit", name(*unit))]),
        LogEntry::Dot {
            unit,
            status: s,
            amount,
        } => f(
            "log.dot",
            &[
                ("unit", name(*unit)),
                ("status", status(*s)),
                ("amount", amount.to_string()),
            ],
        ),
        LogEntry::Backlash { unit, amount } => f(
            "log.backlash",
            &[("unit", name(*unit)), ("amount", amount.to_string())],
        ),
        LogEntry::Overheat { unit, amount } => f(
            "log.overheat",
            &[("unit", name(*unit)), ("amount", amount.to_string())],
        ),
        LogEntry::Item { unit, item } => f(
            "log.item",
            &[
                ("unit", name(*unit)),
                ("item", content.text(&format!("item.{item}.name"), progress)),
            ],
        ),
        LogEntry::Artifact { unit, artifact } => f(
            "log.artifact",
            &[
                ("unit", name(*unit)),
                (
                    "artifact",
                    content.text(&format!("artifact.{artifact}.name"), progress),
                ),
            ],
        ),
        LogEntry::Swap { a, b } => f("log.swap", &[("a", name(*a)), ("b", name(*b))]),
        LogEntry::ElementSet { target, element } => f(
            "log.element_set",
            &[
                ("target", name(*target)),
                ("element", content.text(element.key(), progress)),
            ],
        ),
        LogEntry::Fled => content.text("log.fled", progress),
        LogEntry::FleeFailed => content.text("log.flee_failed", progress),
        LogEntry::Victory => content.text("log.victory", progress),
        LogEntry::Defeat => content.text("log.defeat", progress),
    })
}

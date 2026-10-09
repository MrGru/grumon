//! Party tab: members, battle rows, order, artifacts and the formation.

use bevy::prelude::*;

use super::{Detail, Message, PauseMenu, Sub};
use crate::{
    content::{Content, defs::Realm},
    flow::Story,
    hud::realm_text,
    story::{PLAYER_ID, Progress, stage_thresholds},
    ui,
};

/// Actions for a selected member, in menu order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MemberAction {
    Row,
    Up,
    Down,
    Artifacts,
}

pub(super) const MEMBER_ACTIONS: [MemberAction; 4] = [
    MemberAction::Row,
    MemberAction::Up,
    MemberAction::Down,
    MemberAction::Artifacts,
];

impl MemberAction {
    fn key(self) -> &'static str {
        match self {
            MemberAction::Row => "ui.party.action.row",
            MemberAction::Up => "ui.party.action.up",
            MemberAction::Down => "ui.party.action.down",
            MemberAction::Artifacts => "ui.party.action.artifacts",
        }
    }
}

/// A row in the artifact list: equipped ones first, then the bag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ArtifactRow {
    Equipped(String),
    Bag(String),
}

pub(super) fn artifact_rows(
    content: &Content,
    progress: &Progress,
    member: usize,
) -> Vec<ArtifactRow> {
    let mut rows: Vec<ArtifactRow> = progress
        .party
        .get(member)
        .map(|m| {
            m.artifacts
                .iter()
                .cloned()
                .map(ArtifactRow::Equipped)
                .collect()
        })
        .unwrap_or_default();
    rows.extend(
        progress
            .bag_artifacts(&content.db)
            .into_iter()
            .map(ArtifactRow::Bag),
    );
    rows
}

/// "None" plus every known formation.
pub(super) fn formation_rows(progress: &Progress) -> Vec<Option<String>> {
    std::iter::once(None)
        .chain(progress.formations_known.iter().cloned().map(Some))
        .collect()
}

/// Rows in the tab: one per member, plus the formation row once one is known.
pub(super) fn len(progress: &Progress) -> usize {
    progress.party.len() + usize::from(!progress.formations_known.is_empty())
}

pub(super) fn activate(menu: &mut PauseMenu, progress: &Progress, row: usize) {
    menu.sub = Some(if row < progress.party.len() {
        Sub::Member {
            member: row,
            cursor: 0,
        }
    } else {
        let current = formation_rows(progress)
            .iter()
            .position(|f| *f == progress.formation)
            .unwrap_or(0);
        Sub::Formation { cursor: current }
    });
}

pub(super) fn confirm(menu: &mut PauseMenu, story: &mut Story) {
    let Some(sub) = menu.sub.clone() else {
        return;
    };
    menu.dirty = true;
    match sub {
        Sub::Member { member, cursor } => match MEMBER_ACTIONS[cursor] {
            MemberAction::Row => match story.progress.cycle_slot(member) {
                Ok(_) => menu.message = None,
                Err(e) => menu.message = Some(Message::error(e.key(), vec![])),
            },
            MemberAction::Up | MemberAction::Down => {
                let up = MEMBER_ACTIONS[cursor] == MemberAction::Up;
                if let Some(to) = story.progress.move_member(member, up) {
                    menu.inner = Some(to);
                    menu.sub = Some(Sub::Member { member: to, cursor });
                }
            }
            MemberAction::Artifacts => {
                menu.sub = Some(Sub::Equip { member, cursor: 0 });
            }
        },
        Sub::Equip { member, cursor } => {
            let rows = artifact_rows(&story.content, &story.progress, member);
            let result = match rows.get(cursor) {
                Some(ArtifactRow::Equipped(id)) => story.progress.unequip(member, id),
                Some(ArtifactRow::Bag(id)) => story.progress.equip(&story.content.db, member, id),
                None => return,
            };
            menu.message = result.err().map(|e| Message::error(e.key(), vec![]));
            let n = artifact_rows(&story.content, &story.progress, member).len();
            menu.sub = Some(Sub::Equip {
                member,
                cursor: cursor.min(n.saturating_sub(1)),
            });
        }
        Sub::Formation { cursor } => {
            if let Some(choice) = formation_rows(&story.progress).get(cursor) {
                story.progress.formation = choice.clone();
                menu.message = None;
            }
            menu.sub = None;
        }
        Sub::UseOn { .. } => {}
    }
}

fn member_details(d: &mut Detail, content: &Content, progress: &Progress, index: usize) {
    let member = &progress.party[index];
    let Some(def) = content.db.characters.get(&member.id) else {
        return;
    };
    let stats = member.stats(def);
    d.line(
        content.format(
            "ui.party.stats",
            progress,
            &[
                ("hp", stats.hp.to_string()),
                ("ll", stats.ll.to_string()),
                ("atk", stats.atk.to_string()),
                ("spi", stats.spi.to_string()),
                ("def", stats.def.to_string()),
                ("tp", stats.tp.to_string()),
            ],
        ),
        ui::TEXT,
        16.0,
    );
    let skills: Vec<String> = member
        .skills
        .iter()
        .map(|s| content.text(&format!("skill.{s}.name"), progress))
        .collect();
    d.line(
        content.format("ui.party.skills", progress, &[("list", skills.join(", "))]),
        ui::TEXT_DIM,
        16.0,
    );
    let artifacts: Vec<String> = member
        .artifacts
        .iter()
        .map(|a| content.text(&format!("artifact.{a}.name"), progress))
        .collect();
    let list = if artifacts.is_empty() {
        content.text("ui.common.none", progress)
    } else {
        artifacts.join(", ")
    };
    d.line(
        content.format(
            "ui.party.artifacts",
            progress,
            &[
                ("list", list),
                ("slots", member.realm.artifact_slots().to_string()),
            ],
        ),
        ui::TEXT_DIM,
        16.0,
    );
    if member.realm != Realm::PhamNhan {
        let next = stage_thresholds(member.realm)
            .get(member.stage as usize)
            .copied();
        let text = match next {
            Some(n) => content.format(
                "ui.party.tu_vi",
                progress,
                &[("now", member.tu_vi.to_string()), ("next", n.to_string())],
            ),
            None => content.text("ui.party.tu_vi_peak", progress),
        };
        d.line(text, ui::JADE, 16.0);
    }
}

fn pointer(selected: bool) -> &'static str {
    if selected { "›" } else { " " }
}

pub(super) fn draw(d: &mut Detail, content: &Content, progress: &Progress, menu: &PauseMenu) {
    let cursor = menu.inner.unwrap_or(usize::MAX);
    let focus = match &menu.sub {
        Some(Sub::Member { member, .. } | Sub::Equip { member, .. }) => Some(*member),
        _ if cursor < progress.party.len() => Some(cursor),
        _ if progress.party.len() == 1 && menu.inner.is_none() => Some(0),
        _ => None,
    };
    for (i, member) in progress.party.iter().enumerate() {
        let Some(def) = content.db.characters.get(&member.id) else {
            continue;
        };
        let selected = Some(i) == focus;
        let mut text = format!(
            "{} {} — {} · {} · {}",
            pointer(i == cursor),
            content.character_name(&member.id, progress),
            realm_text(content, progress, member.realm, member.stage),
            content.text(def.element.key(), progress),
            content.text(member.slot.key(), progress),
        );
        if member.id == PLAYER_ID && progress.party.len() > 1 {
            text.push_str(&format!(" · {}", content.text("ui.party.leader", progress)));
        }
        d.line(text, if selected { ui::GOLD } else { ui::TEXT }, 19.0);
        if !selected {
            continue;
        }
        member_details(d, content, progress, i);
        match &menu.sub {
            Some(Sub::Member { cursor, .. }) => {
                for (a, action) in MEMBER_ACTIONS.iter().enumerate() {
                    let chosen = a == *cursor;
                    d.line(
                        format!(
                            "   {} {}",
                            pointer(chosen),
                            content.text(action.key(), progress)
                        ),
                        if chosen { ui::GOLD } else { ui::TEXT },
                        16.0,
                    );
                }
            }
            Some(Sub::Equip { cursor, .. }) => {
                let rows = artifact_rows(content, progress, i);
                if rows.is_empty() {
                    d.line(
                        content.text("ui.party.no_artifacts", progress),
                        ui::TEXT_DIM,
                        16.0,
                    );
                }
                for (r, row) in rows.iter().enumerate() {
                    let chosen = r == *cursor;
                    let (id, key) = match row {
                        ArtifactRow::Equipped(id) => (id, "ui.party.equipped"),
                        ArtifactRow::Bag(id) => (id, "ui.party.in_bag"),
                    };
                    let name = content.text(&format!("artifact.{id}.name"), progress);
                    let mut text = format!(
                        "   {} {}",
                        pointer(chosen),
                        content.format(key, progress, &[("name", name)])
                    );
                    let blocked = match row {
                        ArtifactRow::Bag(id) => progress.can_equip(&content.db, i, id).err(),
                        ArtifactRow::Equipped(_) => None,
                    };
                    if let Some(e) = blocked {
                        text.push_str(&format!(" ({})", content.text(e.key(), progress)));
                    }
                    let color = match (chosen, blocked.is_none()) {
                        (true, _) => ui::GOLD,
                        (false, true) => ui::TEXT,
                        (false, false) => ui::TEXT_DIM,
                    };
                    d.line(text, color, 16.0);
                    if chosen {
                        d.line(
                            format!(
                                "      {}",
                                content.text(&format!("artifact.{id}.desc"), progress)
                            ),
                            ui::TEXT_DIM,
                            14.0,
                        );
                    }
                }
            }
            _ => {}
        }
    }
    if !progress.formations_known.is_empty() {
        let row = progress.party.len();
        let current = match &progress.formation {
            Some(id) => content.text(&format!("formation.{id}.name"), progress),
            None => content.text("ui.common.none", progress),
        };
        d.line(
            format!(
                "{} {}",
                pointer(cursor == row),
                content.format("ui.party.formation", progress, &[("name", current)])
            ),
            if cursor == row { ui::GOLD } else { ui::TEXT },
            19.0,
        );
        if let Some(Sub::Formation { cursor }) = &menu.sub {
            for (f, choice) in formation_rows(progress).iter().enumerate() {
                let chosen = f == *cursor;
                let (name, ready) = match choice {
                    Some(id) => (
                        content.text(&format!("formation.{id}.name"), progress),
                        progress.formation_ready(&content.db, id),
                    ),
                    None => (content.text("ui.common.none", progress), true),
                };
                let mut text = format!("   {} {name}", pointer(chosen));
                if !ready {
                    text.push_str(&format!(
                        " ({})",
                        content.text("ui.party.formation_unmet", progress)
                    ));
                }
                d.line(
                    text,
                    if chosen {
                        ui::GOLD
                    } else if ready {
                        ui::TEXT
                    } else {
                        ui::TEXT_DIM
                    },
                    16.0,
                );
                if chosen && let Some(id) = choice {
                    d.line(
                        format!(
                            "      {}",
                            content.text(&format!("formation.{id}.desc"), progress)
                        ),
                        ui::TEXT_DIM,
                        14.0,
                    );
                }
            }
        }
    }
    d.line(
        content.format(
            "ui.party.money",
            progress,
            &[("count", progress.money.to_string())],
        ),
        ui::TEXT,
        16.0,
    );
    d.line(content.text("ui.party.hint", progress), ui::TEXT_DIM, 14.0);
}

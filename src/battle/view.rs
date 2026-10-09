//! Battle screen layout. Rebuilt from the session whenever it changes.
//! Always visible: timeline, Khí huyết / Linh lực, Tụ khí, statuses, enemy
//! intents, ĐHĐ, formation phase, artifact cooldowns, why an action is unavailable.

use bevy::prelude::*;

use super::{
    BattleSession, EndChoice, MenuMode, PendingCommand, RootEntry,
    core::{BattleState, CHARGE_MULT, Command, Phase, TICKS_PER_CYCLE, Target, Unit},
    menu_len,
    text::{short_name, unit_name},
};
use crate::{
    PlayState,
    asset::GameAssets,
    content::{
        Content,
        defs::{Objective, Side, SkillDef, Slot, TargetKind},
    },
    story::Progress,
    ui::{self, FontKind},
};

pub const CARD_W: f32 = 190.0;
const CARD_H: f32 = 104.0;
const CARDS_TOP: f32 = 96.0;
const PANELS_TOP: f32 = 430.0;
const TIMELINE_ENTRIES: usize = 9;
const POPUP_SECS: f32 = 0.9;

#[derive(Component)]
pub struct BattleRoot;

#[derive(Component)]
pub struct BattleDynamic;

#[derive(Component)]
pub struct PopupLayer;

#[derive(Component)]
pub struct PopupText {
    age: f32,
}

/// Horizontal space each side may use for its cards (screen is 960 wide).
const SIDE_LEFT: f32 = 16.0;
const SIDE_RIGHT: f32 = 470.0;
const PARTY_LEFT: f32 = 490.0;
const COLUMN_GAP: f32 = 12.0;
/// Vertical gap between cards in one row, leaving room for the intent line.
const STACK_GAP: f32 = 34.0;

/// Top-left corner of a unit's card.
///
/// Only occupied rows get a column, front rows nearest the centre. Columns
/// overlap (with a vertical stagger) only when a side fills all three rows.
pub fn card_pos(state: &BattleState, i: usize) -> Vec2 {
    let u = &state.units[i];
    let rows: Vec<Slot> = [Slot::Front, Slot::Middle, Slot::Back]
        .into_iter()
        .filter(|&slot| {
            state
                .units
                .iter()
                .any(|v| v.side == u.side && v.slot == slot)
        })
        .collect();
    let column = rows.iter().position(|&s| s == u.slot).unwrap_or(0);
    let room = SIDE_RIGHT - SIDE_LEFT - CARD_W;
    let step = if rows.len() > 1 {
        (CARD_W + COLUMN_GAP).min(room / (rows.len() - 1) as f32)
    } else {
        0.0
    };
    let x = match u.side {
        Side::Enemy => SIDE_RIGHT - CARD_W - column as f32 * step,
        Side::Party => PARTY_LEFT + column as f32 * step,
    };
    let stack = (0..i)
        .filter(|&j| state.units[j].side == u.side && state.units[j].slot == u.slot)
        .count();
    let crowded = step < CARD_W + COLUMN_GAP;
    let stagger = if crowded && column % 2 == 1 {
        40.0
    } else {
        0.0
    };
    Vec2::new(x, CARDS_TOP + stack as f32 * (CARD_H + STACK_GAP) + stagger)
}

pub fn spawn_view(
    mut commands: Commands,
    assets: Res<GameAssets>,
    session: Option<Res<BattleSession>>,
    content: Res<Content>,
) {
    let Some(session) = session else {
        return;
    };
    let background = content
        .db
        .encounters
        .get(&session.state.encounter)
        .and_then(|e| assets.battle_background(&e.background));
    let mut root = commands.spawn((
        Name::new("Battle"),
        BattleRoot,
        DespawnOnExit(PlayState::Battle),
        ui::fullscreen(),
        BackgroundColor(ui::INK),
        GlobalZIndex(40),
    ));
    if let Some(image) = background {
        root.with_child((
            ImageNode::new(image),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ));
    }
    root.with_child((BattleDynamic, ui::fullscreen()));
    root.with_child((PopupLayer, ui::fullscreen(), Pickable::IGNORE));
}

fn bar(fraction: f32, color: Color, width: f32) -> impl Bundle {
    (
        Node {
            width: Val::Px(width),
            height: Val::Px(7.0),
            flex_shrink: 0.0,
            border_radius: BorderRadius::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        children![(
            Node {
                width: Val::Percent(fraction.clamp(0.0, 1.0) * 100.0),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(color),
        )],
    )
}

fn pips(filled: u8, total: u8) -> String {
    (0..total)
        .map(|i| if i < filled { '•' } else { '·' })
        .collect()
}

struct Ctx<'a> {
    assets: &'a GameAssets,
    content: &'a Content,
    progress: &'a Progress,
}

impl Ctx<'_> {
    fn t(&self, key: &str) -> String {
        self.content.text(key, self.progress)
    }

    fn f(&self, key: &str, params: &[(&str, String)]) -> String {
        self.content.format(key, self.progress, params)
    }

    fn text(&self, s: String, kind: FontKind, size: f32, color: Color) -> impl Bundle + use<> {
        (
            Text::new(s),
            ui::font(self.assets, kind, size),
            TextColor(color),
        )
    }
}

/// Target the cursor is currently on (if choosing a target).
fn hovered_target(session: &BattleSession) -> Option<usize> {
    match &session.mode {
        MenuMode::Target { candidates, .. } => candidates.get(session.cursor).copied(),
        _ => None,
    }
}

fn unit_card(p: &mut ChildSpawnerCommands, session: &BattleSession, i: usize, ctx: &Ctx) {
    let state = &session.state;
    let u = &state.units[i];
    let active = state.phase == Phase::Command(i);
    let candidate =
        matches!(&session.mode, MenuMode::Target { candidates, .. } if candidates.contains(&i));
    let hovered = hovered_target(session) == Some(i);
    let border = if hovered {
        Color::WHITE
    } else if active {
        ui::GOLD
    } else if candidate {
        ui::JADE
    } else {
        ui::BORDER_DIM
    };
    let alpha = if u.alive() { 1.0 } else { 0.35 };

    let sprite = match &u.sprite {
        Some(path) => ImageNode::new(ctx.assets.enemy_sprite(path).unwrap_or_default()),
        None => ImageNode::from_atlas_image(
            ctx.assets.character_image(u.sheet.max(1)),
            TextureAtlas {
                layout: ctx.assets.character_layout.clone(),
                // Party faces left (row 1), enemies face right (row 2).
                index: if u.side == Side::Party { 4 } else { 8 },
            },
        ),
    };
    let tint = u
        .tint
        .map_or(Color::WHITE, |(r, g, b)| Color::srgb(r, g, b));
    let sprite = sprite.with_color(tint.with_alpha(alpha));

    let name = unit_name(state, i, ctx.content, ctx.progress);
    let element = ctx.t(u.current_element().key());
    let mut statuses: Vec<String> = u
        .statuses
        .iter()
        .map(|s| {
            let n = ctx.t(&format!("status.{}.name", s.kind.id()));
            match s.kind {
                crate::content::defs::StatusKind::Khien => format!("{n} {}", s.value),
                _ if s.turns > 0 => format!("{n} {}", s.turns),
                _ => n,
            }
        })
        .collect();
    if u.node_broken {
        statuses.push(ctx.t("ui.battle.node_broken"));
    }
    let charge = if u.charge > 0 {
        ctx.f(
            "ui.battle.charge",
            &[
                ("pips", pips(u.charge, u.max_charge)),
                ("mult", CHARGE_MULT[u.charge as usize].to_string()),
            ],
        )
    } else {
        String::new()
    };

    // Enemy intent under the card.
    let mut lines = Vec::new();
    if u.side == Side::Enemy && u.alive() {
        if let Some(channel) = &u.channel {
            lines.push((
                ctx.f(
                    "ui.battle.channeling",
                    &[
                        ("skill", ctx.t(&format!("skill.{}.name", channel.skill))),
                        (
                            "ticks",
                            channel.resolve_at.saturating_sub(state.clock).to_string(),
                        ),
                    ],
                ),
                ui::DANGER,
            ));
        } else if let Some(intent) = &u.intent {
            let skill = ctx.content.db.skill(&intent.skill);
            let target = match (skill.map(|s| s.target), intent.target) {
                (Some(TargetKind::AllEnemies), _) => ctx.t("ui.battle.target_all"),
                (_, Some(t)) => unit_name(state, t, ctx.content, ctx.progress),
                _ => String::new(),
            };
            let mut text = ctx.f(
                "ui.battle.intent",
                &[
                    ("skill", ctx.t(&format!("skill.{}.name", intent.skill))),
                    ("target", target),
                ],
            );
            if let Some(w) = skill.and_then(|s| s.windup) {
                text.push_str(&ctx.f("ui.battle.intent_windup", &[("ticks", w.to_string())]));
            }
            lines.push((text, Color::srgb(1.0, 0.75, 0.55)));
        }
        if let Some(def) = ctx.content.db.enemies.get(&u.def) {
            for rule in def.ai.iter().filter(|r| r.reactive) {
                let cond = match &rule.when {
                    crate::content::defs::AiCond::FoeCharging(stage) => {
                        ctx.f("intent.cond.foe_charging", &[("stage", stage.to_string())])
                    }
                    crate::content::defs::AiCond::FoeChanneling => {
                        ctx.t("intent.cond.foe_channeling")
                    }
                    crate::content::defs::AiCond::FoeHasShield => ctx.t("intent.cond.foe_shield"),
                    _ => ctx.t("intent.cond.other"),
                };
                lines.push((
                    ctx.f(
                        "ui.battle.reactive",
                        &[
                            ("cond", cond),
                            ("skill", ctx.t(&format!("skill.{}.name", rule.skill))),
                        ],
                    ),
                    ui::TEXT_DIM,
                ));
            }
        }
    }

    // Card and intent share a wrapper so a taller card pushes the intent down.
    p.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(1.0),
        ..default()
    })
    .with_children(|w| {
        w.spawn((
            BackgroundColor(Color::srgba(0.03, 0.05, 0.06, 0.82 * alpha)),
            BorderColor::all(border),
            Node {
                width: Val::Px(CARD_W),
                min_height: Val::Px(CARD_H),
                border: UiRect::all(Val::Px(if active || hovered { 2.0 } else { 1.0 })),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                padding: UiRect::all(Val::Px(4.0)),
                column_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|card| {
            card.spawn((
                sprite,
                Node {
                    width: Val::Px(56.0),
                    height: Val::Px(56.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));
            card.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                // 190 card = 2×2 border + 2×4 padding + 56 sprite + 4 gap + 118.
                width: Val::Px(118.0),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|col| {
                col.spawn(ctx.text(
                    name,
                    FontKind::Bold,
                    14.0,
                    if active { ui::GOLD } else { ui::TEXT }.with_alpha(alpha),
                ));
                col.spawn(ctx.text(
                    format!("{} · {}", element, ctx.t(u.slot.key())),
                    FontKind::Body,
                    11.0,
                    ui::TEXT_DIM,
                ));
                col.spawn(bar(
                    u.hp as f32 / u.stats.hp.max(1) as f32,
                    ui::HP_COLOR,
                    114.0,
                ));
                col.spawn(ctx.text(
                    ctx.f(
                        "ui.battle.hp",
                        &[("now", u.hp.to_string()), ("max", u.stats.hp.to_string())],
                    ),
                    FontKind::Body,
                    11.0,
                    ui::TEXT,
                ));
                if u.stats.ll > 0 {
                    col.spawn(bar(
                        u.ll as f32 / u.stats.ll.max(1) as f32,
                        ui::LL_COLOR,
                        114.0,
                    ));
                    col.spawn(ctx.text(
                        ctx.f(
                            "ui.battle.ll",
                            &[("now", u.ll.to_string()), ("max", u.stats.ll.to_string())],
                        ),
                        FontKind::Body,
                        11.0,
                        ui::TEXT,
                    ));
                }
                if !charge.is_empty() {
                    col.spawn(ctx.text(charge, FontKind::Bold, 11.0, ui::GOLD));
                }
                if !statuses.is_empty() {
                    col.spawn(ctx.text(statuses.join(", "), FontKind::Body, 10.5, ui::JADE));
                }
            });
        });
        if !lines.is_empty() {
            w.spawn((
                Node {
                    max_width: Val::Px(CARD_W),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.82)),
                GlobalZIndex(41),
            ))
            .with_children(|col| {
                for (line, color) in lines {
                    col.spawn(ctx.text(line, FontKind::Body, 12.0, color));
                }
            });
        }
    });
}

/// One absolutely placed column per occupied row; cards stack inside it.
fn unit_columns(p: &mut ChildSpawnerCommands, session: &BattleSession, ctx: &Ctx) {
    let state = &session.state;
    for side in [Side::Enemy, Side::Party] {
        for slot in [Slot::Front, Slot::Middle, Slot::Back] {
            let members: Vec<usize> = (0..state.units.len())
                .filter(|&i| state.units[i].side == side && state.units[i].slot == slot)
                .collect();
            let Some(&first) = members.first() else {
                continue;
            };
            let pos = card_pos(state, first);
            p.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.x),
                top: Val::Px(pos.y),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|col| {
                for i in members {
                    unit_card(col, session, i, ctx);
                }
            });
        }
    }
}

fn timeline(p: &mut ChildSpawnerCommands, state: &BattleState, ctx: &Ctx) {
    let preview = state.preview(TIMELINE_ENTRIES);
    p.spawn((
        BackgroundColor(Color::srgba(0.03, 0.05, 0.06, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(8.0),
            width: Val::Px(936.0),
            height: Val::Px(44.0),
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(8.0)),
            column_gap: Val::Px(6.0),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
    ))
    .with_children(|row| {
        row.spawn(ctx.text(ctx.t("ui.battle.timeline"), FontKind::Bold, 13.0, ui::GOLD));
        for (k, entry) in preview.iter().enumerate() {
            let u = &state.units[entry.unit];
            let color = match (entry.channel, u.side) {
                (true, _) => ui::DANGER,
                (false, Side::Party) => ui::JADE,
                (false, Side::Enemy) => Color::srgb(0.95, 0.6, 0.45),
            };
            let mut label = short_name(state, entry.unit, ctx.content, ctx.progress);
            if entry.channel {
                label = ctx.f("ui.battle.timeline_channel", &[("name", label)]);
            }
            let delta = entry.tick.saturating_sub(state.clock);
            row.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                    border: UiRect::all(Val::Px(if k == 0 { 2.0 } else { 1.0 })),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(color),
                children![
                    ctx.text(label, FontKind::Bold, 11.5, color),
                    ctx.text(format!("+{delta}"), FontKind::Body, 10.0, ui::TEXT_DIM),
                ],
            ));
        }
    });
}

fn objective_and_formation(p: &mut ChildSpawnerCommands, state: &BattleState, ctx: &Ctx) {
    let mut objective = ctx.t(&format!("encounter.{}.objective", state.encounter));
    if let Objective::Survive(cycles) = state.objective {
        let left = (cycles * TICKS_PER_CYCLE).saturating_sub(state.clock);
        objective.push_str(&ctx.f("ui.battle.survive_left", &[("ticks", left.to_string())]));
    }
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(58.0),
            max_width: Val::Px(600.0),
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.85)),
        children![ctx.text(
            ctx.f(
                "ui.battle.objective",
                &[
                    ("text", objective),
                    ("cycle", (state.cycle() + 1).to_string())
                ]
            ),
            FontKind::Bold,
            14.0,
            ui::GOLD,
        )],
    ));
    let Some(f) = &state.formation else {
        return;
    };
    let Some(def) = ctx.content.db.formations.get(&f.id) else {
        return;
    };
    let cap = def.phases.last().map_or(0, |p| p.threshold);
    let next = def
        .phases
        .get(f.phase as usize)
        .map_or(cap, |p| p.threshold);
    let mut text = ctx.f(
        "ui.battle.formation",
        &[
            ("name", ctx.t(&format!("formation.{}.name", f.id))),
            ("phase", f.phase.to_string()),
            ("max", def.phases.len().to_string()),
            ("energy", f.energy.to_string()),
            ("next", next.to_string()),
        ],
    );
    if state.formation_chaos() {
        text.push_str(&ctx.t("ui.battle.formation_chaos"));
    }
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(12.0),
            top: Val::Px(58.0),
            max_width: Val::Px(330.0),
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.05, 0.06, 0.85)),
        children![ctx.text(text, FontKind::Bold, 13.0, ui::JADE)],
    ));
}

/// Label, cost and availability of each row of the current menu.
fn menu_rows(session: &BattleSession, actor: usize, ctx: &Ctx) -> Vec<(String, bool)> {
    let state = &session.state;
    let db = &ctx.content.db;
    let u = &state.units[actor];
    let ok = |c: Command| state.can(db, &c).is_ok();
    let any_target = |kind: TargetKind, melee: bool| state.default_target(actor, kind, melee);
    match &session.mode {
        MenuMode::Root => session
            .root_entries()
            .iter()
            .map(|e| {
                let (key, enabled) = match e {
                    RootEntry::Strike => (
                        "ui.battle.cmd.strike",
                        any_target(TargetKind::Enemy, true).is_some_and(
                            |t| matches!(t, Target::Unit(x) if ok(Command::Strike(x))),
                        ),
                    ),
                    RootEntry::Skills => ("ui.battle.cmd.skills", !u.skills.is_empty()),
                    RootEntry::Charge => ("ui.battle.cmd.charge", ok(Command::Charge)),
                    RootEntry::Artifacts => ("ui.battle.cmd.artifacts", !u.artifacts.is_empty()),
                    RootEntry::Items => {
                        ("ui.battle.cmd.items", state.items.values().any(|n| *n > 0))
                    }
                    RootEntry::Formation => ("ui.battle.cmd.formation", true),
                    RootEntry::Guard => ("ui.battle.cmd.guard", ok(Command::Guard)),
                    RootEntry::Meditate => ("ui.battle.cmd.meditate", ok(Command::Meditate)),
                    RootEntry::Swap => ("ui.battle.cmd.swap", true),
                    RootEntry::Flee => ("ui.battle.cmd.flee", ok(Command::Flee)),
                    RootEntry::EndTurn => ("ui.battle.cmd.end_turn", true),
                };
                (ctx.t(key), enabled)
            })
            .collect(),
        MenuMode::Skills => u
            .skills
            .iter()
            .map(|s| {
                let def = db.skill(&s.id);
                let mut label = ctx.t(&format!("skill.{}.name", s.id));
                if let Some(def) = def {
                    label.push_str(&ctx.f(
                        "ui.battle.cost",
                        &[("ap", def.ap.to_string()), ("ll", def.ll.to_string())],
                    ));
                }
                if s.cooldown > 0 {
                    label.push_str(
                        &ctx.f("ui.battle.cooldown_short", &[("n", s.cooldown.to_string())]),
                    );
                }
                let enabled = def
                    .and_then(|d| state.default_target(actor, d.target, d.melee))
                    .is_some_and(|t| ok(Command::Skill(s.id.clone(), t)));
                (label, enabled)
            })
            .collect(),
        MenuMode::Artifacts => u
            .artifacts
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let mut label = ctx.t(&format!("artifact.{}.name", a.id));
                if a.cooldown > 0 {
                    label.push_str(
                        &ctx.f("ui.battle.cooldown_short", &[("n", a.cooldown.to_string())]),
                    );
                }
                if let Some(c) = a.charges {
                    label.push_str(&ctx.f("ui.battle.charges_left", &[("n", c.to_string())]));
                }
                let skill = db
                    .artifacts
                    .get(&a.id)
                    .and_then(|d| d.active.as_ref())
                    .and_then(|s| db.skill(s));
                let enabled = skill
                    .and_then(|d| state.default_target(actor, d.target, d.melee))
                    .is_some_and(|t| ok(Command::Artifact(i, t)));
                (label, enabled)
            })
            .collect(),
        MenuMode::Items => state
            .items
            .iter()
            .filter(|(_, n)| **n > 0)
            .map(|(id, n)| {
                let item = db.items.get(id);
                let enabled = item
                    .and_then(|d| state.default_target(actor, d.target, false))
                    .is_some_and(|t| ok(Command::Item(id.clone(), t)));
                (
                    format!("{} ×{n}", ctx.t(&format!("item.{id}.name"))),
                    enabled,
                )
            })
            .collect(),
        MenuMode::Formation => vec![
            (
                ctx.t("ui.battle.cmd.release"),
                ok(Command::ReleaseFormation),
            ),
            (ctx.t("ui.battle.cmd.stabilize"), ok(Command::Stabilize)),
        ],
        MenuMode::Target { candidates, .. } => candidates
            .iter()
            .map(|&c| (unit_name(state, c, ctx.content, ctx.progress), true))
            .collect(),
    }
}

/// The command the cursor would issue (for availability and timing preview).
fn previewed_command(
    session: &BattleSession,
    actor: usize,
    ctx: &Ctx,
) -> Option<(Command, Option<&'static str>)> {
    let state = &session.state;
    let db = &ctx.content.db;
    let u = &state.units[actor];
    let target_of = |d: &SkillDef| state.default_target(actor, d.target, d.melee);
    Some(match &session.mode {
        MenuMode::Root => match session.root_entries().get(session.cursor)? {
            RootEntry::Strike => (
                Command::Strike(
                    match state.default_target(actor, TargetKind::Enemy, true)? {
                        Target::Unit(t) => t,
                        _ => return None,
                    },
                ),
                None,
            ),
            RootEntry::Charge => (Command::Charge, None),
            RootEntry::Guard => (Command::Guard, None),
            RootEntry::Meditate => (Command::Meditate, None),
            RootEntry::Flee => (Command::Flee, None),
            RootEntry::EndTurn => (Command::EndTurn, None),
            _ => return None,
        },
        MenuMode::Skills => {
            let s = u.skills.get(session.cursor)?;
            (
                Command::Skill(s.id.clone(), target_of(db.skill(&s.id)?)?),
                None,
            )
        }
        MenuMode::Artifacts => {
            let a = u.artifacts.get(session.cursor)?;
            let skill = db
                .artifacts
                .get(&a.id)?
                .active
                .as_ref()
                .and_then(|s| db.skill(s))?;
            (Command::Artifact(session.cursor, target_of(skill)?), None)
        }
        MenuMode::Items => {
            let (id, _) = state
                .items
                .iter()
                .filter(|(_, n)| **n > 0)
                .nth(session.cursor)?;
            let item = db.items.get(id)?;
            (
                Command::Item(id.clone(), state.default_target(actor, item.target, false)?),
                None,
            )
        }
        MenuMode::Formation => (
            if session.cursor == 0 {
                Command::ReleaseFormation
            } else {
                Command::Stabilize
            },
            None,
        ),
        MenuMode::Target {
            pending,
            candidates,
            ..
        } => {
            let t = *candidates.get(session.cursor)?;
            let cmd = match pending {
                PendingCommand::Strike => Command::Strike(t),
                PendingCommand::Skill(id) => Command::Skill(id.clone(), Target::Unit(t)),
                PendingCommand::Artifact(i) => Command::Artifact(*i, Target::Unit(t)),
                PendingCommand::Item(id) => Command::Item(id.clone(), Target::Unit(t)),
                PendingCommand::Swap => Command::Swap(t),
            };
            (cmd, None)
        }
    })
}

/// Name, cost, description and timing for the description panel.
fn describe(session: &BattleSession, actor: usize, ctx: &Ctx) -> Vec<(String, Color)> {
    let state = &session.state;
    let db = &ctx.content.db;
    let u = &state.units[actor];
    let mut lines: Vec<(String, Color)> = Vec::new();
    let skill_lines = |lines: &mut Vec<(String, Color)>, def: &SkillDef| {
        lines.push((ctx.t(&format!("skill.{}.name", def.id)), ui::GOLD));
        let mut cost = ctx.f(
            "ui.battle.cost_full",
            &[("ap", def.ap.to_string()), ("ll", def.ll.to_string())],
        );
        if def.cooldown > 0 {
            cost.push_str(&ctx.f("ui.battle.cooldown", &[("n", def.cooldown.to_string())]));
        }
        lines.push((cost, ui::TEXT));
        lines.push((ctx.t(&format!("skill.{}.desc", def.id)), ui::TEXT_DIM));
        if def.chargeable {
            lines.push((
                ctx.f(
                    "ui.battle.chargeable",
                    &[
                        ("stage", u.charge.to_string()),
                        ("mult", CHARGE_MULT[u.charge as usize].to_string()),
                    ],
                ),
                ui::JADE,
            ));
        }
        if let Some(w) = def.windup {
            lines.push((
                ctx.f("ui.battle.windup", &[("ticks", w.to_string())]),
                ui::JADE,
            ));
        }
        if def.delay > 0 {
            lines.push((
                ctx.f("ui.battle.delay", &[("ticks", def.delay.to_string())]),
                ui::JADE,
            ));
        }
    };
    let mut extra_delay = 0;
    match &session.mode {
        MenuMode::Root => {
            if let Some(entry) = session.root_entries().get(session.cursor) {
                let key = match entry {
                    RootEntry::Strike => "ui.battle.cmd.strike",
                    RootEntry::Skills => "ui.battle.cmd.skills",
                    RootEntry::Charge => "ui.battle.cmd.charge",
                    RootEntry::Artifacts => "ui.battle.cmd.artifacts",
                    RootEntry::Items => "ui.battle.cmd.items",
                    RootEntry::Formation => "ui.battle.cmd.formation",
                    RootEntry::Guard => "ui.battle.cmd.guard",
                    RootEntry::Meditate => "ui.battle.cmd.meditate",
                    RootEntry::Swap => "ui.battle.cmd.swap",
                    RootEntry::Flee => "ui.battle.cmd.flee",
                    RootEntry::EndTurn => "ui.battle.cmd.end_turn",
                };
                lines.push((ctx.t(key), ui::GOLD));
                lines.push((ctx.t(&format!("{key}_desc")), ui::TEXT_DIM));
            }
        }
        MenuMode::Skills => {
            if let Some(def) = u.skills.get(session.cursor).and_then(|s| db.skill(&s.id)) {
                extra_delay = def.delay;
                skill_lines(&mut lines, def);
            }
        }
        MenuMode::Artifacts => {
            if let Some(a) = u.artifacts.get(session.cursor) {
                lines.push((ctx.t(&format!("artifact.{}.name", a.id)), ui::GOLD));
                lines.push((ctx.t(&format!("artifact.{}.desc", a.id)), ui::TEXT_DIM));
                if let Some(def) = db
                    .artifacts
                    .get(&a.id)
                    .and_then(|d| d.active.as_ref())
                    .and_then(|s| db.skill(s))
                {
                    extra_delay = def.delay;
                    skill_lines(&mut lines, def);
                }
            }
        }
        MenuMode::Items => {
            if let Some((id, n)) = state
                .items
                .iter()
                .filter(|(_, n)| **n > 0)
                .nth(session.cursor)
            {
                lines.push((
                    format!("{} ×{n}", ctx.t(&format!("item.{id}.name"))),
                    ui::GOLD,
                ));
                lines.push((ctx.t("ui.battle.item_cost"), ui::TEXT));
                lines.push((ctx.t(&format!("item.{id}.desc")), ui::TEXT_DIM));
            }
        }
        MenuMode::Formation => {
            if let Some(f) = &state.formation {
                lines.push((ctx.t(&format!("formation.{}.name", f.id)), ui::GOLD));
                lines.push((ctx.t(&format!("formation.{}.desc", f.id)), ui::TEXT_DIM));
            }
            let key = if session.cursor == 0 {
                "ui.battle.cmd.release_desc"
            } else {
                "ui.battle.cmd.stabilize_desc"
            };
            lines.push((ctx.t(key), ui::TEXT));
        }
        MenuMode::Target { candidates, .. } => {
            if let Some(&t) = candidates.get(session.cursor) {
                let target: &Unit = &state.units[t];
                lines.push((unit_name(state, t, ctx.content, ctx.progress), ui::GOLD));
                lines.push((
                    ctx.f(
                        "ui.battle.target_info",
                        &[
                            ("hp", target.hp.to_string()),
                            ("max", target.stats.hp.to_string()),
                            ("element", ctx.t(target.current_element().key())),
                            ("tp", state.tp_eff(t).to_string()),
                        ],
                    ),
                    ui::TEXT,
                ));
                lines.push((ctx.t("ui.battle.target_hint"), ui::TEXT_DIM));
            }
        }
    }
    if let Some((command, _)) = previewed_command(session, actor, ctx) {
        if let Err(reason) = state.can(db, &command) {
            let text = match reason {
                super::core::Unavailable::Ap(n) => ctx.f(reason.key(), &[("n", n.to_string())]),
                super::core::Unavailable::Ll(n) => ctx.f(reason.key(), &[("n", n.to_string())]),
                super::core::Unavailable::Cooldown(n) => {
                    ctx.f(reason.key(), &[("n", n.to_string())])
                }
                _ => ctx.t(reason.key()),
            };
            lines.push((text, ui::DANGER));
        }
        let ends = matches!(
            command,
            Command::Guard | Command::Meditate | Command::EndTurn
        );
        if ends || extra_delay > 0 {
            let next = state
                .predicted_next(actor, extra_delay)
                .saturating_sub(state.clock);
            lines.push((
                ctx.f("ui.battle.next_turn", &[("ticks", next.to_string())]),
                ui::JADE,
            ));
        }
    }
    lines
}

fn bottom_panels(p: &mut ChildSpawnerCommands, session: &BattleSession, ctx: &Ctx) {
    let state = &session.state;
    let panel = |left: f32, width: f32| {
        (
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(PANELS_TOP),
                width: Val::Px(width),
                height: Val::Px(640.0 - PANELS_TOP - 10.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.035, 0.05, 0.06, 0.97)),
            BorderColor::all(ui::BORDER),
        )
    };

    // Command list.
    p.spawn(panel(12.0, 250.0)).with_children(|col| {
        match state.phase {
            Phase::Command(actor) => {
                let u = &state.units[actor];
                col.spawn(ctx.text(
                    ctx.f(
                        "ui.battle.turn_of",
                        &[
                            ("name", short_name(state, actor, ctx.content, ctx.progress)),
                            ("ap", u.ap.to_string()),
                        ],
                    ),
                    FontKind::Bold,
                    14.0,
                    ui::GOLD,
                ));
                let rows = menu_rows(session, actor, ctx);
                let n = menu_len(session, actor);
                // Scroll window of 8 rows around the cursor.
                let start = session.cursor.saturating_sub(7).min(n.saturating_sub(8));
                for (i, (label, enabled)) in rows.iter().enumerate().skip(start).take(8) {
                    let selected = i == session.cursor;
                    let color = match (enabled, selected) {
                        (false, true) => ui::DANGER,
                        (false, false) => ui::TEXT_DIM.with_alpha(0.6),
                        (true, true) => ui::GOLD,
                        (true, false) => ui::TEXT,
                    };
                    col.spawn(ctx.text(
                        format!("{} {label}", if selected { "›" } else { " " }),
                        if selected {
                            FontKind::Bold
                        } else {
                            FontKind::Body
                        },
                        14.0,
                        color,
                    ));
                }
            }
            _ => {
                col.spawn(ctx.text(
                    ctx.t("ui.battle.waiting"),
                    FontKind::Body,
                    14.0,
                    ui::TEXT_DIM,
                ));
            }
        }
    });

    // Description.
    p.spawn(panel(270.0, 330.0)).with_children(|col| {
        if let Phase::Command(actor) = state.phase {
            for (line, color) in describe(session, actor, ctx) {
                col.spawn(ctx.text(line, FontKind::Body, 13.5, color));
            }
        } else {
            col.spawn(ctx.text(
                ctx.t("ui.battle.controls"),
                FontKind::Body,
                13.0,
                ui::TEXT_DIM,
            ));
        }
    });

    // Log.
    p.spawn(panel(608.0, 340.0)).with_children(|col| {
        col.spawn(ctx.text(ctx.t("ui.battle.log"), FontKind::Bold, 13.0, ui::GOLD));
        for line in &session.lines {
            col.spawn(ctx.text(line.clone(), FontKind::Body, 12.5, ui::TEXT));
        }
    });
}

fn overlay_box(
    p: &mut ChildSpawnerCommands,
    assets: &GameAssets,
    lines: Vec<(String, Color, f32)>,
) {
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        GlobalZIndex(45),
    ))
    .with_children(|center| {
        center
            .spawn(ui::panel(Node {
                width: Val::Px(560.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            }))
            .with_children(|col| {
                for (i, (line, color, size)) in lines.into_iter().enumerate() {
                    let kind = if i == 0 {
                        FontKind::Title
                    } else {
                        FontKind::Body
                    };
                    col.spawn((
                        Text::new(line),
                        TextColor(color),
                        ui::font(assets, kind, size),
                    ));
                }
            });
    });
}

#[allow(clippy::too_many_arguments)]
pub fn redraw(
    mut commands: Commands,
    assets: Res<GameAssets>,
    content: Res<Content>,
    progress: Res<Progress>,
    mut session: ResMut<BattleSession>,
    root: Query<Entity, With<BattleDynamic>>,
) {
    if !session.dirty {
        return;
    }
    session.dirty = false;
    let Ok(root) = root.single() else {
        return;
    };
    let ctx = Ctx {
        assets: &assets,
        content: &content,
        progress: &progress,
    };
    commands.entity(root).despawn_children();
    let session = &*session;
    commands.entity(root).with_children(|p| {
        timeline(p, &session.state, &ctx);
        objective_and_formation(p, &session.state, &ctx);
        unit_columns(p, session, &ctx);
        bottom_panels(p, session, &ctx);

        if let Some(hint) = session.hints.first() {
            overlay_box(
                p,
                &assets,
                vec![
                    (ctx.t("ui.battle.hint_title"), ui::GOLD, 22.0),
                    (ctx.t(hint), ui::TEXT, 17.0),
                    (ctx.t("ui.battle.hint_continue"), ui::TEXT_DIM, 14.0),
                ],
            );
        } else if session.state.is_over() {
            let enc = content.db.encounters.get(&session.state.encounter);
            let defeat_continues = enc.is_some_and(|e| e.defeat_continues);
            let result = session.state.result(&content.db);
            let mut lines = Vec::new();
            match session.state.phase {
                Phase::Victory => {
                    lines.push((ctx.t("ui.battle.victory"), ui::GOLD, 30.0));
                    if result.tu_vi > 0
                        && progress
                            .player()
                            .is_some_and(|m| m.realm != crate::content::defs::Realm::PhamNhan)
                    {
                        lines.push((
                            ctx.f("ui.battle.reward_tu_vi", &[("n", result.tu_vi.to_string())]),
                            ui::TEXT,
                            17.0,
                        ));
                    }
                    for (item, n) in &result.drops {
                        lines.push((
                            ctx.f(
                                "ui.battle.reward_item",
                                &[
                                    ("item", ctx.t(&format!("item.{item}.name"))),
                                    ("n", n.to_string()),
                                ],
                            ),
                            ui::TEXT,
                            17.0,
                        ));
                    }
                }
                Phase::Defeat => {
                    lines.push((ctx.t("ui.battle.defeat"), ui::DANGER, 30.0));
                    lines.push((ctx.t("ui.battle.defeat_hint"), ui::TEXT_DIM, 16.0));
                }
                _ => lines.push((ctx.t("ui.battle.fled"), ui::TEXT, 26.0)),
            }
            for (i, choice) in session.end_choices(defeat_continues).iter().enumerate() {
                let key = match choice {
                    EndChoice::Continue => "ui.battle.end_continue",
                    EndChoice::Retry => "ui.battle.end_retry",
                    EndChoice::LoadLast => "ui.battle.end_load",
                    EndChoice::Title => "ui.battle.end_title",
                };
                let selected = i == session.end_cursor;
                lines.push((
                    format!("{} {}", if selected { "›" } else { " " }, ctx.t(key)),
                    if selected { ui::GOLD } else { ui::TEXT },
                    19.0,
                ));
            }
            overlay_box(p, &assets, lines);
        }
    });
}

/// Floating damage numbers.
pub fn animate_popups(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut session: ResMut<BattleSession>,
    layer: Query<Entity, With<PopupLayer>>,
    mut popups: Query<(Entity, &mut PopupText, &mut Node, &mut TextColor)>,
) {
    let Ok(layer) = layer.single() else {
        return;
    };
    let new: Vec<_> = session.popups.drain(..).collect();
    for (unit, text, kind, _) in new {
        let pos = card_pos(&session.state, unit);
        let color = match kind {
            0 => Color::srgb(1.0, 0.45, 0.4),
            1 => Color::srgb(0.5, 1.0, 0.6),
            _ => ui::TEXT,
        };
        commands.entity(layer).with_child((
            PopupText { age: 0.0 },
            Text::new(text),
            ui::font(&assets, FontKind::Bold, 22.0),
            TextColor(color),
            TextShadow::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.x + 18.0),
                top: Val::Px(pos.y + 10.0),
                ..default()
            },
            GlobalZIndex(44),
        ));
    }
    for (entity, mut popup, mut node, mut color) in &mut popups {
        popup.age += time.delta_secs();
        if let Val::Px(top) = node.top {
            node.top = Val::Px(top - 30.0 * time.delta_secs());
        }
        color.0 = color.0.with_alpha(1.0 - (popup.age / POPUP_SECS).min(1.0));
        if popup.age >= POPUP_SECS {
            commands.entity(entity).despawn();
        }
    }
}

//! Battle mode: wraps the pure [`core::BattleState`] in a Bevy session,
//! paces enemy turns, drives the command menu and applies results.

#[cfg(test)]
mod balance_tests;
pub mod core;
mod text;
mod view;

use std::collections::VecDeque;

use bevy::prelude::*;

use self::core::{BattleState, Command, Phase, Target, rng::Rng};
use crate::{
    GameState, PlayState,
    content::defs::{StoryEffect, TargetKind},
    flow::Story,
    input::MenuInput,
    pause_menu::Settings,
    save::{LoadRequest, SaveSlot},
};

/// Seconds between automated timeline steps (enemy turns, channel resolutions).
const PACE_NORMAL: f32 = 0.7;
const PACE_FAST: f32 = 0.3;
const LOG_LINES: usize = 7;

/// Asks the battle plugin to start or resume a battle.
#[derive(Resource, Debug, Clone)]
pub enum BattleRequest {
    Start(String),
    Resume(Box<BattleState>),
}

/// Menu entries of the root command list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootEntry {
    Strike,
    Skills,
    Charge,
    Artifacts,
    Items,
    Formation,
    Guard,
    Meditate,
    Swap,
    Flee,
    EndTurn,
}

/// What the command panel is showing.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuMode {
    Root,
    Skills,
    Artifacts,
    Items,
    Formation,
    /// Choosing a target for `pending`; `candidates` are unit indices.
    Target {
        pending: PendingCommand,
        candidates: Vec<usize>,
        back: Box<MenuMode>,
    },
}

/// A command waiting for its target.
#[derive(Debug, Clone, PartialEq)]
pub enum PendingCommand {
    Strike,
    Skill(String),
    Artifact(usize),
    Item(String),
    Swap,
}

impl PendingCommand {
    fn with_target(&self, t: usize) -> Command {
        match self {
            PendingCommand::Strike => Command::Strike(t),
            PendingCommand::Skill(id) => Command::Skill(id.clone(), Target::Unit(t)),
            PendingCommand::Artifact(i) => Command::Artifact(*i, Target::Unit(t)),
            PendingCommand::Item(id) => Command::Item(id.clone(), Target::Unit(t)),
            PendingCommand::Swap => Command::Swap(t),
        }
    }
}

/// End-of-battle choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndChoice {
    Continue,
    Retry,
    LoadLast,
    Title,
}

#[derive(Resource)]
pub struct BattleSession {
    pub state: BattleState,
    pub mode: MenuMode,
    pub cursor: usize,
    /// Cursor stack for nested menus.
    cursors: Vec<usize>,
    log_seen: usize,
    pub lines: VecDeque<String>,
    pace: Timer,
    /// Hint pages still to show before the first command.
    pub hints: Vec<String>,
    pub end_cursor: usize,
    pub dirty: bool,
    /// Damage/heal numbers to float over units: (unit, text, color index, age).
    pub popups: Vec<(usize, String, u8, f32)>,
    /// Initial state for “Thử lại”.
    initial: BattleState,
}

impl BattleSession {
    pub fn snapshot(&self) -> Option<BattleState> {
        (!self.state.is_over()).then(|| self.state.clone())
    }

    pub fn root_entries(&self) -> Vec<RootEntry> {
        // Mortals have not learned Tụ khí yet.
        let can_charge = match self.state.phase {
            Phase::Command(actor) => self.state.units[actor].max_charge > 0,
            _ => true,
        };
        let mut entries = vec![RootEntry::Strike, RootEntry::Skills];
        if can_charge {
            entries.push(RootEntry::Charge);
        }
        entries.extend([RootEntry::Artifacts, RootEntry::Items]);
        if self.state.formation.is_some() {
            entries.push(RootEntry::Formation);
        }
        entries.extend([RootEntry::Guard, RootEntry::Meditate]);
        if self
            .state
            .party()
            .filter(|&i| self.state.units[i].alive())
            .count()
            > 1
        {
            entries.push(RootEntry::Swap);
        }
        if self.state.can_flee {
            entries.push(RootEntry::Flee);
        }
        entries.push(RootEntry::EndTurn);
        entries
    }

    pub fn end_choices(&self, defeat_continues: bool) -> Vec<EndChoice> {
        match self.state.phase {
            Phase::Defeat if !defeat_continues => {
                vec![EndChoice::Retry, EndChoice::LoadLast, EndChoice::Title]
            }
            _ => vec![EndChoice::Continue],
        }
    }
}

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlayState::Battle),
            (start_battle, view::spawn_view).chain(),
        )
        .add_systems(
            Update,
            (
                advance_battle,
                battle_input,
                collect_log,
                view::redraw,
                view::animate_popups,
            )
                .chain()
                .run_if(in_state(PlayState::Battle).and_then(resource_exists::<BattleSession>)),
        )
        .add_systems(OnExit(PlayState::Battle), end_battle);
    }
}

fn start_battle(
    mut commands: Commands,
    request: Option<Res<BattleRequest>>,
    story: Story,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    let Some(request) = request.map(|r| r.clone()) else {
        next_state.set(PlayState::Exploring);
        return;
    };
    commands.remove_resource::<BattleRequest>();
    let state = match request {
        BattleRequest::Start(id) => {
            let seed = Rng::seed_from(&id, story.progress.battles_fought);
            match BattleState::from_progress(&story.content.db, &story.progress, &id, seed) {
                Ok(state) => state,
                Err(e) => {
                    error!("cannot start battle {id}: {e}");
                    next_state.set(PlayState::Exploring);
                    return;
                }
            }
        }
        BattleRequest::Resume(state) => *state,
    };
    let enc = story.content.db.encounters.get(&state.encounter);
    let seen_flag = format!("tut.{}", state.encounter);
    let hints = if story.progress.flag(&seen_flag) == 0 {
        enc.map(|e| e.hints.clone()).unwrap_or_default()
    } else {
        Vec::new()
    };
    let log_seen = state.log.len();
    commands.insert_resource(BattleSession {
        initial: state.clone(),
        state,
        mode: MenuMode::Root,
        cursor: 0,
        cursors: Vec::new(),
        log_seen,
        lines: VecDeque::new(),
        pace: Timer::from_seconds(0.4, TimerMode::Once),
        hints,
        end_cursor: 0,
        dirty: true,
        popups: Vec::new(),
    });
}

fn advance_battle(
    time: Res<Time>,
    settings: Res<Settings>,
    input: Res<MenuInput>,
    mut session: ResMut<BattleSession>,
    story: Story,
) {
    if !session.hints.is_empty() || session.state.phase != Phase::Running {
        return;
    }
    let fast = settings.fast_battle || input.tab;
    session.pace.tick(time.delta() * if fast { 2 } else { 1 });
    if !session.pace.is_finished() {
        return;
    }
    let before = session.state.log.len();
    session.state.step(&story.content.db);
    // Only pause after steps the player should see (not after starting a party turn).
    let visible = session.state.log[before..]
        .iter()
        .any(|e| !matches!(e, core::LogEntry::Turn { unit } if session.state.units[*unit].side == crate::content::defs::Side::Party));
    let secs = if settings.fast_battle {
        PACE_FAST
    } else {
        PACE_NORMAL
    };
    session.pace = Timer::from_seconds(if visible { secs } else { 0.0 }, TimerMode::Once);
    session.dirty = true;
}

/// Converts new log entries into text lines and floating numbers.
fn collect_log(mut session: ResMut<BattleSession>, story: Story) {
    if session.log_seen >= session.state.log.len() {
        return;
    }
    let entries: Vec<_> = session.state.log[session.log_seen..].to_vec();
    session.log_seen = session.state.log.len();
    for entry in &entries {
        if let Some(line) = text::log_line(entry, &session.state, &story.content, &story.progress) {
            session.lines.push_back(line);
        }
        match entry {
            core::LogEntry::Damage {
                target,
                amount,
                absorbed,
                ..
            } => {
                let text = if *absorbed > 0 && *amount == 0 {
                    format!("({absorbed})")
                } else {
                    format!("-{amount}")
                };
                session.popups.push((*target, text, 0, 0.0));
            }
            core::LogEntry::Heal { target, amount } if *amount > 0 => {
                session.popups.push((*target, format!("+{amount}"), 1, 0.0));
            }
            core::LogEntry::Miss { target } => {
                let text = story.content.text("ui.battle.miss", &story.progress);
                session.popups.push((*target, text, 2, 0.0));
            }
            _ => {}
        }
    }
    while session.lines.len() > LOG_LINES {
        session.lines.pop_front();
    }
    session.dirty = true;
}

fn single_target_kind(kind: TargetKind) -> bool {
    matches!(kind, TargetKind::Enemy | TargetKind::Ally)
}

/// Resolves a menu choice into a command (or a target prompt).
fn choose(session: &mut BattleSession, story: &Story, actor: usize) -> Option<Command> {
    let db = &story.content.db;
    let state = &session.state;
    let mode = session.mode.clone();
    let cursor = session.cursor;
    let target_prompt = |pending: PendingCommand, kind: TargetKind, melee: bool, back: MenuMode| {
        let candidates = state.valid_targets(actor, kind, melee);
        MenuMode::Target {
            pending,
            candidates,
            back: Box::new(back),
        }
    };
    let next_mode = match &mode {
        MenuMode::Root => match session.root_entries().get(cursor)? {
            RootEntry::Strike => Some(target_prompt(
                PendingCommand::Strike,
                TargetKind::Enemy,
                true,
                MenuMode::Root,
            )),
            RootEntry::Skills => Some(MenuMode::Skills),
            RootEntry::Artifacts => Some(MenuMode::Artifacts),
            RootEntry::Items => Some(MenuMode::Items),
            RootEntry::Formation => Some(MenuMode::Formation),
            RootEntry::Charge => return Some(Command::Charge),
            RootEntry::Guard => return Some(Command::Guard),
            RootEntry::Meditate => return Some(Command::Meditate),
            RootEntry::Flee => return Some(Command::Flee),
            RootEntry::EndTurn => return Some(Command::EndTurn),
            RootEntry::Swap => {
                let rank = state.units[actor].slot.rank();
                let candidates = state
                    .allies_of(actor)
                    .into_iter()
                    .filter(|&a| a != actor && state.units[a].slot.rank().abs_diff(rank) == 1)
                    .collect();
                Some(MenuMode::Target {
                    pending: PendingCommand::Swap,
                    candidates,
                    back: Box::new(MenuMode::Root),
                })
            }
        },
        MenuMode::Skills => {
            let slot = state.units[actor].skills.get(cursor)?;
            let skill = db.skill(&slot.id)?;
            if single_target_kind(skill.target) {
                Some(target_prompt(
                    PendingCommand::Skill(slot.id.clone()),
                    skill.target,
                    skill.melee,
                    mode.clone(),
                ))
            } else {
                return Some(Command::Skill(
                    slot.id.clone(),
                    state.default_target(actor, skill.target, skill.melee)?,
                ));
            }
        }
        MenuMode::Artifacts => {
            let slot = state.units[actor].artifacts.get(cursor)?;
            let skill = db
                .artifacts
                .get(&slot.id)?
                .active
                .as_ref()
                .and_then(|s| db.skill(s))?;
            if single_target_kind(skill.target) {
                Some(target_prompt(
                    PendingCommand::Artifact(cursor),
                    skill.target,
                    skill.melee,
                    mode.clone(),
                ))
            } else {
                return Some(Command::Artifact(
                    cursor,
                    state.default_target(actor, skill.target, skill.melee)?,
                ));
            }
        }
        MenuMode::Items => {
            let (id, _) = state.items.iter().filter(|(_, n)| **n > 0).nth(cursor)?;
            let item = db.items.get(id)?;
            if single_target_kind(item.target) {
                Some(target_prompt(
                    PendingCommand::Item(id.clone()),
                    item.target,
                    false,
                    mode.clone(),
                ))
            } else {
                return Some(Command::Item(
                    id.clone(),
                    state.default_target(actor, item.target, false)?,
                ));
            }
        }
        MenuMode::Formation => {
            return Some(if cursor == 0 {
                Command::ReleaseFormation
            } else {
                Command::Stabilize
            });
        }
        MenuMode::Target {
            pending,
            candidates,
            ..
        } => {
            return candidates.get(cursor).map(|&t| pending.with_target(t));
        }
    };
    if let Some(next) = next_mode {
        // A submenu with nothing in it (no items, no artifacts, no target)
        // would trap the cursor; stay where we are instead.
        let previous = std::mem::replace(&mut session.mode, next);
        if menu_len(session, actor) == 0 {
            session.mode = previous;
            return None;
        }
        session.cursors.push(session.cursor);
        session.cursor = 0;
        // Default target: the first candidate; for allies prefer the actor.
        if let MenuMode::Target { candidates, .. } = &session.mode
            && let Some(pos) = candidates.iter().position(|&c| c == actor)
        {
            session.cursor = pos;
        }
    }
    None
}

/// Number of rows in the current menu.
pub fn menu_len(session: &BattleSession, actor: usize) -> usize {
    match &session.mode {
        MenuMode::Root => session.root_entries().len(),
        MenuMode::Skills => session.state.units[actor].skills.len(),
        MenuMode::Artifacts => session.state.units[actor].artifacts.len(),
        MenuMode::Items => session.state.items.values().filter(|n| **n > 0).count(),
        MenuMode::Formation => 2,
        MenuMode::Target { candidates, .. } => candidates.len(),
    }
}

#[allow(clippy::too_many_arguments)]
fn battle_input(
    mut input: ResMut<MenuInput>,
    mut session: ResMut<BattleSession>,
    mut story: Story,
    mut next_play: ResMut<NextState<PlayState>>,
    mut next_game: ResMut<NextState<GameState>>,
    mut loads: MessageWriter<LoadRequest>,
) {
    // Tutorial hints first.
    if !session.hints.is_empty() {
        if input.take_confirm() {
            session.hints.remove(0);
            if session.hints.is_empty() {
                let flag = format!("tut.{}", session.state.encounter);
                story.run(&[StoryEffect::SetFlag(flag, 1)]);
            }
            session.dirty = true;
        }
        return;
    }

    if session.state.is_over() {
        let enc = story
            .content
            .db
            .encounters
            .get(&session.state.encounter)
            .cloned();
        let defeat_continues = enc.as_ref().is_some_and(|e| e.defeat_continues);
        let choices = session.end_choices(defeat_continues);
        let step = input.vertical();
        if step != 0 {
            session.end_cursor =
                (session.end_cursor as i32 + step).rem_euclid(choices.len() as i32) as usize;
            session.dirty = true;
        }
        if !input.take_confirm() {
            return;
        }
        match choices
            .get(session.end_cursor)
            .copied()
            .unwrap_or(EndChoice::Continue)
        {
            EndChoice::Continue => {
                apply_result(&session.state, &mut story);
                next_play.set(PlayState::Exploring);
            }
            EndChoice::Retry => {
                let initial = session.initial.clone();
                session.state = initial;
                session.log_seen = session.state.log.len();
                session.lines.clear();
                session.mode = MenuMode::Root;
                session.cursor = 0;
                session.end_cursor = 0;
                session.dirty = true;
            }
            EndChoice::LoadLast => {
                let dir = crate::save::save_dir();
                match crate::save::latest_save(&dir) {
                    Some(slot) => {
                        loads.write(LoadRequest(slot));
                    }
                    None => next_game.set(GameState::Menu),
                }
            }
            EndChoice::Title => next_game.set(GameState::Menu),
        }
        return;
    }

    let Phase::Command(actor) = session.state.phase else {
        return;
    };
    let n = menu_len(&session, actor).max(1);
    let step = input.vertical()
        + if matches!(session.mode, MenuMode::Target { .. }) {
            input.horizontal()
        } else {
            0
        };
    if step != 0 {
        session.cursor = (session.cursor as i32 + step).rem_euclid(n as i32) as usize;
        session.dirty = true;
    }
    if input.cancel && !input.consumed {
        input.consumed = true;
        let back = match &session.mode {
            MenuMode::Root => None,
            MenuMode::Target { back, .. } => Some((**back).clone()),
            _ => Some(MenuMode::Root),
        };
        if let Some(back) = back {
            session.mode = back;
            session.cursor = session.cursors.pop().unwrap_or(0);
            session.dirty = true;
        }
        return;
    }
    if !input.take_confirm() {
        return;
    }
    let Some(command) = choose(&mut session, &story, actor) else {
        session.dirty = true;
        return;
    };
    let db = &story.content.db;
    match session.state.execute(db, command) {
        Ok(()) => {
            session.mode = MenuMode::Root;
            session.cursors.clear();
            // Keep the cursor where it was for repeated actions.
            if session.state.phase != Phase::Command(actor) {
                session.cursor = 0;
            }
            session.pace = Timer::from_seconds(0.35, TimerMode::Once);
        }
        Err(reason) => {
            let line = story.content.text(reason.key(), &story.progress);
            session.lines.push_back(line);
        }
    }
    session.dirty = true;
}

/// Victory/defeat/flee consequences for the story state.
fn apply_result(state: &BattleState, story: &mut Story) {
    let result = state.result(&story.content.db);
    for (item, n) in &result.items_used {
        story.progress.remove_item(item, *n);
    }
    story.progress.battles_fought += 1;
    let mut effects = Vec::new();
    if result.overcharge_releases > 0 {
        effects.push(StoryEffect::AddFlag(
            "stat.tam_ma".into(),
            result.overcharge_releases as i32,
        ));
    }
    let enc = story.content.db.encounters.get(&state.encounter).cloned();
    if result.victory {
        if result.tu_vi > 0 {
            effects.push(StoryEffect::GainTuVi(result.tu_vi));
        }
        for (item, n) in &result.drops {
            effects.push(StoryEffect::GiveItem(item.clone(), *n));
        }
        if let Some(enc) = &enc {
            effects.extend(enc.on_victory.iter().cloned());
        }
    } else if state.phase == Phase::Defeat
        && let Some(enc) = &enc
    {
        effects.extend(enc.on_defeat.iter().cloned());
    }
    story.run(&effects);
}

fn end_battle(
    mut commands: Commands,
    mut saves: MessageWriter<crate::save::SaveRequest>,
    session: Option<Res<BattleSession>>,
) {
    // Keep an autosave after story battles so a crash never loses them.
    if session.is_some_and(|s| s.state.phase == Phase::Victory) {
        saves.write(crate::save::SaveRequest(SaveSlot::Auto));
    }
    commands.remove_resource::<BattleSession>();
}

//! Session lifecycle and the story action queue.
//!
//! Effects from dialogue, triggers, objects and battles all go through
//! [`Story::run`]. Deferred actions (start a dialogue, a battle, a warp, a
//! card, an autosave) are queued and executed one at a time while exploring,
//! so content can chain them freely.

use std::collections::VecDeque;

use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    GameState, PlayState,
    battle::{BattleRequest, core::BattleState},
    content::{Content, defs::StoryEffect},
    dialogue::DialogueRequest,
    hud::CardRequest,
    input::MenuInput,
    save::{LoadRequest, SaveFile, SaveRequest, SaveSlot},
    story::{Deferred, Notice, Progress},
    transition::PendingWarp,
};

/// Systems that need `Progress` on entering `Playing` run after this set.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionSetup;

/// How the next `Playing` session starts.
#[derive(Resource, Debug)]
pub enum PendingStart {
    New(Box<Progress>),
    Load(Box<SaveFile>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueuedAction {
    Deferred(Deferred),
    ResumeDialogue { dialogue: String, node: String },
    ResumeBattle(Box<BattleState>),
}

/// Deferred story actions waiting for the player to be free.
#[derive(Resource, Default, Debug)]
pub struct ActionQueue(pub VecDeque<QueuedAction>);

/// Messages for the HUD (items gained, quests updated…).
#[derive(Resource, Default, Debug)]
pub struct Notices(pub VecDeque<Notice>);

/// Everything needed to apply story effects from a system.
#[derive(SystemParam)]
pub struct Story<'w> {
    pub progress: ResMut<'w, Progress>,
    pub content: Res<'w, Content>,
    pub queue: ResMut<'w, ActionQueue>,
    pub notices: ResMut<'w, Notices>,
}

impl Story<'_> {
    pub fn run(&mut self, effects: &[StoryEffect]) {
        let outcome = self.progress.apply_all(effects, &self.content.db);
        self.absorb(outcome);
    }

    /// Queues the deferred actions and shows the notices of an outcome.
    pub fn absorb(&mut self, outcome: crate::story::Outcome) {
        self.queue
            .0
            .extend(outcome.deferred.into_iter().map(QueuedAction::Deferred));
        self.notices.0.extend(outcome.notices);
    }
}

pub struct FlowPlugin;

impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionQueue>()
            .init_resource::<Notices>()
            .add_systems(
                OnEnter(GameState::Starting),
                |mut next: ResMut<NextState<GameState>>| {
                    next.set(GameState::Playing);
                },
            )
            .add_systems(
                OnEnter(GameState::Playing),
                begin_session.in_set(SessionSetup),
            )
            .add_systems(OnExit(GameState::Playing), end_session)
            .add_systems(
                Update,
                (
                    run_queue.run_if(in_state(PlayState::Exploring)),
                    count_play_time.run_if(in_state(GameState::Playing)),
                    quick_save_load.run_if(
                        in_state(PlayState::Exploring)
                            .or_else(in_state(PlayState::Dialogue))
                            .or_else(in_state(PlayState::Battle)),
                    ),
                ),
            );
    }
}

fn begin_session(
    mut commands: Commands,
    pending: Option<Res<PendingStart>>,
    content: Res<Content>,
    mut queue: ResMut<ActionQueue>,
    mut notices: ResMut<Notices>,
) {
    queue.0.clear();
    notices.0.clear();
    let Some(pending) = pending else {
        error!("entered Playing without a pending start");
        return;
    };
    match &*pending {
        PendingStart::New(progress) => {
            let mut progress = (**progress).clone();
            let Some(chapter) = content.db.chapters.get(&progress.chapter).cloned() else {
                error!("chapter {} is not defined", progress.chapter);
                return;
            };
            progress.level = chapter.start_level.clone();
            if let Some(feet) = chapter.start_feet {
                progress.feet = feet;
            }
            let outcome = progress.apply_all(&chapter.start_effects, &content.db);
            queue
                .0
                .extend(outcome.deferred.into_iter().map(QueuedAction::Deferred));
            notices.0.extend(outcome.notices);
            commands.insert_resource(PendingWarp::to(&progress.level, IVec2::from(progress.feet)));
            commands.insert_resource(progress);
        }
        PendingStart::Load(save) => {
            let progress = save.progress.clone();
            if let Some(battle) = &save.battle {
                queue
                    .0
                    .push_back(QueuedAction::ResumeBattle(battle.clone()));
            }
            if let Some(dialogue) = &save.dialogue {
                queue.0.push_back(QueuedAction::ResumeDialogue {
                    dialogue: dialogue.dialogue.clone(),
                    node: dialogue.node.clone(),
                });
            }
            queue.0.extend(save.pending_actions.iter().cloned());
            commands.insert_resource(PendingWarp::to(&progress.level, IVec2::from(progress.feet)));
            commands.insert_resource(progress);
        }
    }
    commands.remove_resource::<PendingStart>();
}

fn end_session(mut commands: Commands, mut queue: ResMut<ActionQueue>) {
    queue.0.clear();
    commands.remove_resource::<Progress>();
}

fn run_queue(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    mut next_state: ResMut<NextState<PlayState>>,
    mut saves: ParamSet<(MessageReader<SaveRequest>, MessageWriter<SaveRequest>)>,
    input: Res<MenuInput>,
) {
    // Keep queued actions in the snapshot until the save has been written.
    let saving = saves.p0().read().count() > 0;
    if saving || input.quick_save {
        return;
    }
    let Some(action) = queue.0.pop_front() else {
        return;
    };
    match action {
        QueuedAction::Deferred(Deferred::Dialogue(id)) => {
            commands.insert_resource(DialogueRequest::Start(id));
            next_state.set(PlayState::Dialogue);
        }
        QueuedAction::ResumeDialogue { dialogue, node } => {
            commands.insert_resource(DialogueRequest::Resume { dialogue, node });
            next_state.set(PlayState::Dialogue);
        }
        QueuedAction::Deferred(Deferred::Battle(id)) => {
            commands.insert_resource(BattleRequest::Start(id));
            next_state.set(PlayState::Battle);
        }
        QueuedAction::ResumeBattle(state) => {
            commands.insert_resource(BattleRequest::Resume(state));
            next_state.set(PlayState::Battle);
        }
        QueuedAction::Deferred(Deferred::Warp { level, x, y }) => {
            commands.insert_resource(PendingWarp::to(&level, IVec2::new(x, y)));
            next_state.set(PlayState::Transition);
        }
        QueuedAction::Deferred(Deferred::Card(key)) => {
            commands.insert_resource(CardRequest(key));
            next_state.set(PlayState::Card);
        }
        QueuedAction::Deferred(Deferred::Autosave) => {
            saves.p1().write(SaveRequest(SaveSlot::Auto));
        }
        QueuedAction::Deferred(Deferred::Shop(id)) => {
            commands.insert_resource(crate::workshop::WorkshopRequest::Shop(id));
            next_state.set(PlayState::Workshop);
        }
        QueuedAction::Deferred(Deferred::Craft(station)) => {
            commands.insert_resource(crate::workshop::WorkshopRequest::Craft(station));
            next_state.set(PlayState::Workshop);
        }
    }
}

fn count_play_time(time: Res<Time>, progress: Option<ResMut<Progress>>) {
    if let Some(mut progress) = progress {
        progress.play_time += time.delta_secs_f64();
    }
}

fn quick_save_load(
    input: Res<MenuInput>,
    mut saves: MessageWriter<SaveRequest>,
    mut loads: MessageWriter<LoadRequest>,
) {
    if input.quick_save {
        saves.write(SaveRequest(SaveSlot::Quick));
    }
    if input.quick_load {
        loads.write(LoadRequest(SaveSlot::Quick));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::DialogueSnapshot;
    use bevy::ecs::system::RunSystemOnce;

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<ActionQueue>()
            .init_resource::<Notices>()
            .init_resource::<MenuInput>()
            .init_resource::<NextState<PlayState>>()
            .add_message::<SaveRequest>();
        app
    }

    #[test]
    fn loading_resumes_dialogue_before_pending_actions() {
        let (db, locale, errors) = crate::content::load_from_dir(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"),
        )
        .expect("assets");
        assert!(errors.is_empty(), "{errors:?}");
        let progress = Progress::new_game(&db, Default::default());
        let mut save = SaveFile::new(
            &progress,
            Some(DialogueSnapshot {
                dialogue: "ch1_prologue".into(),
                node: "n5".into(),
            }),
            None,
        );
        let pending = [
            QueuedAction::Deferred(Deferred::Dialogue("ch1_ong_mac_morning".into())),
            QueuedAction::Deferred(Deferred::Autosave),
        ];
        save.pending_actions.extend(pending.clone());
        let save = serde_json::from_str(&serde_json::to_string(&save).expect("serialize"))
            .expect("deserialize");
        let mut app = app();
        app.insert_resource(Content { db, locale })
            .insert_resource(PendingStart::Load(Box::new(save)));
        app.world_mut()
            .run_system_once(begin_session)
            .expect("begin session");
        let queue = &app.world().resource::<ActionQueue>().0;
        assert_eq!(queue.len(), 3);
        assert_eq!(
            queue[0],
            QueuedAction::ResumeDialogue {
                dialogue: "ch1_prologue".into(),
                node: "n5".into(),
            }
        );
        assert_eq!(queue[1], pending[0]);
        assert_eq!(queue[2], pending[1]);
    }

    #[test]
    fn saving_does_not_start_or_drop_a_queued_action() {
        for quick in [false, true] {
            let mut app = app();
            let action = QueuedAction::Deferred(Deferred::Battle("ch1_da_tru".into()));
            app.world_mut()
                .resource_mut::<ActionQueue>()
                .0
                .push_back(action.clone());
            if quick {
                app.world_mut().resource_mut::<MenuInput>().quick_save = true;
            } else {
                app.world_mut().write_message(SaveRequest(SaveSlot::Auto));
            }
            app.world_mut()
                .run_system_once(run_queue)
                .expect("run queue");
            assert_eq!(
                app.world().resource::<ActionQueue>().0.front(),
                Some(&action)
            );
            assert!(!app.world().contains_resource::<BattleRequest>());
        }
    }
}

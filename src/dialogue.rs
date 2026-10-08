//! Branching dialogue: data-driven graph (content-schema §3.10) with
//! conditions, effects, `{playerName}` and gendered addressing, shown in a
//! classic JRPG box with typewriter text and choices.

use bevy::prelude::*;

use crate::{
    PlayState,
    asset::GameAssets,
    content::{
        Content,
        defs::{Condition, DialogueDef, DialogueNode, StoryEffect},
    },
    flow::Story,
    input::MenuInput,
    save::DialogueSnapshot,
    story::Progress,
    ui::{self, FontKind},
};

/// Characters revealed per second by the typewriter effect.
const CHARS_PER_SECOND: f32 = 55.0;

/// Asks the dialogue plugin to open a conversation.
#[derive(Resource, Debug, Clone)]
pub enum DialogueRequest {
    Start(String),
    /// Continue a saved conversation at a node without re-running its effects.
    Resume {
        dialogue: String,
        node: String,
    },
}

/// The conversation on screen. Present only in [`PlayState::Dialogue`].
#[derive(Resource, Debug, Clone)]
pub struct DialogueSession {
    pub dialogue: String,
    pub node: String,
    /// Characters of the current line revealed so far.
    revealed: f32,
    /// Resolved text of the current line.
    text: String,
    speaker: Option<String>,
    /// (option id, resolved text) of visible choices.
    options: Vec<(String, String)>,
    cursor: usize,
}

impl DialogueSession {
    pub fn snapshot(&self) -> Option<DialogueSnapshot> {
        Some(DialogueSnapshot {
            dialogue: self.dialogue.clone(),
            node: self.node.clone(),
        })
    }

    fn line_len(&self) -> usize {
        self.text.chars().count()
    }

    fn is_line_complete(&self) -> bool {
        self.revealed as usize >= self.line_len()
    }

    fn visible_text(&self) -> String {
        self.text.chars().take(self.revealed as usize).collect()
    }
}

/// Where the graph walk ended up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Show this text node.
    Show(String),
    End,
}

/// What the dialogue walker needs from the game: run effects, test conditions.
pub trait DialogueHost {
    fn run(&mut self, effects: &[StoryEffect]);
    fn eval(&self, condition: &Condition) -> bool;
}

impl DialogueHost for Story<'_> {
    fn run(&mut self, effects: &[StoryEffect]) {
        Story::run(self, effects);
    }

    fn eval(&self, condition: &Condition) -> bool {
        self.progress.eval(condition)
    }
}

/// Walks from `start` through branch/effect nodes to the next node with text,
/// running every effect list on the way (including the text node's own
/// effects unless `skip_first_effects`). Pure: unit tested without ECS.
pub fn walk(
    def: &DialogueDef,
    start: Option<&str>,
    host: &mut dyn DialogueHost,
    skip_first_effects: bool,
) -> Step {
    let mut current = start.map(str::to_string);
    let mut skip = skip_first_effects;
    // Guard against accidental loops of non-text nodes.
    for _ in 0..64 {
        let Some(id) = current.take() else {
            return Step::End;
        };
        let Some(node) = def.nodes.get(&id) else {
            error!("dialogue {}: missing node {id}", def.id);
            return Step::End;
        };
        match node {
            DialogueNode::Line { effects, .. } | DialogueNode::Choice { effects, .. } => {
                if !skip {
                    host.run(effects);
                }
                return Step::Show(id);
            }
            DialogueNode::Branch { arms, default } => {
                current = arms
                    .iter()
                    .find(|a| host.eval(&a.when))
                    .map(|a| a.next.clone())
                    .or_else(|| default.clone());
            }
            DialogueNode::Effects { effects, next } => {
                host.run(effects);
                current = next.clone();
            }
        }
        skip = false;
    }
    error!("dialogue {}: too many non-text nodes in a row", def.id);
    Step::End
}

#[derive(Component)]
struct DialogueBox;

#[derive(Component)]
struct SpeakerText;

#[derive(Component)]
struct DialogueText;

#[derive(Component)]
struct ChoiceList;

#[derive(Component)]
struct ContinueIndicator;

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlayState::Dialogue),
            (open_dialogue, spawn_dialogue_box).chain(),
        )
        .add_systems(OnExit(PlayState::Dialogue), close_dialogue)
        .add_systems(
            Update,
            (advance_dialogue, update_dialogue_ui)
                .chain()
                .run_if(in_state(PlayState::Dialogue).and_then(resource_exists::<DialogueSession>)),
        );
    }
}

/// Resolves the text node `node` into the session (text, speaker, options).
fn load_node(
    session: &mut DialogueSession,
    def: &DialogueDef,
    node: &str,
    content: &Content,
    progress: &Progress,
) {
    session.node = node.to_string();
    session.revealed = 0.0;
    session.cursor = 0;
    session.options.clear();
    let key = format!("dlg.{}.{node}", def.id);
    session.text = content.text(&key, progress);
    match def.nodes.get(node) {
        Some(DialogueNode::Line { speaker, .. }) => session.speaker = speaker.clone(),
        Some(DialogueNode::Choice {
            speaker, options, ..
        }) => {
            session.speaker = speaker.clone();
            session.options = options
                .iter()
                .filter(|o| progress.eval_opt(&o.when))
                .map(|o| {
                    (
                        o.id.clone(),
                        content.text(&format!("{key}.{}", o.id), progress),
                    )
                })
                .collect();
        }
        _ => session.speaker = None,
    }
}

fn open_dialogue(
    mut commands: Commands,
    request: Option<Res<DialogueRequest>>,
    mut story: Story,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    let Some(request) = request.map(|r| r.clone()) else {
        next_state.set(PlayState::Exploring);
        return;
    };
    commands.remove_resource::<DialogueRequest>();
    let (id, start, resume) = match request {
        DialogueRequest::Start(id) => (id, "start".to_string(), false),
        DialogueRequest::Resume { dialogue, node } => (dialogue, node, true),
    };
    let Some(def) = story.content.db.dialogues.get(&id).cloned() else {
        error!("unknown dialogue `{id}`");
        next_state.set(PlayState::Exploring);
        return;
    };
    let step = walk(&def, Some(&start), &mut story, resume);
    match step {
        Step::Show(node) => {
            let mut session = DialogueSession {
                dialogue: id,
                node: String::new(),
                revealed: 0.0,
                text: String::new(),
                speaker: None,
                options: Vec::new(),
                cursor: 0,
            };
            load_node(&mut session, &def, &node, &story.content, &story.progress);
            commands.insert_resource(session);
        }
        Step::End => next_state.set(PlayState::Exploring),
    }
}

fn spawn_dialogue_box(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Name::new("DialogueBox"),
        DialogueBox,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(28.0),
            right: Val::Px(28.0),
            bottom: Val::Px(20.0),
            min_height: Val::Px(170.0),
            padding: UiRect::axes(Val::Px(22.0), Val::Px(16.0)),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(ui::PANEL),
        BorderColor::all(ui::BORDER),
        GlobalZIndex(20),
        children![
            (
                SpeakerText,
                Text::new(""),
                ui::font(&assets, FontKind::Bold, 21.0),
                TextColor(ui::GOLD)
            ),
            (
                DialogueText,
                Text::new(""),
                ui::font(&assets, FontKind::Body, 21.0),
                TextColor(ui::TEXT)
            ),
            (
                ChoiceList,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
            ),
            (
                ContinueIndicator,
                Text::new("›"),
                ui::font(&assets, FontKind::Bold, 24.0),
                TextColor(ui::GOLD),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    bottom: Val::Px(6.0),
                    ..default()
                },
            ),
        ],
    ));
}

fn advance_dialogue(
    time: Res<Time>,
    mut input: ResMut<MenuInput>,
    mut session: ResMut<DialogueSession>,
    mut story: Story,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    session.revealed += time.delta_secs() * CHARS_PER_SECOND;
    let choosing = !session.options.is_empty() && session.is_line_complete();
    if choosing {
        let n = session.options.len();
        let step = input.vertical();
        if step != 0 {
            session.cursor = (session.cursor as i32 + step).rem_euclid(n as i32) as usize;
        }
    }
    if !input.take_confirm() {
        return;
    }
    if !session.is_line_complete() {
        session.revealed = session.line_len() as f32;
        return;
    }
    let Some(def) = story.content.db.dialogues.get(&session.dialogue).cloned() else {
        next_state.set(PlayState::Exploring);
        return;
    };
    let next = match def.nodes.get(&session.node) {
        Some(DialogueNode::Line { next, .. }) => next.clone(),
        Some(DialogueNode::Choice { options, .. }) => {
            let Some((chosen, _)) = session.options.get(session.cursor).cloned() else {
                return;
            };
            let option = options.iter().find(|o| o.id == chosen).cloned();
            match option {
                Some(option) => {
                    story.run(&option.effects);
                    option.next
                }
                None => None,
            }
        }
        _ => None,
    };
    match walk(&def, next.as_deref(), &mut story, false) {
        Step::Show(node) => load_node(&mut session, &def, &node, &story.content, &story.progress),
        Step::End => next_state.set(PlayState::Exploring),
    }
}

#[allow(clippy::too_many_arguments)]
fn update_dialogue_ui(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    session: Res<DialogueSession>,
    content: Res<Content>,
    progress: Res<Progress>,
    mut speaker: Single<&mut Text, (With<SpeakerText>, Without<DialogueText>)>,
    mut text: Single<&mut Text, (With<DialogueText>, Without<SpeakerText>)>,
    mut indicator: Single<&mut Visibility, With<ContinueIndicator>>,
    choices: Single<(Entity, Option<&Children>), With<ChoiceList>>,
) {
    let name = session
        .speaker
        .as_deref()
        .map(|s| content.character_name(s, &progress))
        .unwrap_or_default();
    if speaker.0 != name {
        speaker.0 = name;
    }
    let visible = session.visible_text();
    if text.0 != visible {
        text.0 = visible;
    }
    let complete = session.is_line_complete();
    // Blink the "continue" arrow once the line is fully shown.
    let blink_on = ((time.elapsed_secs() * 3.0) as u32).is_multiple_of(2);
    **indicator = if complete && session.options.is_empty() && blink_on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    // Rebuild the choice list when it changes.
    let (list, children) = *choices;
    let wanted = if complete { session.options.len() } else { 0 };
    let have = children.map_or(0, |c| c.len());
    if session.is_changed() || wanted != have {
        commands.entity(list).despawn_children();
        if complete {
            for (i, (_, label)) in session.options.iter().enumerate() {
                let selected = i == session.cursor;
                commands.entity(list).with_child((
                    Text::new(format!("{} {label}", if selected { "›" } else { " " })),
                    ui::font(
                        &assets,
                        if selected {
                            FontKind::Bold
                        } else {
                            FontKind::Body
                        },
                        20.0,
                    ),
                    TextColor(if selected { ui::GOLD } else { ui::TEXT_DIM }),
                ));
            }
        }
    }
}

fn close_dialogue(mut commands: Commands, boxes: Query<Entity, With<DialogueBox>>) {
    for entity in &boxes {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<DialogueSession>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::defs::DataFile;

    #[derive(Default)]
    struct Host {
        flags: Vec<String>,
        ran: Vec<StoryEffect>,
    }

    impl DialogueHost for Host {
        fn run(&mut self, effects: &[StoryEffect]) {
            for e in effects {
                if let StoryEffect::SetFlag(name, _) = e {
                    self.flags.push(name.clone());
                }
                self.ran.push(e.clone());
            }
        }

        fn eval(&self, condition: &Condition) -> bool {
            matches!(condition, Condition::Flag(f) if self.flags.contains(f))
        }
    }

    fn def() -> DialogueDef {
        let file: DataFile = ron::from_str(
            r#"(dialogues: [(id: "d", nodes: {
                "start": Effects(effects: [SetFlag("met", 1)], next: Some("check")),
                "check": Branch(arms: [(when: Flag("rich"), next: "rich")], default: Some("poor")),
                "rich": Line(speaker: Some("a"), next: None),
                "poor": Line(speaker: Some("a"), next: Some("again"), effects: [SetFlag("poor_seen", 1)]),
                "again": Branch(arms: [(when: Flag("met"), next: "ask")], default: None),
                "ask": Choice(options: [(id: "yes", next: None), (id: "no", next: None)]),
            })])"#,
        )
        .expect("parses");
        file.dialogues.into_iter().next().expect("one dialogue")
    }

    #[test]
    fn walk_runs_effects_and_follows_branches() {
        let def = def();
        let mut host = Host::default();
        assert_eq!(
            walk(&def, Some("start"), &mut host, false),
            Step::Show("poor".into())
        );
        assert_eq!(
            host.ran,
            vec![
                StoryEffect::SetFlag("met".into(), 1),
                StoryEffect::SetFlag("poor_seen".into(), 1)
            ]
        );
        // Effects applied during the walk are visible to later branches.
        assert_eq!(
            walk(&def, Some("again"), &mut host, false),
            Step::Show("ask".into())
        );
        let mut rich = Host {
            flags: vec!["rich".into()],
            ..Default::default()
        };
        assert_eq!(
            walk(&def, Some("start"), &mut rich, false),
            Step::Show("rich".into())
        );
    }

    #[test]
    fn resume_skips_the_node_effects() {
        let def = def();
        let mut host = Host::default();
        assert_eq!(
            walk(&def, Some("poor"), &mut host, true),
            Step::Show("poor".into())
        );
        assert!(host.ran.is_empty());
        assert_eq!(walk(&def, None, &mut host, false), Step::End);
    }
}

//! Classic JRPG text box: speaker name, typewriter text, press to continue.

use bevy::prelude::*;

use crate::{PlayState, asset::GameAssets, npc::interact_pressed};

/// Characters revealed per second by the typewriter effect.
const CHARS_PER_SECOND: f32 = 45.0;

const BOX_COLOR: Color = Color::srgba(0.05, 0.07, 0.22, 0.97);
const BORDER_COLOR: Color = Color::srgb(0.95, 0.95, 1.0);
const NAME_COLOR: Color = Color::srgb(1.0, 0.85, 0.35);
const TEXT_COLOR: Color = Color::WHITE;

/// The conversation currently on screen. Present only in [`PlayState::Dialogue`].
#[derive(Resource, Debug, Clone)]
pub struct Dialogue {
    pub speaker: String,
    pub lines: Vec<String>,
    pub current: usize,
    /// Characters of the current line revealed so far.
    revealed: f32,
}

impl Dialogue {
    pub fn new(speaker: String, lines: Vec<String>) -> Self {
        Self {
            speaker,
            lines,
            current: 0,
            revealed: 0.0,
        }
    }

    fn line(&self) -> &str {
        self.lines.get(self.current).map_or("", String::as_str)
    }

    fn line_len(&self) -> usize {
        self.line().chars().count()
    }

    fn is_line_complete(&self) -> bool {
        self.revealed as usize >= self.line_len()
    }

    fn visible_text(&self) -> String {
        self.line().chars().take(self.revealed as usize).collect()
    }

    /// Handles a confirm press. Returns `false` once the conversation is over.
    pub fn advance(&mut self) -> bool {
        if !self.is_line_complete() {
            self.revealed = self.line_len() as f32;
            return true;
        }
        self.current += 1;
        self.revealed = 0.0;
        self.current < self.lines.len()
    }
}

#[derive(Component)]
struct DialogueBox;

#[derive(Component)]
struct DialogueText;

#[derive(Component)]
struct ContinueIndicator;

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayState::Dialogue), spawn_dialogue_box)
            .add_systems(OnExit(PlayState::Dialogue), close_dialogue)
            .add_systems(
                Update,
                (advance_dialogue, update_dialogue_text)
                    .chain()
                    .run_if(in_state(PlayState::Dialogue)),
            );
    }
}

fn spawn_dialogue_box(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    dialogue: Option<Res<Dialogue>>,
) {
    let speaker = dialogue.map(|d| d.speaker.clone()).unwrap_or_default();
    let font = |size: f32| TextFont {
        font: game_assets.grumon_font.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    };

    commands.spawn((
        Name::new("DialogueBox"),
        DialogueBox,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            right: Val::Px(24.0),
            bottom: Val::Px(20.0),
            height: Val::Px(150.0),
            padding: UiRect::axes(Val::Px(20.0), Val::Px(14.0)),
            border: UiRect::all(Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(BOX_COLOR),
        BorderColor::all(BORDER_COLOR),
        GlobalZIndex(10),
        children![
            (Text::new(speaker), font(22.0), TextColor(NAME_COLOR)),
            (
                DialogueText,
                Text::new(""),
                font(22.0),
                TextColor(TEXT_COLOR)
            ),
            (
                ContinueIndicator,
                Text::new("v"),
                font(18.0),
                TextColor(NAME_COLOR),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    bottom: Val::Px(8.0),
                    ..default()
                },
            ),
        ],
    ));
}

fn advance_dialogue(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut dialogue: ResMut<Dialogue>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    dialogue.revealed += time.delta_secs() * CHARS_PER_SECOND;
    if interact_pressed(&keyboard) && !dialogue.advance() {
        next_state.set(PlayState::Exploring);
    }
}

fn update_dialogue_text(
    time: Res<Time>,
    dialogue: Res<Dialogue>,
    mut text: Single<&mut Text, With<DialogueText>>,
    mut indicator: Single<&mut Visibility, With<ContinueIndicator>>,
) {
    let visible = dialogue.visible_text();
    if text.0 != visible {
        text.0 = visible;
    }
    // Blink the "continue" arrow once the line is fully shown.
    let blink_on = ((time.elapsed_secs() * 3.0) as u32).is_multiple_of(2);
    **indicator = if dialogue.is_line_complete() && blink_on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

fn close_dialogue(mut commands: Commands, boxes: Query<Entity, With<DialogueBox>>) {
    for entity in &boxes {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<Dialogue>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_press_reveals_line_second_press_advances() {
        let mut d = Dialogue::new("A".into(), vec!["Hello".into(), "Bye".into()]);
        assert_eq!(d.visible_text(), "");
        assert!(d.advance());
        assert_eq!(d.visible_text(), "Hello");
        assert!(d.advance());
        assert_eq!(d.current, 1);
        assert_eq!(d.visible_text(), "");
        assert!(d.advance());
        assert!(!d.advance());
    }
}

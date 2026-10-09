//! One place that maps keyboard and gamepad to game actions.

use bevy::prelude::*;

/// Actions pressed this frame (menus) and the movement axis (field).
#[derive(Resource, Default, Debug, Clone)]
pub struct MenuInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub confirm: bool,
    pub cancel: bool,
    /// Open/close the pause menu.
    pub menu: bool,
    /// Secondary action (battle speed, input method toggle).
    pub tab: bool,
    pub quick_save: bool,
    pub quick_load: bool,
    /// Normalised-ish movement direction for walking.
    pub axis: Vec2,
    /// Set by a system that consumed `confirm`, so later systems in the same
    /// frame ignore it (e.g. closing a dialogue must not re-open it).
    pub consumed: bool,
}

impl MenuInput {
    /// `confirm` if nobody consumed it yet; consumes it.
    pub fn take_confirm(&mut self) -> bool {
        if self.confirm && !self.consumed {
            self.consumed = true;
            true
        } else {
            false
        }
    }

    /// Vertical menu step: -1 up, +1 down.
    pub fn vertical(&self) -> i32 {
        self.down as i32 - self.up as i32
    }

    pub fn horizontal(&self) -> i32 {
        self.right as i32 - self.left as i32
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuInput>()
            .add_systems(PreUpdate, read_input.after(bevy::input::InputSystems));
    }
}

const STICK_DEADZONE: f32 = 0.35;

fn read_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut input: ResMut<MenuInput>,
) {
    let key = |codes: &[KeyCode]| keyboard.any_just_pressed(codes.iter().copied());
    let held = |codes: &[KeyCode]| keyboard.any_pressed(codes.iter().copied()) as i32 as f32;

    let mut next = MenuInput {
        up: key(&[KeyCode::ArrowUp, KeyCode::KeyW]),
        down: key(&[KeyCode::ArrowDown, KeyCode::KeyS]),
        left: key(&[KeyCode::ArrowLeft, KeyCode::KeyA]),
        right: key(&[KeyCode::ArrowRight, KeyCode::KeyD]),
        confirm: key(&[
            KeyCode::Space,
            KeyCode::Enter,
            KeyCode::NumpadEnter,
            KeyCode::KeyE,
            KeyCode::KeyZ,
        ]),
        cancel: key(&[KeyCode::Escape, KeyCode::KeyX, KeyCode::Backspace]),
        menu: key(&[KeyCode::Escape]),
        tab: key(&[KeyCode::Tab]),
        quick_save: key(&[KeyCode::F5]),
        quick_load: key(&[KeyCode::F9]),
        axis: Vec2::new(
            held(&[KeyCode::KeyD, KeyCode::ArrowRight])
                - held(&[KeyCode::KeyA, KeyCode::ArrowLeft]),
            held(&[KeyCode::KeyW, KeyCode::ArrowUp]) - held(&[KeyCode::KeyS, KeyCode::ArrowDown]),
        ),
        consumed: false,
    };

    for pad in &gamepads {
        next.up |= pad.just_pressed(GamepadButton::DPadUp);
        next.down |= pad.just_pressed(GamepadButton::DPadDown);
        next.left |= pad.just_pressed(GamepadButton::DPadLeft);
        next.right |= pad.just_pressed(GamepadButton::DPadRight);
        next.confirm |= pad.just_pressed(GamepadButton::South);
        next.cancel |= pad.just_pressed(GamepadButton::East);
        next.menu |= pad.just_pressed(GamepadButton::Start);
        next.tab |= pad.just_pressed(GamepadButton::North);
        let mut axis = pad.left_stick();
        if axis.length() < STICK_DEADZONE {
            axis = Vec2::ZERO;
        }
        let dpad = Vec2::new(
            pad.pressed(GamepadButton::DPadRight) as i32 as f32
                - pad.pressed(GamepadButton::DPadLeft) as i32 as f32,
            pad.pressed(GamepadButton::DPadUp) as i32 as f32
                - pad.pressed(GamepadButton::DPadDown) as i32 as f32,
        );
        if next.axis == Vec2::ZERO {
            next.axis = if dpad != Vec2::ZERO { dpad } else { axis };
        }
    }
    *input = next;
}

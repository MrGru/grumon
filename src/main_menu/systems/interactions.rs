use bevy::prelude::*;

use crate::{
    GameState,
    main_menu::{components::MenuButton, styles::*},
};

type ChangedButtons<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<MenuButton>),
>;

pub fn button_feedback(mut buttons: ChangedButtons) {
    for (interaction, mut color) in &mut buttons {
        color.0 = match interaction {
            Interaction::Pressed => PRESSED_BUTTON_COLOR,
            Interaction::Hovered => HOVERED_BUTTON_COLOR,
            Interaction::None => NORMAL_BUTTON_COLOR,
        };
    }
}

pub fn menu_actions(
    buttons: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button {
            MenuButton::Play => next_state.set(GameState::Playing),
            MenuButton::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

pub fn menu_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        next_state.set(GameState::Playing);
    }
}

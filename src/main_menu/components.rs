use bevy::prelude::*;

#[derive(Component)]
pub struct MainMenu;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuButton {
    Play,
    Quit,
}

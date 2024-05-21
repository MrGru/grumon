use bevy::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Component)]
pub struct Collider;

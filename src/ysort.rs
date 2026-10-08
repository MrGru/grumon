use bevy::prelude::*;

/// World-space z where y-sorted sprites start. Must stay above every LDtk
/// tile layer (bevy_ecs_ldtk stacks layers at z = 0, 1, 2, ...).
const Y_SORT_BASE_Z: f32 = 10.0;
/// How much z changes per pixel of y. Maps up to ~10 000 px tall stay in range.
const Y_SORT_SCALE: f32 = 0.0001;

/// Draw order follows the sprite's feet: lower on screen = drawn in front.
#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component)]
pub struct YSort {
    /// Extra bias added to the computed z.
    pub z: f32,
    /// Distance from the entity's origin (sprite center) down to its feet.
    pub foot: f32,
}

impl YSort {
    pub const fn from_height(height: f32) -> Self {
        Self {
            z: 0.0,
            foot: height / 2.0,
        }
    }
}

pub struct YSortPlugin;

impl Plugin for YSortPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<YSort>()
            .add_systems(PostUpdate, y_sort.before(TransformSystems::Propagate));
    }
}

/// Works for both top-level entities (the player) and entities parented to an
/// LDtk layer (trees, NPCs) by converting the desired world z into local z.
fn y_sort(
    mut query: Query<(&mut Transform, &YSort, Option<&ChildOf>)>,
    parents: Query<&GlobalTransform>,
) {
    for (mut transform, ysort, child_of) in &mut query {
        let parent = child_of
            .and_then(|c| parents.get(c.parent()).ok())
            .map(GlobalTransform::translation)
            .unwrap_or(Vec3::ZERO);
        let feet_y = parent.y + transform.translation.y - ysort.foot;
        let world_z = Y_SORT_BASE_Z + ysort.z - feet_y * Y_SORT_SCALE;
        transform.translation.z = world_z - parent.z;
    }
}

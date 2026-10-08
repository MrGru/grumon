//! LDtk world loading, level metadata (walls, size) and map props.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    GameState, TILE_SIZE,
    collision::{Collider, cells_in},
    ysort::YSort,
};

/// Level shown when a new game starts.
pub const START_LEVEL: &str = "Village";
/// IntGrid layer holding collision data.
const COLLISION_LAYER: &str = "IntGrid";
/// IntGrid value used for impassable cells.
const WALL_VALUE: i32 = 1;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin)
            .insert_resource(LevelSelection::Identifier(START_LEVEL.to_string()))
            .insert_resource(LdtkSettings {
                // Collision data lives in the IntGrid layer; it should not be drawn.
                int_grid_rendering: IntGridRendering::Invisible,
                ..default()
            })
            .init_resource::<LevelInfo>()
            .add_message::<LevelReady>()
            .register_type::<Warp>()
            .register_ldtk_entity::<TreeBundle>("Tree")
            .register_ldtk_entity::<TreeBundle>("Pine")
            .register_ldtk_entity::<TreeBundle>("SnowTree")
            .register_ldtk_entity::<TreeBundle>("Palm")
            .register_ldtk_entity::<HouseBundle>("House")
            .register_ldtk_entity::<WarpBundle>("Warp")
            .add_systems(OnEnter(GameState::Playing), spawn_world)
            .add_systems(
                PreUpdate,
                update_level_info.run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("LdtkWorld"),
        LdtkWorldBundle {
            ldtk_handle: asset_server.load("world.ldtk").into(),
            ..default()
        },
    ));
}

/// Collision grid and size of the currently loaded level.
///
/// Grid cells use Bevy orientation: `(0, 0)` is the bottom-left cell.
#[derive(Resource, Default, Debug)]
pub struct LevelInfo {
    pub identifier: String,
    /// Size in cells.
    pub grid_size: UVec2,
    walls: Vec<bool>,
}

impl LevelInfo {
    pub fn empty(identifier: &str, grid_size: UVec2) -> Self {
        Self {
            identifier: identifier.to_string(),
            grid_size,
            walls: vec![false; (grid_size.x * grid_size.y) as usize],
        }
    }

    /// Builds the collision grid from a raw LDtk level.
    pub fn from_ldtk(level: &bevy_ecs_ldtk::ldtk::Level) -> Self {
        let mut info = Self::empty(
            &level.identifier,
            (IVec2::new(level.px_wid, level.px_hei).as_vec2() / TILE_SIZE)
                .ceil()
                .as_uvec2(),
        );
        let layer = level
            .layer_instances
            .iter()
            .flatten()
            .find(|layer| layer.identifier == COLLISION_LAYER);
        if let Some(layer) = layer {
            for (i, value) in layer.int_grid_csv.iter().enumerate() {
                let (x, ldtk_y) = (i as i32 % layer.c_wid, i as i32 / layer.c_wid);
                // LDtk rows go top-down; flip to Bevy's bottom-up grid.
                let y = layer.c_hei - 1 - ldtk_y;
                info.set_wall(IVec2::new(x, y), *value == WALL_VALUE);
            }
        } else {
            warn!("level {} has no {COLLISION_LAYER} layer", level.identifier);
        }
        info
    }

    pub fn size_px(&self) -> Vec2 {
        self.grid_size.as_vec2() * TILE_SIZE
    }

    fn index(&self, cell: IVec2) -> Option<usize> {
        let in_bounds = cell.x >= 0
            && cell.y >= 0
            && (cell.x as u32) < self.grid_size.x
            && (cell.y as u32) < self.grid_size.y;
        in_bounds.then(|| (cell.y as u32 * self.grid_size.x + cell.x as u32) as usize)
    }

    pub fn set_wall(&mut self, cell: IVec2, wall: bool) {
        if let Some(i) = self.index(cell) {
            self.walls[i] = wall;
        }
    }

    /// Cells outside the level count as walls.
    pub fn is_wall(&self, cell: IVec2) -> bool {
        self.index(cell).is_none_or(|i| self.walls[i])
    }

    pub fn rect_hits_wall(&self, rect: Rect) -> bool {
        cells_in(rect).any(|cell| self.is_wall(cell))
    }

    /// Converts an LDtk pixel position (y down) into world space.
    pub fn ldtk_to_world(&self, px: IVec2) -> Vec2 {
        Vec2::new(px.x as f32, self.size_px().y - px.y as f32)
    }
}

/// Fired after a level spawned and [`LevelInfo`] describes it.
#[derive(Message, Debug, Clone)]
pub struct LevelReady;

fn update_level_info(
    mut level_events: MessageReader<LevelEvent>,
    mut level_ready: MessageWriter<LevelReady>,
    mut level_info: ResMut<LevelInfo>,
    projects: Query<&LdtkProjectHandle>,
    project_assets: Res<Assets<LdtkProject>>,
) -> Result {
    for event in level_events.read() {
        let LevelEvent::Spawned(level_iid) = event else {
            continue;
        };
        let project = project_assets
            .get(projects.single()?)
            .ok_or("LdtkProject should be loaded when a level spawns")?;
        let level = project
            .get_raw_level_by_iid(level_iid.get())
            .ok_or("spawned level should exist in the project")?;
        *level_info = LevelInfo::from_ldtk(level);
        info!("entered level {}", level_info.identifier);
        level_ready.write(LevelReady);
    }
    Ok(())
}

fn size_of(entity_instance: &EntityInstance) -> Vec2 {
    IVec2::new(entity_instance.width, entity_instance.height).as_vec2()
}

fn ysort_from_instance(entity_instance: &EntityInstance) -> YSort {
    YSort::from_height(entity_instance.height as f32)
}

/// Trees of every kind: only the trunk blocks, so the player can walk behind the canopy.
#[derive(Bundle, LdtkEntity, Default)]
pub struct TreeBundle {
    #[sprite_sheet(no_grid)]
    sprite: Sprite,
    #[with(ysort_from_instance)]
    y_sort: YSort,
    #[with(tree_collider)]
    collider: Collider,
}

fn tree_collider(entity_instance: &EntityInstance) -> Collider {
    Collider::feet(size_of(entity_instance), 14.0, 10.0, 2.0)
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct HouseBundle {
    #[sprite_sheet(no_grid)]
    sprite: Sprite,
    #[with(ysort_from_instance)]
    y_sort: YSort,
    #[with(house_collider)]
    collider: Collider,
}

/// Walls and the lower part of the roof block; the roof's top overlaps whatever is behind it.
fn house_collider(entity_instance: &EntityInstance) -> Collider {
    let size = size_of(entity_instance);
    Collider::feet(size, size.x - 8.0, size.y * 0.6, 2.0)
}

/// Zone that moves the player to another level when stepped on.
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Warp {
    pub to_level: String,
    /// Arrival position of the player's feet in the target level (LDtk pixels, y down).
    pub to: IVec2,
    pub size: Vec2,
}

impl Warp {
    fn from_instance(entity_instance: &EntityInstance) -> Self {
        let to_level = entity_instance
            .get_string_field("to_level")
            .cloned()
            .unwrap_or_else(|_| {
                warn!("Warp {} has no to_level", entity_instance.iid);
                START_LEVEL.to_string()
            });
        let to = IVec2::new(
            *entity_instance.get_int_field("to_x").unwrap_or(&0),
            *entity_instance.get_int_field("to_y").unwrap_or(&0),
        );
        Self {
            to_level,
            to,
            size: size_of(entity_instance),
        }
    }

    pub fn aabb(&self, center: Vec2) -> Rect {
        Rect::from_center_size(center, self.size)
    }
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct WarpBundle {
    #[with(Warp::from_instance)]
    warp: Warp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ldtk_rows_are_flipped() {
        let mut info = LevelInfo::empty("Test", UVec2::new(3, 2));
        info.set_wall(IVec2::new(0, 1), true);
        assert!(info.is_wall(IVec2::new(0, 1)));
        assert!(!info.is_wall(IVec2::new(0, 0)));
        assert!(info.is_wall(IVec2::new(-1, 0)));
        assert!(info.is_wall(IVec2::new(3, 0)));
        assert_eq!(info.ldtk_to_world(IVec2::new(8, 0)), Vec2::new(8.0, 32.0));
    }
}

//! NPCs placed in LDtk (`Npc` entity with an `id` field). Name, sprite,
//! visibility and dialogue come from `npcs` in the data files.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    GameState, animation::Facing, asset::GameAssets, collision::Collider, content::Content,
    story::Progress, ysort::YSort,
};

const NPC_SIZE: Vec2 = Vec2::splat(32.0);

/// An NPC instance; `id` references `NpcDef`.
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Npc {
    pub id: String,
}

impl Npc {
    fn from_instance(entity_instance: &EntityInstance) -> Self {
        let id = entity_instance
            .get_string_field("id")
            .cloned()
            .unwrap_or_else(|_| {
                warn!("Npc {} has no id", entity_instance.iid);
                String::new()
            });
        Self { id }
    }
}

/// Feet collider used while the NPC is visible.
pub fn npc_collider() -> Collider {
    Collider::feet(NPC_SIZE, 18.0, 10.0, 1.0)
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct NpcBundle {
    #[with(Npc::from_instance)]
    npc: Npc,
    #[with(npc_ysort)]
    y_sort: YSort,
}

fn npc_ysort(_: &EntityInstance) -> YSort {
    YSort::from_height(NPC_SIZE.y)
}

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Npc>()
            .register_ldtk_entity::<NpcBundle>("Npc")
            .add_systems(
                Update,
                (setup_npc_sprites, update_npc_visibility)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// LDtk only knows the editor icon; pick the real character sheet here.
fn setup_npc_sprites(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    content: Res<Content>,
    npcs: Query<(Entity, &Npc), Added<Npc>>,
) {
    for (entity, npc) in &npcs {
        let Some(def) = content.db.npcs.get(&npc.id) else {
            error!("unknown npc id `{}`", npc.id);
            continue;
        };
        let mut sprite = game_assets.character_sprite(def.sheet);
        if let Some((r, g, b)) = def.tint {
            sprite.color = Color::srgb(r, g, b);
        }
        commands.entity(entity).insert((
            Name::new(format!("Npc {}", npc.id)),
            sprite,
            Facing::Down,
            Visibility::Hidden,
        ));
    }
}

/// Shows NPCs whose `visible` condition holds; hidden NPCs don't block.
fn update_npc_visibility(
    mut commands: Commands,
    content: Res<Content>,
    progress: Option<Res<Progress>>,
    mut npcs: Query<(Entity, &Npc, &mut Visibility, Has<Collider>)>,
    added: Query<(), Added<Npc>>,
) {
    let Some(progress) = progress else {
        return;
    };
    if !progress.is_changed() && added.is_empty() {
        return;
    }
    for (entity, npc, mut visibility, has_collider) in &mut npcs {
        let visible = content
            .db
            .npcs
            .get(&npc.id)
            .is_some_and(|def| progress.eval_opt(&def.visible));
        let wanted = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
        if visible && !has_collider {
            commands.entity(entity).insert(npc_collider());
        } else if !visible && has_collider {
            commands.entity(entity).remove::<Collider>();
        }
    }
}

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    GameState, PlayState,
    animation::Facing,
    asset::GameAssets,
    collision::Collider,
    dialogue::Dialogue,
    player::{Player, PlayerMovement},
    ysort::YSort,
};

const NPC_SIZE: Vec2 = Vec2::splat(32.0);
/// How far in front of the player's feet we look for someone to talk to.
const TALK_REACH: f32 = 14.0;

/// A character the player can talk to. Data comes from LDtk fields
/// `name`, `sprite` (character sheet number) and `dialogue` (lines).
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Npc {
    pub name: String,
    pub sheet: usize,
    pub lines: Vec<String>,
}

impl Npc {
    fn from_instance(entity_instance: &EntityInstance) -> Self {
        let name = entity_instance
            .get_string_field("name")
            .cloned()
            .unwrap_or_else(|_| "???".to_string());
        let sheet = entity_instance
            .get_int_field("sprite")
            .map(|n| (*n).max(1) as usize)
            .unwrap_or(2);
        let lines: Vec<String> = entity_instance
            .iter_strings_field("dialogue")
            .map(|lines| lines.cloned().collect())
            .unwrap_or_default();
        if lines.is_empty() {
            warn!("NPC {name} has no dialogue lines");
        }
        Self { name, sheet, lines }
    }
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct NpcBundle {
    #[with(Npc::from_instance)]
    npc: Npc,
    #[with(npc_collider)]
    collider: Collider,
    #[with(npc_ysort)]
    y_sort: YSort,
}

fn npc_collider(_: &EntityInstance) -> Collider {
    Collider::feet(NPC_SIZE, 18.0, 10.0, 1.0)
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
                setup_npc_sprites.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                talk_to_npc
                    .after(PlayerMovement)
                    .run_if(in_state(PlayState::Exploring)),
            );
    }
}

/// LDtk only knows the editor icon; pick the real character sheet here.
fn setup_npc_sprites(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    npcs: Query<(Entity, &Npc), Added<Npc>>,
) {
    for (entity, npc) in &npcs {
        commands.entity(entity).insert((
            Name::new(format!("Npc {}", npc.name)),
            game_assets.character_sprite(npc.sheet),
            Facing::Down,
        ));
    }
}

pub fn interact_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.any_just_pressed([KeyCode::Space, KeyCode::Enter, KeyCode::KeyE, KeyCode::KeyZ])
}

fn talk_to_npc(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Query<(&Transform, &Collider, &Facing), With<Player>>,
    mut npcs: Query<(&Npc, &Collider, &GlobalTransform, &mut Facing, &mut Sprite), Without<Player>>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    if !interact_pressed(&keyboard) {
        return;
    }
    let Ok((transform, collider, facing)) = player.single() else {
        return;
    };
    let feet = collider.aabb(transform.translation.truncate());
    let probe = feet.center() + facing.as_vec2() * TALK_REACH;

    let target = npcs
        .iter_mut()
        .filter_map(|(npc, npc_collider, gt, npc_facing, sprite)| {
            let npc_box = npc_collider.aabb(gt.translation().truncate());
            let distance = npc_box.center().distance(probe);
            // Generous reach: anything whose feet box (grown a bit) contains the probe.
            let reachable = npc_box.inflate(6.0).contains(probe);
            reachable.then_some((distance, npc, npc_facing, sprite))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));

    let Some((_, npc, mut npc_facing, mut sprite)) = target else {
        return;
    };
    if npc.lines.is_empty() {
        return;
    }

    // Turn to face the player, like any good JRPG townsfolk.
    *npc_facing = Facing::from_vec2(-facing.as_vec2());
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = npc_facing.first_frame();
    }

    commands.insert_resource(Dialogue::new(npc.name.clone(), npc.lines.clone()));
    next_state.set(PlayState::Dialogue);
}

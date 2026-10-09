//! Map triggers (zones that run effects when entered), interactive objects
//! (herbs, graves, shrines…) and the “talk / examine” action.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    GameState, PlayState,
    animation::Facing,
    asset::GameAssets,
    collision::{Collider, overlaps},
    flow::Story,
    input::MenuInput,
    npc::Npc,
    player::{Player, PlayerMovement},
    ysort::YSort,
};

/// How far in front of the player's feet we look for something to interact with.
const TALK_REACH: f32 = 14.0;

/// Zone from LDtk; `id` references `TriggerDef`.
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Trigger {
    pub id: String,
    pub size: Vec2,
    /// Player was inside last frame (edge-triggered).
    pub inside: bool,
}

/// Interactive object from LDtk; `id` references `ObjectDef`.
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct MapObject {
    pub id: String,
}

fn id_field(entity_instance: &EntityInstance) -> String {
    entity_instance
        .get_string_field("id")
        .cloned()
        .unwrap_or_else(|_| {
            warn!(
                "{} {} has no id",
                entity_instance.identifier, entity_instance.iid
            );
            String::new()
        })
}

impl Trigger {
    fn from_instance(entity_instance: &EntityInstance) -> Self {
        Self {
            id: id_field(entity_instance),
            size: IVec2::new(entity_instance.width, entity_instance.height).as_vec2(),
            inside: false,
        }
    }
}

impl MapObject {
    fn from_instance(entity_instance: &EntityInstance) -> Self {
        Self {
            id: id_field(entity_instance),
        }
    }
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct TriggerBundle {
    #[with(Trigger::from_instance)]
    trigger: Trigger,
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct ObjectBundle {
    #[with(MapObject::from_instance)]
    object: MapObject,
}

pub struct MapEventsPlugin;

impl Plugin for MapEventsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Trigger>()
            .register_type::<MapObject>()
            .register_ldtk_entity::<TriggerBundle>("Trigger")
            .register_ldtk_entity::<ObjectBundle>("Object")
            .add_systems(
                Update,
                (setup_objects, update_objects)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (interact, check_triggers)
                    .chain()
                    .after(PlayerMovement)
                    .run_if(in_state(PlayState::Exploring)),
            );
    }
}

fn setup_objects(
    mut commands: Commands,
    assets: Res<GameAssets>,
    story: Option<Story>,
    objects: Query<(Entity, &MapObject), Added<MapObject>>,
) {
    let Some(story) = story else {
        return;
    };
    for (entity, object) in &objects {
        let Some(def) = story.content.db.objects.get(&object.id) else {
            error!("unknown object id `{}`", object.id);
            continue;
        };
        let mut e = commands.entity(entity);
        e.insert((
            Name::new(format!("Object {}", object.id)),
            Visibility::Hidden,
        ));
        if let Some((x, y, w, h)) = def.sprite {
            let image = match def.sheet {
                crate::content::defs::SpriteSheet::Tileset => assets.tileset.clone(),
                crate::content::defs::SpriteSheet::Objects => assets.objects.clone(),
            };
            e.insert((
                Sprite {
                    image,
                    rect: Some(Rect::new(
                        x as f32,
                        y as f32,
                        (x + w) as f32,
                        (y + h) as f32,
                    )),
                    ..default()
                },
                YSort::from_height(h as f32),
            ));
        }
    }
}

/// Shows objects whose condition holds (and that weren't used up).
fn update_objects(
    mut commands: Commands,
    story: Option<Story>,
    mut objects: Query<(Entity, &MapObject, &mut Visibility, Has<Collider>)>,
    added: Query<(), Added<MapObject>>,
) {
    let Some(story) = story else {
        return;
    };
    if !story.progress.is_changed() && added.is_empty() {
        return;
    }
    for (entity, object, mut visibility, has_collider) in &mut objects {
        let Some(def) = story.content.db.objects.get(&object.id) else {
            continue;
        };
        let active = story.progress.eval_opt(&def.when)
            && !(def.once && story.progress.flag(&def.once_flag()) != 0);
        let wanted = if active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
        let solid = active && def.solid;
        if solid && !has_collider {
            let h = def.sprite.map_or(16, |s| s.3) as f32;
            let w = def.sprite.map_or(16, |s| s.2) as f32;
            commands.entity(entity).insert(Collider::feet(
                Vec2::new(w, h),
                w - 4.0,
                (h * 0.5).max(8.0),
                1.0,
            ));
        } else if !solid && has_collider {
            commands.entity(entity).remove::<Collider>();
        }
    }
}

/// Confirm in front of an NPC or object: talk / examine.
#[allow(clippy::type_complexity)]
fn interact(
    mut input: ResMut<MenuInput>,
    mut story: Story,
    player: Query<(&Transform, &Collider, &Facing), With<Player>>,
    mut npcs: Query<
        (
            &Npc,
            &GlobalTransform,
            &Visibility,
            &mut Facing,
            &mut Sprite,
        ),
        (Without<Player>, Without<MapObject>),
    >,
    objects: Query<(&MapObject, &GlobalTransform, &Visibility), Without<Player>>,
) {
    if !input.confirm || input.consumed {
        return;
    }
    let Ok((transform, collider, facing)) = player.single() else {
        return;
    };
    let feet = collider.aabb(transform.translation.truncate());
    let probe = feet.center() + facing.as_vec2() * TALK_REACH;

    // NPCs: their feet box (grown a bit) must contain the probe.
    let npc_box =
        |gt: &GlobalTransform| crate::npc::npc_collider().aabb(gt.translation().truncate());
    let target = npcs
        .iter_mut()
        .filter(|(_, _, v, _, _)| **v != Visibility::Hidden)
        .filter_map(|(npc, gt, _, f, s)| {
            let b = npc_box(gt);
            b.inflate(6.0)
                .contains(probe)
                .then(|| (b.center().distance(probe), npc, f, s))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));
    if let Some((_, npc, mut npc_facing, mut sprite)) = target {
        let Some(def) = story.content.db.npcs.get(&npc.id) else {
            return;
        };
        let Some(entry) = def
            .talk
            .iter()
            .find(|t| story.progress.eval_opt(&t.when))
            .cloned()
        else {
            return;
        };
        input.consumed = true;
        // Turn to face the player, like any good JRPG townsfolk.
        *npc_facing = Facing::from_vec2(-facing.as_vec2());
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = npc_facing.first_frame();
        }
        story.run(&[crate::content::defs::StoryEffect::Dialogue(entry.dialogue)]);
        return;
    }

    let object = objects
        .iter()
        .filter(|(_, _, v)| **v != Visibility::Hidden)
        .filter_map(|(o, gt, _)| {
            let center = gt.translation().truncate();
            let size = story
                .content
                .db
                .objects
                .get(&o.id)
                .and_then(|d| d.sprite)
                .map_or(Vec2::splat(16.0), |s| Vec2::new(s.2 as f32, s.3 as f32));
            // Objects are interacted with at their base.
            let base = Rect::from_center_size(
                center - Vec2::Y * (size.y / 2.0 - 8.0),
                Vec2::new(size.x, 16.0),
            );
            base.inflate(8.0)
                .contains(probe)
                .then(|| (base.center().distance(probe), o.id.clone()))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));
    if let Some((_, id)) = object
        && let Some(def) = story.content.db.objects.get(&id).cloned()
    {
        input.consumed = true;
        let mut effects = Vec::new();
        if def.once {
            effects.push(crate::content::defs::StoryEffect::SetFlag(
                def.once_flag(),
                1,
            ));
        }
        effects.extend(def.effects.iter().cloned());
        story.run(&effects);
    }
}

fn check_triggers(
    mut story: Story,
    player: Query<(&Transform, &Collider), With<Player>>,
    mut triggers: Query<(&mut Trigger, &GlobalTransform)>,
) {
    let Ok((transform, collider)) = player.single() else {
        return;
    };
    let feet = collider.aabb(transform.translation.truncate());
    for (mut trigger, gt) in &mut triggers {
        let zone = Rect::from_center_size(gt.translation().truncate(), trigger.size);
        let inside = overlaps(zone, feet);
        let entered = inside && !trigger.inside;
        trigger.inside = inside;
        if !entered {
            continue;
        }
        let Some(def) = story.content.db.triggers.get(&trigger.id).cloned() else {
            continue;
        };
        if def.once && story.progress.flag(&def.once_flag()) != 0 {
            continue;
        }
        if !story.progress.eval_opt(&def.when) {
            // Not yet: allow firing when re-checked while still inside.
            trigger.inside = false;
            continue;
        }
        let mut effects = Vec::new();
        if def.once {
            effects.push(crate::content::defs::StoryEffect::SetFlag(
                def.once_flag(),
                1,
            ));
        }
        effects.extend(def.effects.iter().cloned());
        story.run(&effects);
        // One trigger per frame keeps chained events ordered.
        return;
    }
}

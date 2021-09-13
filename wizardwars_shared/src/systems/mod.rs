use crate::components::{
    damage::{Attack, Damage},
    Health, Position, Velocity,
};
use bevy::prelude::*;

pub struct CollisionEvent {
    entity1: Entity,
    entity2: Entity,
}

pub fn apply_damage_system(mut query: Query<(&mut Health, &Damage)>) {
    for (mut health, damage) in query.iter_mut() {
        health.change_by(damage.amount() as i32);
    }
}

pub fn move_system(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut position, velocity) in query.iter_mut() {
        position.0 += velocity.0 * time.delta_seconds()
    }
}

pub fn attack_system(
    mut cmd: Commands,
    mut events: EventReader<CollisionEvent>,
    attacks: Query<&Attack>,
    targets: Query<&Health>,
) {
    for event in events.iter() {
        let mut attacker = None;
        let mut target = None;
        let &CollisionEvent { entity1, entity2 } = event;
        if let Ok(attack) = attacks.get(entity1) {
            attacker = Some((entity1, attack));
        } else if let Ok(health) = targets.get(entity1) {
            target = Some((entity1, health));
        }

        if let Ok(attack) = attacks.get(entity2) {
            attacker = Some((entity2, attack));
        } else if let Ok(health) = targets.get(entity2) {
            target = Some((entity2, health));
        }

        if let Some((e1, attack)) = attacker {
            if let Some((e2, _)) = target {
                cmd.entity(e2).insert(attack.damage());
                cmd.entity(e1).despawn();
            }
        }
    }
}

pub fn collision_system(
    query: Query<(Entity, &Position)>,
    mut events: EventWriter<CollisionEvent>,
) {
    for (entity1, p1) in query.iter() {
        for (entity2, p2) in query.iter() {
            if entity1 == entity2 {
                continue;
            }

            if sphere_intersection(p1.0, p2.0, 1.0) {
                info!("Collision: {:?} - {:?}", entity1, entity2);
                events.send(CollisionEvent { entity1, entity2 });
            }
        }
    }
}

fn sphere_intersection(p1: Vec3, p2: Vec3, r: f32) -> bool {
    (p2 - p1).length_squared() <= r * r
}

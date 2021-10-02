use crate::components::{damage::Damage, Health};
use bevy::prelude::*;

pub fn apply_damage_system(mut cmd: Commands, mut query: Query<(Entity, &mut Health, &Damage)>) {
    for (entity, mut health, damage) in query.iter_mut() {
        health.change_by(-(damage.amount() as i32));
        cmd.entity(entity).remove::<Damage>();
    }
}

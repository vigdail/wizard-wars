use crate::components::{damage::Damage, Health, Position, Velocity};
use bevy::prelude::*;

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

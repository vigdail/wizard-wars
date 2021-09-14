use crate::components::{damage::Damage, Health};
use bevy::prelude::*;

pub fn apply_damage_system(mut query: Query<(&mut Health, &Damage)>) {
    for (mut health, damage) in query.iter_mut() {
        health.change_by(damage.amount() as i32);
    }
}

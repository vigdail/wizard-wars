pub mod damage;
mod health;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use health::Health;

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Uuid(pub u32);

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct Client(pub u32);

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct Player;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct Bot;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct Position(pub Vec3);

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Dead;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Winner;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReadyState {
    Ready,
    NotReady,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub struct Waypoint(pub Vec3);

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Owner(Entity);

impl Owner {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }
    pub fn entity(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct LifeTime {
    pub timer: Timer,
}

impl LifeTime {
    pub fn from_seconds(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, false),
        }
    }
}

mod health;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use health::Health;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
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

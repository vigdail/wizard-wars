use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NetworkId(pub u32);

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct Client(pub u32);

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct Position(pub Vec3);

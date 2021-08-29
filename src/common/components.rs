use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct NetworkId(pub u32);

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct Position(pub Vec3);

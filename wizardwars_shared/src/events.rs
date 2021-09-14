use crate::components::Uuid;
use serde::{Deserialize, Serialize};

pub struct DespawnEntityEvent {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SpawnEvent {
    Projectile(Uuid),
}

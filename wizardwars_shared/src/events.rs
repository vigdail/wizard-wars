use crate::components::{Client, Uuid};
use serde::{Deserialize, Serialize};

pub struct DespawnEntityEvent {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SpawnEvent {
    Projectile(Uuid),
}

pub struct ClientEvent<T> {
    client: Client,
    event: T,
}

impl<T> ClientEvent<T> {
    pub fn new<E: Into<T>>(client: Client, event: E) -> Self {
        Self {
            client,
            event: event.into(),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn event(&self) -> &T {
        &self.event
    }
}

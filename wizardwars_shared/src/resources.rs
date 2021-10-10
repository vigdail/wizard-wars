use crate::{
    components::{Client, Uuid},
    messages::ecs_message::EcsCompPacket,
    network::sync::packet::EntityPackage,
};
use bevy::prelude::*;
use std::slice::Iter;

pub struct CharacterDimensions {
    width: f32,
    height: f32,
}

impl CharacterDimensions {
    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn radius(&self) -> f32 {
        self.width / 2.0
    }

    pub fn half_height(&self) -> f32 {
        self.height / 2.0
    }
}

impl Default for CharacterDimensions {
    fn default() -> Self {
        Self {
            width: 0.5,
            height: 1.8,
        }
    }
}

pub struct ArenaDimensions {
    pub radius: f32,
}

impl Default for ArenaDimensions {
    fn default() -> Self {
        Self { radius: 50.0 }
    }
}

pub const MAX_PLAYERS: usize = 8;

pub struct PlayerColors {
    pub colors: [Color; MAX_PLAYERS],
}

impl Default for PlayerColors {
    fn default() -> Self {
        let colors = [
            Color::RED,
            Color::BLUE,
            Color::GREEN,
            Color::PURPLE,
            Color::CYAN,
            Color::ORANGE,
            Color::OLIVE,
            Color::WHITE,
        ];
        Self { colors }
    }
}

#[derive(Default)]
pub struct IdFactory(u32);

impl IdFactory {
    pub fn generate(&mut self) -> Uuid {
        let id = Uuid(self.0);
        self.0 += 1;

        id
    }
}

#[derive(Default)]
pub struct DespawnedList {
    ids: Vec<Uuid>,
}

impl DespawnedList {
    pub fn push(&mut self, id: Uuid) {
        self.ids.push(id);
    }

    pub fn iter(&self) -> Iter<'_, Uuid> {
        self.ids.iter()
    }

    pub fn clear(&mut self) {
        self.ids.clear();
    }
}

#[derive(Default)]
pub struct WorldSyncList {
    list: Vec<(Client, EntityPackage<EcsCompPacket>)>,
}

impl WorldSyncList {
    pub fn push(&mut self, client: Client, item: EntityPackage<EcsCompPacket>) {
        self.list.push((client, item));
    }

    pub fn iter(&self) -> Iter<'_, (Client, EntityPackage<EcsCompPacket>)> {
        self.list.iter()
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_factory() {
        let mut factory = IdFactory::default();
        let id1 = factory.generate();
        assert_eq!(id1, Uuid(0));

        let id2 = factory.generate();
        assert_eq!(id2, Uuid(1));
    }
}

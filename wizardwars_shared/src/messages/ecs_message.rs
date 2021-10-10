use crate::{
    components::{Player, Position},
    network::sync::packet::{handle_insert, handle_modify, handle_remove, CompPacket},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use sum_type::sum_type;

sum_type! {
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EcsCompPacket {
    Position(Position),
    Player(Player),
}
}

sum_type! {
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EcsCompPacketPhantom {
    Position(PhantomData<Position>),
    Player(PhantomData<Player>),
}
}

// TODO: use some macro to reduce boilerplate code
impl CompPacket for EcsCompPacket {
    type Phantom = EcsCompPacketPhantom;

    fn apply_insert(self, entity: Entity, cmd: &mut Commands) {
        match self {
            EcsCompPacket::Position(pos) => handle_insert(pos, entity, cmd),
            EcsCompPacket::Player(player) => handle_insert(player, entity, cmd),
        }
    }

    fn apply_modify(self, entity: Entity, cmd: &mut Commands) {
        match self {
            EcsCompPacket::Position(pos) => handle_modify(pos, entity, cmd),
            EcsCompPacket::Player(player) => handle_modify(player, entity, cmd),
        }
    }

    fn apply_remove(phantom: Self::Phantom, entity: Entity, cmd: &mut Commands) {
        match phantom {
            Self::Phantom::Position(_) => handle_remove::<Position>(entity, cmd),
            Self::Phantom::Player(_) => handle_remove::<Player>(entity, cmd),
        }
    }
}

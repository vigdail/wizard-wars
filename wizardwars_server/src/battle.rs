use bevy::prelude::*;
use wizardwars_shared::{
    components::{Client, NetworkId, Position},
    messages::ServerMessage,
    network::Dest,
};

use crate::{arena::Arena, network::ServerPacket, states::ServerState};

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(ServerState::Battle).with_system(setup_clients.system()),
        );
    }
}

fn setup_clients(
    mut cmd: Commands,
    arena: Res<Arena>,
    mut packets: EventWriter<ServerPacket>,
    clients: Query<(Entity, &NetworkId, &Client)>,
) {
    let spawn_points = arena.spawn_points();
    clients
        .iter()
        .zip(spawn_points.iter())
        .for_each(|((entity, id, client), point)| {
            cmd.entity(entity).insert(Position(*point));
            packets.send(ServerPacket::new(
                ServerMessage::InsertPlayer(*id, *point),
                Dest::AllExcept(*client),
            ));
            packets.send(ServerPacket::new(
                ServerMessage::InsertLocalPlayer(*id, *point),
                Dest::Single(*client),
            ));
        });
}

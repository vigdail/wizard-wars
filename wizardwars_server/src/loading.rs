use std::collections::HashMap;

use crate::{
    arena::{ArenaBuilder, SpawnPointsBuilder},
    network::ServerPacket,
    states::ServerState,
};
use bevy::prelude::*;
use wizardwars_shared::{
    components::{Client, Uuid},
    messages::server_messages::{LoadingServerMessage, LobbyServerMessage, ServerMessage},
};

pub struct LoadCompleteEvent {
    pub client: Client,
}

struct Loading;

pub struct WaitLoadingPlugin;

impl Plugin for WaitLoadingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<LoadCompleteEvent>()
            .add_system_set(
                SystemSet::on_enter(ServerState::WaitLoading)
                    .with_system(notify_clients.system())
                    .with_system(create_arena.system()),
            )
            .add_system_set(
                SystemSet::on_exit(ServerState::WaitLoading).with_system(on_exit.system()),
            )
            .add_system_set(
                SystemSet::on_update(ServerState::WaitLoading)
                    .with_system(handle_loading_events.system())
                    .with_system(check_players_loading.system()),
            );
    }
}

fn notify_clients(
    mut cmd: Commands,
    mut packets: EventWriter<ServerPacket>,
    clients: Query<Entity, With<Client>>,
) {
    for e in clients.iter() {
        cmd.entity(e).insert(Loading);
    }

    packets.send(ServerPacket::all(ServerMessage::Lobby(
        LobbyServerMessage::StartLoading,
    )));
}

fn create_arena(mut cmd: Commands, clients: Query<Entity, With<Client>>) {
    let clients_count = clients.iter().count() as u32;

    let spawn_points = SpawnPointsBuilder::new()
        .with_circle_points(clients_count, 1.0)
        .build();
    let arena = ArenaBuilder::new().with_spawn_points(spawn_points).build();

    cmd.insert_resource(arena);
}

fn on_exit(mut packets: EventWriter<ServerPacket>) {
    packets.send(ServerPacket::all(LoadingServerMessage::LoadingComplete));
}

fn handle_loading_events(
    mut cmd: Commands,
    clients: Query<(Entity, &Client, &Uuid)>,
    mut loading_events: EventReader<LoadCompleteEvent>,
    mut packets: EventWriter<ServerPacket>,
) {
    let clients_map = clients
        .iter()
        .map(|(entity, client, id)| (client, (entity, id)))
        .collect::<HashMap<_, _>>();
    for event in loading_events.iter() {
        let client = &event.client;

        if let Some(&(entity, &network_id)) = clients_map.get(client) {
            cmd.entity(entity).remove::<Loading>();
            packets.send(ServerPacket::except(
                LoadingServerMessage::PlayerLoaded(network_id),
                *client,
            ));
        }
    }
}

fn check_players_loading(
    clients: Query<Option<&Loading>, With<Client>>,
    mut state: ResMut<State<ServerState>>,
) {
    let all_loaded = clients.iter().all(|loading| loading.is_none());
    if all_loaded {
        state
            .set(ServerState::Shopping)
            .expect("Unable to change state");
    }
}

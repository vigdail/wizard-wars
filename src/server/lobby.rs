use std::collections::HashMap;

use crate::common::{
    components::{Client, NetworkId},
    messages::{LobbyServerMessage, PlayerReadyState, ServerMessage},
    network::{Dest, Pack},
};
use bevy::prelude::*;

use super::{
    network::{CurrentId, Host, ServerPacket},
    states::ServerState,
};

pub struct LobbyEvent {
    client: Client,
    event: LobbyEventEntry,
}

impl LobbyEvent {
    pub fn new(client: Client, event: LobbyEventEntry) -> Self {
        Self { client, event }
    }
}

#[allow(dead_code)]
pub enum LobbyEventEntry {
    ClientJoined(String),
    ReadyChanged(PlayerReadyState),
    StartGame,
}

pub struct StartGameEvent;

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<LobbyEvent>()
            .add_event::<StartGameEvent>()
            .add_system_set(
                SystemSet::on_update(ServerState::Lobby)
                    .with_system(handle_lobby_events.system())
                    .with_system(handle_start_game.system()),
            );
    }
}

fn handle_lobby_events(
    mut cmd: Commands,
    mut lobby_evets: EventReader<LobbyEvent>,
    mut start_game_events: EventWriter<StartGameEvent>,
    mut host: ResMut<Host>,
    mut ids: ResMut<CurrentId>,
    mut packets: EventWriter<ServerPacket>,
    clients: Query<(Entity, &Client, &NetworkId)>,
) {
    let clients_map = clients
        .iter()
        .map(|(e, client, id)| (client, (e, id)))
        .collect::<HashMap<_, _>>();
    for event in lobby_evets.iter() {
        let client = event.client;
        match &event.event {
            LobbyEventEntry::ClientJoined(name) => {
                let network_id = NetworkId(ids.0);
                ids.0 += 1;

                if host.0.is_none() {
                    host.0 = Some(network_id);
                }

                let client_name = Name::new(name.clone());

                cmd.spawn()
                    .insert(client)
                    .insert(client_name.clone())
                    .insert(network_id);

                packets.send(Pack::new(
                    ServerMessage::LobbyMessage(LobbyServerMessage::Welcome(network_id)),
                    Dest::Single(client),
                ));
                packets.send(Pack::new(
                    ServerMessage::LobbyMessage(LobbyServerMessage::SetHost(host.0.unwrap())),
                    Dest::Single(client),
                ));

                packets.send(Pack::new(
                    ServerMessage::LobbyMessage(LobbyServerMessage::PlayerJoined(
                        client_name.as_str().to_owned(),
                    )),
                    Dest::AllExcept(client),
                ));
            }
            LobbyEventEntry::ReadyChanged(_ready) => todo!(),
            LobbyEventEntry::StartGame => {
                if host.0.is_some() && host.0 == clients_map.get(&client).map(|(_, &id)| id) {
                    // TODO: check if all players are ready
                    start_game_events.send(StartGameEvent);
                } else {
                    error!(
                        "The client ({:?}) is not a host (cannot start game)",
                        client
                    );
                }
            }
        }
    }
}

fn handle_start_game(
    mut start_game_events: EventReader<StartGameEvent>,
    mut app_state: ResMut<State<ServerState>>,
) {
    if start_game_events.iter().next().is_some() {
        app_state
            .set(ServerState::WaitLoading)
            .expect("Cannot change state");
    }
}

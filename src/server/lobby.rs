use super::{
    network::{CurrentId, Host, ServerPacket},
    states::ServerState,
};
use crate::common::{
    components::{Client, NetworkId},
    messages::{LobbyServerMessage, ReadyState, ServerMessage},
    network::{Dest, Pack},
};
use bevy::prelude::*;

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
    ReadyChanged(ReadyState),
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
    mut clients: Query<(Entity, &Client, &NetworkId, &mut ReadyState)>,
) {
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
                    .insert(ReadyState::NotReady)
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
            LobbyEventEntry::ReadyChanged(ready) => {
                for (_, &c, _, mut ready_state) in clients.iter_mut() {
                    if c == client {
                        *ready_state = *ready;
                        info!("{:?} ready state is {:?}", client, ready_state);
                    }
                }

                let can_start = clients
                    .iter_mut()
                    .all(|(_, _, _, ready_state)| *ready_state == ReadyState::Ready);

                let lobby_ready_state = if can_start {
                    ReadyState::Ready
                } else {
                    ReadyState::NotReady
                };

                packets.send(Pack::new(
                    ServerMessage::LobbyMessage(LobbyServerMessage::ReadyState(lobby_ready_state)),
                    Dest::All,
                ));
            }
            LobbyEventEntry::StartGame => {
                let client_id =
                    clients.iter_mut().find_map(
                        |(_, &c, id, _)| {
                            if c == client {
                                Some(*id)
                            } else {
                                None
                            }
                        },
                    );
                if host.0.is_some() && host.0 == client_id {
                    let can_start = clients
                        .iter_mut()
                        .all(|(_, _, _, ready_state)| *ready_state == ReadyState::Ready);

                    if can_start {
                        start_game_events.send(StartGameEvent);
                    } else {
                        error!("Cannot start game: some clients are not ready");
                    }
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

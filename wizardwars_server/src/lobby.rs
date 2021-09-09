use super::{
    network::{Host, IdFactory, ServerPacket},
    states::ServerState,
};
use bevy::{prelude::*, utils::HashMap};
use wizardwars_shared::{
    components::{Client, NetworkId},
    messages::{
        server_messages::{LobbyServerMessage, ServerMessage},
        ReadyState,
    },
    network::Pack,
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

pub enum LobbyEventEntry {
    ClientJoined(String),
    ReadyChanged(ReadyState),
    StartGame,
}

#[derive(PartialEq)]
pub struct LobbyReadyState(ReadyState);

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<LobbyEvent>()
            .add_system_set(
                SystemSet::on_enter(ServerState::Lobby).with_system(setup_lobby.system()),
            )
            .add_system_set(
                SystemSet::on_exit(ServerState::Lobby).with_system(teardown_lobby.system()),
            )
            .add_system_set(
                SystemSet::on_update(ServerState::Lobby)
                    .with_system(handle_client_joined.system())
                    .with_system(handle_client_ready_events.system())
                    .with_system(handle_ready_changed.system())
                    .with_system(handle_start_game_event.system()),
            );
    }
}

fn setup_lobby(mut cmd: Commands) {
    cmd.insert_resource(LobbyReadyState(ReadyState::NotReady));
}

fn teardown_lobby(mut cmd: Commands) {
    cmd.remove_resource::<LobbyReadyState>();
}

fn handle_client_joined(
    mut cmd: Commands,
    mut lobby_evets: EventReader<LobbyEvent>,
    mut host: ResMut<Host>,
    mut id_factory: ResMut<IdFactory>,
    mut packets: EventWriter<ServerPacket>,
    clients: Query<(&NetworkId, &Name), With<Client>>,
) {
    for event in lobby_evets.iter() {
        let client = event.client;
        if let LobbyEventEntry::ClientJoined(name) = &event.event {
            let network_id = id_factory.generate();

            if host.0.is_none() {
                host.0 = Some(network_id);
            }

            let client_name = Name::new(name.clone());

            cmd.spawn()
                .insert(client)
                .insert(client_name.clone())
                .insert(ReadyState::NotReady)
                .insert(network_id);

            packets.send(Pack::single(
                LobbyServerMessage::Welcome(network_id),
                client,
            ));
            packets.send(Pack::single(
                LobbyServerMessage::SetHost(host.0.unwrap()),
                client,
            ));
            for (&id, name) in clients.iter() {
                packets.send(Pack::single(
                    LobbyServerMessage::PlayerJoined(id, name.to_string()),
                    client,
                ));
            }

            packets.send(Pack::except(
                ServerMessage::Lobby(LobbyServerMessage::PlayerJoined(
                    network_id,
                    client_name.as_str().to_owned(),
                )),
                client,
            ));
        }
    }
}

fn handle_client_ready_events(
    mut cmd: Commands,
    mut lobby_evets: EventReader<LobbyEvent>,
    clients: Query<(Entity, &Client)>,
) {
    let clients_map = clients
        .iter()
        .map(|(entity, client)| (client, entity))
        .collect::<HashMap<_, _>>();
    for event in lobby_evets.iter() {
        let client = &event.client;
        if let LobbyEventEntry::ReadyChanged(ready) = &event.event {
            if let Some(&entity) = clients_map.get(client) {
                cmd.entity(entity).insert(*ready);
            }
        }
    }
}

fn handle_ready_changed(
    mut cmd: Commands,
    mut packets: EventWriter<ServerPacket>,
    changed_clients: Query<Entity, (With<Client>, Changed<ReadyState>)>,
    all_clients: Query<(Entity, &ReadyState), With<Client>>,
) {
    // TODO: This should be turned into run criteria
    if changed_clients.iter().next().is_none() {
        return;
    }

    let all_ready = all_clients
        .iter()
        .all(|(_, ready_state)| *ready_state == ReadyState::Ready);
    let clients_count = all_clients.iter().count();

    let lobby_ready_state = if all_ready && clients_count > 1 {
        ReadyState::Ready
    } else {
        ReadyState::NotReady
    };

    cmd.insert_resource(LobbyReadyState(lobby_ready_state));

    packets.send(Pack::all(ServerMessage::Lobby(
        LobbyServerMessage::ReadyState(lobby_ready_state),
    )));
}

fn handle_start_game_event(
    mut app_state: ResMut<State<ServerState>>,
    mut lobby_evets: EventReader<LobbyEvent>,
    host: Res<Host>,
    lobby_ready_state: Res<LobbyReadyState>,
    clients: Query<(&Client, &NetworkId)>,
) {
    let clients_map = clients
        .iter()
        .map(|(client, id)| (client, id))
        .collect::<HashMap<_, _>>();
    for event in lobby_evets.iter() {
        let client = &event.client;
        if let LobbyEventEntry::StartGame = &event.event {
            if let Some(&client_id) = clients_map.get(client) {
                if host.is_host(client_id) {
                    if *lobby_ready_state == LobbyReadyState(ReadyState::Ready) {
                        app_state
                            .set(ServerState::WaitLoading)
                            .expect("Cannot change state");
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

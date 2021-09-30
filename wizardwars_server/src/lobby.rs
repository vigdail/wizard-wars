use super::{
    network::{Host, IdFactory, ServerPacket},
    states::ServerState,
};
use bevy::{prelude::*, utils::HashMap};
use wizardwars_shared::{
    components::{Bot, Client, Player, ReadyState, Uuid},
    events::ClientEvent,
    messages::server_messages::{LobbyServerMessage, ServerMessage},
    network::Pack,
};

pub type LobbyEvent = ClientEvent<LobbyEventEntry>;

pub enum LobbyEventEntry {
    ClientJoined(String),
    ReadyChanged(ReadyState),
    CreateBot,
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
                    .with_system(handle_create_bot.system())
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
    clients: Query<(&Uuid, &Name), With<Client>>,
) {
    for event in lobby_evets.iter() {
        let client = *event.client();
        if let LobbyEventEntry::ClientJoined(name) = event.event() {
            let network_id = id_factory.generate();

            if host.0.is_none() {
                host.0 = Some(network_id);
            }

            let client_name = Name::new(name.clone());

            cmd.spawn()
                .insert(client)
                .insert(Player)
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

fn handle_create_bot(
    mut cmd: Commands,
    mut lobby_evets: EventReader<LobbyEvent>,
    mut host: ResMut<Host>,
    mut id_factory: ResMut<IdFactory>,
    mut packets: EventWriter<ServerPacket>,
) {
    for event in lobby_evets.iter() {
        if let LobbyEventEntry::CreateBot = event.event() {
            let network_id = id_factory.generate();

            if host.0.is_none() {
                host.0 = Some(network_id);
            }

            let name = Name::new("BOT");

            cmd.spawn()
                .insert(Player)
                .insert(Bot)
                .insert(name.clone())
                .insert(ReadyState::Ready)
                .insert(network_id);

            packets.send(Pack::all(ServerMessage::Lobby(
                LobbyServerMessage::PlayerJoined(network_id, name.as_str().to_owned()),
            )));
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
        let client = event.client();
        if let LobbyEventEntry::ReadyChanged(ready) = event.event() {
            if let Some(&entity) = clients_map.get(client) {
                cmd.entity(entity).insert(*ready);
            }
        }
    }
}

fn handle_ready_changed(
    mut cmd: Commands,
    mut packets: EventWriter<ServerPacket>,
    changed_players: Query<Entity, (With<Player>, Changed<ReadyState>)>,
    all_players: Query<(Entity, &ReadyState), With<Player>>,
) {
    // TODO: This should be turned into run criteria
    if changed_players.iter().next().is_none() {
        return;
    }

    let all_ready = all_players
        .iter()
        .all(|(_, ready_state)| *ready_state == ReadyState::Ready);
    let clients_count = all_players.iter().count();

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
    lobby_ready_state: Res<LobbyReadyState>,
) {
    for event in lobby_evets.iter() {
        if let LobbyEventEntry::StartGame = event.event() {
            if *lobby_ready_state == LobbyReadyState(ReadyState::Ready) {
                app_state
                    .set(ServerState::WaitLoading)
                    .expect("Cannot change state");
            } else {
                error!("Cannot start game: some clients are not ready");
            }
        }
    }
}

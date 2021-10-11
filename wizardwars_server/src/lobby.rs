use super::{network::ServerPacket, states::ServerState};
use bevy::{prelude::*, utils::HashMap};
use wizardwars_shared::{
    components::{Bot, Client, Host, Player, ReadyState},
    events::ClientEvent,
    messages::{
        client_messages::LobbyClientMessage,
        server_messages::{LobbyServerMessage, RejectReason, ServerMessage},
    },
    network::{
        sync::{CommandsSync, EntityCommandsSync},
        Pack,
    },
    resources::MAX_PLAYERS,
};

pub type LobbyEvent = ClientEvent<LobbyClientMessage>;

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
    mut packets: EventWriter<ServerPacket>,
    clients: Query<&Name, With<Client>>,
    hosts: Query<Option<&Host>>,
    players: Query<&Player>,
) {
    let mut players_count = players.iter().count();
    let mut has_host = hosts.iter().next().is_some();
    for event in lobby_evets.iter() {
        let client = *event.client();
        if let LobbyClientMessage::Join(name) = event.event() {
            if players_count >= MAX_PLAYERS {
                warn!("Max players reached");
                packets.send(Pack::single(
                    LobbyServerMessage::Reject {
                        reason: RejectReason::LobbyFull,
                        disconnect: true,
                    },
                    client,
                ));
                continue;
            }
            players_count += 1;

            let client_name = Name::new(name.clone());

            let mut builder = cmd.spawn_sync();
            builder
                .insert(client)
                .insert(Player)
                .insert(client_name.clone())
                .insert(ReadyState::NotReady);

            if !has_host {
                builder.insert(Host);
                has_host = true;
            }

            builder.sync_world();

            // packets.send(Pack::single(
            //     LobbyServerMessage::Welcome(WorldSync {}),
            //     client,
            // ));
            for name in clients.iter() {
                packets.send(Pack::single(
                    LobbyServerMessage::PlayerJoined(name.to_string()),
                    client,
                ));
            }

            packets.send(Pack::except(
                LobbyServerMessage::PlayerJoined(client_name.as_str().to_owned()),
                client,
            ));
        }
    }
}

fn handle_create_bot(
    mut cmd: Commands,
    mut lobby_evets: EventReader<LobbyEvent>,
    mut packets: EventWriter<ServerPacket>,
    host: Query<&Client, With<Host>>,
    players: Query<&Player>,
) {
    let mut players_count = players.iter().count();
    for event in lobby_evets.iter() {
        if let LobbyClientMessage::AddBot = event.event() {
            if players_count >= MAX_PLAYERS {
                warn!("Cannot add a bot, lobby is full");
                if let Some(host_client) = host.iter().next() {
                    packets.send(Pack::single(
                        LobbyServerMessage::Reject {
                            reason: RejectReason::LobbyFull,
                            disconnect: false,
                        },
                        *host_client,
                    ));
                }
                continue;
            }
            players_count += 1;

            let name = Name::new("BOT");

            cmd.spawn_sync()
                .insert(Player)
                .insert(Bot)
                .insert(name.clone())
                .insert(ReadyState::Ready);

            packets.send(Pack::all(ServerMessage::Lobby(
                LobbyServerMessage::PlayerJoined(name.as_str().to_owned()),
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
        if let LobbyClientMessage::ChangeReadyState(ready) = event.event() {
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
        if let LobbyClientMessage::StartGame = event.event() {
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

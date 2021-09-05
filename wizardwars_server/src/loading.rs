use bevy::prelude::*;
use wizardwars_shared::{
    components::{Client, NetworkId},
    messages::{LoadingServerMessage, LobbyServerMessage, ServerMessage},
    network::Dest,
};

use crate::{network::ServerPacket, states::ServerState};

pub struct LoadCompleteEvent {
    client: Client,
}

struct Loading;

pub struct WaitLoadingPlugin;

impl Plugin for WaitLoadingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<LoadCompleteEvent>()
            .add_system_set(
                SystemSet::on_enter(ServerState::WaitLoading).with_system(on_enter.system()),
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

fn on_enter(
    mut cmd: Commands,
    mut packets: EventWriter<ServerPacket>,
    clients: Query<Entity, With<Client>>,
) {
    for e in clients.iter() {
        cmd.entity(e).insert(Loading);
    }

    packets.send(ServerPacket::new(
        ServerMessage::Lobby(LobbyServerMessage::StartLoading),
        Dest::All,
    ));
}

fn on_exit(mut packets: EventWriter<ServerPacket>) {
    packets.send(ServerPacket::new(
        ServerMessage::Loading(LoadingServerMessage::LoadingComplete),
        Dest::All,
    ));
}

fn handle_loading_events(
    mut cmd: Commands,
    clients: Query<(Entity, &Client)>,
    ids: Query<&NetworkId>,
    mut loading_events: EventReader<LoadCompleteEvent>,
    mut packets: EventWriter<ServerPacket>,
) {
    for event in loading_events.iter() {
        let event_client = &event.client;
        for (entity, client) in clients.iter() {
            if client != event_client {
                continue;
            }

            cmd.entity(entity).remove::<Loading>();
            match ids.get(entity) {
                Ok(&network_id) => {
                    packets.send(ServerPacket::new(
                        ServerMessage::Loading(LoadingServerMessage::PlayerLoaded(network_id)),
                        Dest::AllExcept(*client),
                    ));
                }
                Err(e) => error!("{}", e),
            }
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

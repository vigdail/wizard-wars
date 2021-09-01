use std::collections::HashMap;

use crate::common::{
    components::{Client, NetworkId},
    messages::PlayerReadyState,
};
use bevy::prelude::*;

use super::{network::Host, states::ServerState};

#[allow(dead_code)]
pub enum LobbyEvent {
    ClientJoined(Client, String),
    ReadyChanged(Client, PlayerReadyState),
    StartGame(Client),
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
    mut _cmd: Commands,
    mut lobby_evets: EventReader<LobbyEvent>,
    mut start_game_events: EventWriter<StartGameEvent>,
    host: Res<Host>,
    clients: Query<(Entity, &Client, &NetworkId)>,
) {
    let clients_map = clients
        .iter()
        .map(|(e, client, id)| (client, (e, id)))
        .collect::<HashMap<_, _>>();
    for event in lobby_evets.iter() {
        match event {
            LobbyEvent::ClientJoined(_client, _name) => {
                // create client entity, etc.
            }
            LobbyEvent::ReadyChanged(_client, _ready) => todo!(),
            LobbyEvent::StartGame(client) => {
                if host.0.is_some() && host.0 == clients_map.get(client).map(|(_, &id)| id) {
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

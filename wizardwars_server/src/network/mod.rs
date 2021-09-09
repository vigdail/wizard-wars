use crate::loading::LoadCompleteEvent;
use crate::lobby::{LobbyEvent, LobbyEventEntry};
use crate::states::ServerState;
use crate::ActionEvent;
use bevy::prelude::*;
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use wizardwars_shared::components::{Client, NetworkId, Position};
use wizardwars_shared::messages::client_messages::{
    ActionMessage, ClientMessage, LobbyClientMessage,
};
use wizardwars_shared::messages::{network_channels_setup, server_messages::ServerMessage};
use wizardwars_shared::network::{Dest, Pack};

#[derive(Default)]
pub struct IdFactory(u32);

impl IdFactory {
    pub fn generate(&mut self) -> NetworkId {
        let id = NetworkId(self.0);
        self.0 += 1;

        id
    }
}

#[derive(Default)]
pub struct Host(pub Option<NetworkId>);

impl Host {
    pub fn is_host(&self, id: &NetworkId) -> bool {
        Some(*id) == self.0
    }
}

pub struct NetworkPlugin;

pub type ServerPacket = Pack<ServerMessage>;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(IdFactory::default())
            .insert_resource(Host::default())
            .add_event::<ServerPacket>()
            .add_plugin(NetworkingPlugin::default())
            .add_system_set(
                SystemSet::on_enter(ServerState::Init)
                    .with_system(network_channels_setup.system())
                    .with_system(server_setup_system.system()),
            )
            .add_system(handle_network_events_system.system())
            .add_system(read_network_channels_system.system())
            .add_system(send_packets_system.system())
            .add_system(broadcast_changes_system.system());
    }
}

pub fn server_setup_system(mut net: ResMut<NetworkResource>) {
    let ip_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket_address = SocketAddr::new(ip_address, 9001);
    net.listen(socket_address, None, None);
    info!("Listening...");
}

fn handle_network_events_system(
    mut net: ResMut<NetworkResource>,
    mut network_event_reader: EventReader<NetworkEvent>,
) {
    for event in network_event_reader.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(_connection) => {
                    info!("New connection handle: {:?}", &handle);
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            NetworkEvent::Disconnected(handle) => {
                info!("Disconnected handle: {:?}", &handle);
            }
            _ => {}
        }
    }
}

fn read_network_channels_system(
    mut net: ResMut<NetworkResource>,
    mut action_events: EventWriter<ActionEvent>,
    mut lobby_events: EventWriter<LobbyEvent>,
    mut loading_events: EventWriter<LoadCompleteEvent>,
) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        let client = Client(*handle);
        while let Some(message) = channels.recv::<ClientMessage>() {
            match message {
                ClientMessage::LobbyMessage(msg) => match msg {
                    LobbyClientMessage::Join(name) => lobby_events
                        .send(LobbyEvent::new(client, LobbyEventEntry::ClientJoined(name))),
                    LobbyClientMessage::ChangeReadyState(ready) => lobby_events.send(
                        LobbyEvent::new(client, LobbyEventEntry::ReadyChanged(ready)),
                    ),
                    LobbyClientMessage::GetPlayerList => todo!(),
                    LobbyClientMessage::StartGame => {
                        lobby_events.send(LobbyEvent::new(client, LobbyEventEntry::StartGame));
                    }
                },
                ClientMessage::Action(e) => match e {
                    ActionMessage::Move(dir) => {
                        action_events.send(ActionEvent::Move(client, dir));
                    }
                    ActionMessage::Attack { target } => {
                        action_events.send(ActionEvent::Attack(client, target));
                    }
                },
                ClientMessage::Loaded => {
                    loading_events.send(LoadCompleteEvent { client });
                }
            }
        }
    }
}

fn send_packets_system(
    mut net: ResMut<NetworkResource>,
    mut events: EventReader<ServerPacket>,
    clients: Query<&Client>,
) {
    for pack in events.iter() {
        match &pack.dest {
            Dest::Single(client) => {
                net.send_message(client.0, pack.msg.clone())
                    .expect("Unable to send message");
            }
            Dest::AllExcept(exclude_client) => {
                for &client in clients.iter() {
                    if exclude_client != &client {
                        net.send_message(client.0, pack.msg.clone())
                            .expect("Unable to send message");
                    }
                }
            }
            Dest::All => net.broadcast_message(pack.msg.clone()),
        }
    }
}

fn broadcast_changes_system(
    mut net: ResMut<NetworkResource>,
    changed_positions: Query<(&NetworkId, &Position), Changed<Position>>,
) {
    for (id, position) in changed_positions.iter() {
        let _ = net.broadcast_message((*id, *position));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_factory() {
        let mut factory = IdFactory::default();
        let id1 = factory.generate();
        assert_eq!(id1, NetworkId(0));

        let id2 = factory.generate();
        assert_eq!(id2, NetworkId(1));
    }
}

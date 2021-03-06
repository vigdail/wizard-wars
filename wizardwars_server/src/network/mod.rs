use crate::loading::LoadCompleteEvent;
use crate::lobby::LobbyEvent;
use crate::states::ServerState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use wizardwars_shared::components::{Client, Position, Uuid};
use wizardwars_shared::events::ClientEvent;
use wizardwars_shared::messages::client_messages::{ActionMessage, ClientMessage, Verify};
use wizardwars_shared::messages::server_messages::LobbyServerMessage;
use wizardwars_shared::messages::{network_channels_setup, server_messages::ServerMessage};
use wizardwars_shared::network::{Dest, Pack};

#[derive(Default)]
pub struct IdFactory(u32);

impl IdFactory {
    pub fn generate(&mut self) -> Uuid {
        let id = Uuid(self.0);
        self.0 += 1;

        id
    }
}

#[derive(Default)]
pub struct Host(pub Option<Uuid>);

impl Host {
    pub fn set_host(&mut self, id: Option<Uuid>) {
        self.0 = id;
    }

    pub fn is_host(&self, id: &Uuid) -> bool {
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
            .add_plugin(NetworkingPlugin {
                idle_timeout_ms: Some(3000),
                auto_heartbeat_ms: Some(1000),
                ..Default::default()
            })
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
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
    mut network_event_reader: EventReader<NetworkEvent>,
    mut host: ResMut<Host>,
    clients: Query<(Entity, &Client, &Uuid)>,
) {
    let clients_map = clients
        .iter()
        .map(|(entity, client, id)| (client.0, (entity, id)))
        .collect::<HashMap<_, _>>();
    let mut disconnected = Vec::new();

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
                if let Some(&(entity, &id)) = clients_map.get(handle) {
                    cmd.entity(entity).despawn();
                    disconnected.push(id);
                }
            }
            _ => {}
        }
    }
    for id in disconnected.into_iter() {
        net.broadcast_message(ServerMessage::Despawn(id));

        if host.is_host(&id) {
            let new_host_id = clients_map.iter().find_map(|(_, (_, &client_id))| {
                if client_id != id {
                    Some(client_id)
                } else {
                    None
                }
            });
            host.set_host(new_host_id);

            if let Some(host_id) = new_host_id {
                net.broadcast_message(ServerMessage::Lobby(LobbyServerMessage::SetHost(host_id)));
            }
        }
    }
}

fn read_network_channels_system(
    mut net: ResMut<NetworkResource>,
    mut action_events: EventWriter<ClientEvent<ActionMessage>>,
    mut lobby_events: EventWriter<LobbyEvent>,
    mut loading_events: EventWriter<LoadCompleteEvent>,
    host: Res<Host>,
    query: Query<(&Client, &Uuid)>,
) {
    let client_map = query
        .iter()
        .map(|(client, &id)| (client, id))
        .collect::<HashMap<_, _>>();

    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        let client = Client(*handle);
        let client_id = client_map.get(&client);
        while let Some(message) = channels.recv::<ClientMessage>() {
            let is_host = client_id.map(|id| host.is_host(id)).unwrap_or(false);
            if !message.verify(is_host) {
                error!("Only host can use: {:?}", message);
                continue;
            }

            match message {
                ClientMessage::LobbyMessage(msg) => {
                    lobby_events.send(ClientEvent::new(client, msg))
                }
                ClientMessage::Action(msg) => action_events.send(ClientEvent::new(client, msg)),
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
    changed_positions: Query<(&Uuid, &Position), Changed<Position>>,
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
        assert_eq!(id1, Uuid(0));

        let id2 = factory.generate();
        assert_eq!(id2, Uuid(1));
    }
}

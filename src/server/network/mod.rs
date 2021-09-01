use crate::common::components::{Client, NetworkId, Position};
use crate::common::messages::{
    network_channels_setup, ActionMessage, ClientMessage, LobbyClientMessage, LobbyServerMessage,
    ServerMessage,
};
use crate::common::network::{Dest, Pack};
use crate::server::states::ServerState;
use crate::server::InputEvent;
use bevy::prelude::*;
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Default)]
pub struct CurrentId(u32);

#[derive(Default)]
pub struct Host(pub Option<NetworkId>);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(CurrentId::default())
            .insert_resource(Host::default())
            .add_plugin(NetworkingPlugin::default())
            .add_system_set(
                SystemSet::on_enter(ServerState::Init)
                    .with_system(network_channels_setup.system())
                    .with_system(server_setup_system.system()),
            )
            .add_system(handle_network_events_system.system())
            .add_system(read_network_channels_system.system())
            .add_system(broadcast_changes_system.system());
    }
}

pub fn server_setup_system(mut net: ResMut<NetworkResource>) {
    let ip_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket_address = SocketAddr::new(ip_address, 9001);
    net.listen(socket_address, None, None);
    println!("Listening...");
}

fn handle_network_events_system(
    mut net: ResMut<NetworkResource>,
    mut network_event_reader: EventReader<NetworkEvent>,
) {
    for event in network_event_reader.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(_connection) => {
                    println!("New connection handle: {:?}", &handle);
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            NetworkEvent::Disconnected(handle) => {
                println!("Disconnected handle: {:?}", &handle);
            }
            _ => {}
        }
    }
}

fn read_network_channels_system(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
    mut host: ResMut<Host>,
    mut input_events: EventWriter<InputEvent>,
    mut ids: ResMut<CurrentId>,
    clients: Query<&Client>,
) {
    let mut packs = Vec::<Pack<ServerMessage>>::new();

    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(message) = channels.recv::<ClientMessage>() {
            match message {
                ClientMessage::LobbyMessage(msg) => match msg {
                    LobbyClientMessage::Join(name) => {
                        let network_id = NetworkId(ids.0);
                        ids.0 += 1;

                        if host.0.is_none() {
                            host.0 = Some(network_id);
                        }

                        let client = Client(*handle);
                        let client_name = Name::new(name);

                        cmd.spawn()
                            .insert(client)
                            .insert(client_name.clone())
                            .insert(network_id);

                        packs.push(Pack::new(
                            ServerMessage::LobbyMessage(LobbyServerMessage::Welcome(network_id)),
                            Dest::Single(client),
                        ));
                        packs.push(Pack::new(
                            ServerMessage::LobbyMessage(LobbyServerMessage::SetHost(
                                host.0.unwrap(),
                            )),
                            Dest::Single(client),
                        ));

                        packs.push(Pack::new(
                            ServerMessage::LobbyMessage(LobbyServerMessage::PlayerJoined(
                                client_name.as_str().to_owned(),
                            )),
                            Dest::AllExcept(client),
                        ));
                    }
                    LobbyClientMessage::ChangeReadyState(_) => todo!(),
                    LobbyClientMessage::GetPlayerList => todo!(),
                    LobbyClientMessage::StartGame => todo!(),
                },
                ClientMessage::Action(e) => match e {
                    ActionMessage::Move(dir) => {
                        input_events.send(InputEvent::Move(Client(*handle), dir));
                    }
                },
                ClientMessage::Loaded => {}
            }
        }
    }

    for pack in packs.drain(..) {
        match pack.dest {
            Dest::Single(client) => {
                net.send_message(client.0, pack.msg)
                    .expect("Unable to send message");
            }
            Dest::AllExcept(exclude_client) => {
                for &client in clients.iter() {
                    if exclude_client != client {
                        net.send_message(client.0, pack.msg.clone())
                            .expect("Unable to send message");
                    }
                }
            }
            Dest::All => {
                for client in clients.iter() {
                    net.send_message(client.0, pack.msg.clone())
                        .expect("Unable to send message");
                }
            }
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

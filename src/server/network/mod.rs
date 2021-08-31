use crate::common::components::{NetworkId, Position};
use crate::common::{network_channels_setup, ActionMessage, ClientMessage, ServerMessage};
use crate::server::states::ServerState;
use crate::server::InputEvent;
use bevy::prelude::*;
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub struct NetworkHandle(u32);

#[derive(Default)]
pub struct CurrentId(u32);

#[derive(Default)]
pub struct Host(Option<NetworkId>);

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
    clients: Query<(Entity, &NetworkHandle, &Name)>,
) {
    let mut to_send = Vec::new();
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(message) = channels.recv::<ClientMessage>() {
            match message {
                ClientMessage::Hello(name) => {
                    let network_id = NetworkId(ids.0);
                    ids.0 += 1;

                    if host.0.is_none() {
                        host.0 = Some(network_id);
                    }

                    cmd.spawn()
                        .insert(NetworkHandle(*handle))
                        .insert(Name::new(name))
                        .insert(network_id);

                    to_send.push((*handle, ServerMessage::Welcome(network_id)));
                    to_send.push((*handle, ServerMessage::SetHost(host.0.unwrap())));

                    for (_, client, client_name) in clients.iter() {
                        to_send.push((
                            client.0,
                            ServerMessage::PlayerJoined(client_name.as_str().to_owned()),
                        ));
                    }
                }
                ClientMessage::Action(e) => match e {
                    ActionMessage::Move(dir) => {
                        input_events.send(InputEvent::Move(NetworkHandle(*handle), dir));
                    }
                },
                ClientMessage::StartGame => {}
                ClientMessage::Loaded => {}
            }
        }
    }
    for (handle, msg) in to_send.drain(..) {
        net.send_message(handle, msg.clone())
            .unwrap_or_else(|_| panic!("Can not send message: {:?} to {}", msg, handle));
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

use crate::common::components::{NetworkId, Position};
use crate::common::{network_channels_setup, ClientMessage, InputMessage, ServerMessage};
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

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(CurrentId::default())
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
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
    mut ids: ResMut<CurrentId>,
    mut network_event_reader: EventReader<NetworkEvent>,
    players: Query<(Entity, &NetworkHandle, &NetworkId)>,
) {
    for event in network_event_reader.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(_connection) => {
                    println!("New connection handle: {:?}", &handle);
                    let network_id = NetworkId(ids.0);
                    ids.0 += 1;

                    cmd.spawn()
                        .insert(NetworkHandle(*handle))
                        .insert(Position(Vec3::default()))
                        .insert(network_id);
                    net.send_message(*handle, ServerMessage::Welcome(network_id))
                        .expect("Could not send welcome");
                    net.send_message(*handle, ServerMessage::InsertLocalPlayer(network_id))
                        .expect("Could not send message");

                    for (_, _, id) in players.iter() {
                        net.send_message(*handle, ServerMessage::InsertPlayer(*id))
                            .expect("Could not send message");
                    }

                    for (_, connection, _) in players.iter() {
                        net.send_message(connection.0, ServerMessage::InsertPlayer(network_id))
                            .expect("Could not send message");
                    }
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            NetworkEvent::Disconnected(handle) => {
                println!("Remove ball: {:?}", *handle);
                for (entity, player_handle, _) in players.iter() {
                    if player_handle.0 == *handle {
                        cmd.entity(entity).despawn();
                    }
                }
            }
            _ => {}
        }
    }
}

fn read_network_channels_system(
    mut net: ResMut<NetworkResource>,
    mut input_events: EventWriter<InputEvent>,
) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(message) = channels.recv::<ClientMessage>() {
            match message {
                ClientMessage::Hello => {}
                ClientMessage::Input(e) => match e {
                    InputMessage::Move(dir) => {
                        input_events.send(InputEvent::Move(NetworkHandle(*handle), dir));
                    }
                },
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

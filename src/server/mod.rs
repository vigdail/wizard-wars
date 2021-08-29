use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use bevy::app::ScheduleRunnerSettings;
use bevy::prelude::*;
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use serde::{Deserialize, Serialize};

use crate::common::components::{NetworkId, Position};
use crate::common::{network_channels_setup, ServerMessage};

#[derive(Serialize, Deserialize)]
struct NetworkHandle(u32);

#[derive(Default)]
struct CurrentId(u32);

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(
            1000 / 30,
        )))
        .insert_resource(CurrentId::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(NetworkingPlugin::default())
        .add_startup_system(network_channels_setup.system())
        .add_startup_system(server_setup_system.system())
        .add_system(handle_network_events_system.system())
        .add_system(broadcast_changes_system.system());
    }
}

fn server_setup_system(mut net: ResMut<NetworkResource>) {
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

fn broadcast_changes_system(
    mut net: ResMut<NetworkResource>,
    changed_positions: Query<(&NetworkId, &Position), Changed<Position>>,
) {
    for (id, position) in changed_positions.iter() {
        let _ = net.broadcast_message((*id, *position));
    }
}

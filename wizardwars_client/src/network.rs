use crate::lobby::LobbyEvent;
use bevy::{app::AppExit, prelude::*};
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};
use turbulence::message_channels::ChannelMessage;
use wizardwars_shared::{
    components::Uuid,
    events::{DespawnEntityEvent, InsertPlayerEvent, SpawnEvent},
    messages::{
        client_messages::{ClientMessage, LobbyClientMessage},
        ecs_message::EcsCompPacket,
        network_channels_setup,
        server_messages::{ComponentSyncMessage, LobbyServerMessage, ServerMessage, WorldSync},
    },
    network::sync::packet::{CompPacket, CompUpdateKind},
};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<ClientMessage>()
            .add_event::<DespawnEntityEvent>()
            .add_event::<WorldSync>()
            .add_plugin(NetworkingPlugin {
                idle_timeout_ms: Some(3000),
                auto_heartbeat_ms: Some(1000),
                ..Default::default()
            })
            .add_startup_system(network_channels_setup.system())
            .add_startup_system(client_setup_system.system())
            .add_system(handle_network_events_system.system())
            .add_system(send_packets_system.system())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                read_server_message_channel_system.system(),
            )
            .add_system(despawn_entities_system.system())
            .add_system_to_stage(CoreStage::Last, handle_app_exit_event.system())
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new().with_system(world_sync_system.system()),
            );
    }
}

pub fn read_component_channel_system<C: ChannelMessage + std::fmt::Debug>(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
    entities_query: Query<(&Uuid, Entity)>,
) {
    let entities: HashMap<&Uuid, Entity> = entities_query.iter().map(|(id, e)| (id, e)).collect();

    for (_, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();
        while let Some(ComponentSyncMessage(pack)) = channels.recv::<ComponentSyncMessage>() {
            for (id, update) in pack.comp_updates.into_iter() {
                match entities.get(&id) {
                    Some(entity) => match update {
                        CompUpdateKind::Inserted(update) => {
                            update.apply_insert(*entity, &mut cmd);
                        }
                        CompUpdateKind::Modified(update) => {
                            update.apply_modify(*entity, &mut cmd);
                        }
                        CompUpdateKind::Removed(update) => {
                            EcsCompPacket::apply_remove(update, *entity, &mut cmd);
                        }
                    },
                    None => {
                        warn!("No entity found with id: {:?}", id);
                    }
                }
            }
        }
    }
}

fn client_setup_system(mut net: ResMut<NetworkResource>) {
    let ip_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket_address = SocketAddr::new(ip_address, 9001);
    info!("Connecting to {}...", socket_address);
    net.connect(socket_address);
}

fn handle_network_events_system(
    mut net: ResMut<NetworkResource>,
    mut network_event_reader: EventReader<NetworkEvent>,
) {
    for event in network_event_reader.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(_connection) => {
                    info!("Connection successful");

                    net.send_message(
                        *handle,
                        ClientMessage::LobbyMessage(LobbyClientMessage::Join(
                            "John Doe".to_owned(),
                        )),
                    )
                    .expect("Could not send hello");
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            NetworkEvent::Disconnected(handle) => {
                info!("Disconnected from: {}", handle);
            }
            _ => (),
        }
    }
}

fn read_server_message_channel_system(
    mut net: ResMut<NetworkResource>,
    mut insert_player_events: EventWriter<InsertPlayerEvent>,
    mut remove_player_events: EventWriter<DespawnEntityEvent>,
    mut lobby_events: EventWriter<LobbyEvent>,
    mut world_sync_events: EventWriter<WorldSync>,
    mut spawn_events: EventWriter<SpawnEvent>,
) {
    let mut disconnected = Vec::new();
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(message) = channels.recv::<ServerMessage>() {
            info!("Received message: {:?}", message);
            match message {
                ServerMessage::Lobby(msg) => match msg {
                    LobbyServerMessage::Welcome(pack) => {
                        world_sync_events.send(pack);
                    }
                    LobbyServerMessage::Reject { reason, disconnect } => {
                        error!("Cannot perform action: {:?}", reason);
                        if disconnect {
                            disconnected.push(*handle);
                        }
                    }
                    LobbyServerMessage::SetHost(_) => {}
                    LobbyServerMessage::PlayerJoined(_) => {
                        //
                    }
                    LobbyServerMessage::ReadyState(_) => {}
                    LobbyServerMessage::StartLoading => {
                        lobby_events.send(LobbyEvent::StartLoading);
                    }
                    LobbyServerMessage::PlayersList(_) => todo!(),
                },
                ServerMessage::Loading(_) => {}
                ServerMessage::Shopping(_) => {}
                ServerMessage::InsertPlayer(event) => {
                    insert_player_events.send(event);
                }
                ServerMessage::Spawn(spawn) => {
                    spawn_events.send(spawn);
                }
                ServerMessage::Despawn(id) => {
                    remove_player_events.send(DespawnEntityEvent { id });
                }
            }
        }
    }

    for handle in disconnected {
        net.disconnect(handle);
    }
}

fn send_packets_system(mut net: ResMut<NetworkResource>, mut events: EventReader<ClientMessage>) {
    for message in events.iter() {
        net.broadcast_message(message.clone());
    }
}

fn handle_app_exit_event(mut events: EventReader<AppExit>, mut net: ResMut<NetworkResource>) {
    if events.iter().next().is_none() {
        return;
    }

    info!("Closing all connections");

    let handles = net
        .connections
        .iter()
        .map(|(&handle, _)| handle)
        .collect::<Vec<_>>();

    for handle in handles {
        net.disconnect(handle);
    }
}

fn despawn_entities_system(
    mut cmd: Commands,
    mut events: EventReader<DespawnEntityEvent>,
    query: Query<(Entity, &Uuid)>,
) {
    let map = query
        .iter()
        .map(|(entity, id)| (id, entity))
        .collect::<HashMap<_, _>>();
    for event in events.iter() {
        if let Some(&entity) = map.get(&event.id) {
            cmd.entity(entity).despawn();
        } else {
            warn!("Trying to remove non existing entity: {:?}", event.id);
        }
    }
}

fn world_sync_system(
    mut cmd: Commands,
    mut events: EventReader<WorldSync>,
    query: Query<(&Uuid, Entity)>,
) {
    let map = query.iter().collect::<HashMap<_, _>>();
    for event in events.iter() {
        let package = &event.entity_package;
        let entity = map
            .get(&package.uid)
            .cloned()
            .unwrap_or_else(|| cmd.spawn().insert(package.uid).id());

        for comp in package.comps.iter() {
            comp.clone().apply_insert(entity, &mut cmd);
        }
    }
}

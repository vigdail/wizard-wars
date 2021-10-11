use crate::{loading::LoadCompleteEvent, lobby::LobbyEvent, states::ServerState};
use bevy::{prelude::*, utils::HashMap};
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use turbulence::message_channels::ChannelMessage;
use wizardwars_shared::{
    components::{Client, Host, Player, Position, Uuid},
    events::{ClientEvent, SpawnEvent},
    messages::{
        client_messages::{ActionMessage, ClientMessage, Verify},
        ecs_message::EcsCompPacket,
        network_channels_setup,
        server_messages::{ComponentSyncMessage, LobbyServerMessage, ServerMessage, WorldSync},
    },
    network::{
        sync::{
            packet::{CompSyncPackage, CompUpdateKind},
            EntityCommandsSync,
        },
        Dest, Pack,
    },
    resources::{DespawnedList, IdFactory, WorldSyncList},
};

pub struct NetworkPlugin;

pub type ServerPacket = Pack<ServerMessage>;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum SyncStage {
    Spawn,
    Despawn,
    Components,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(IdFactory::default())
            .add_event::<ServerPacket>()
            .add_plugin(NetworkingPlugin {
                idle_timeout_ms: Some(3000),
                auto_heartbeat_ms: Some(1000),
                ..Default::default()
            })
            .add_stage_after(CoreStage::Update, SyncStage::Spawn, SystemStage::parallel())
            .add_stage_after(
                SyncStage::Spawn,
                SyncStage::Despawn,
                SystemStage::parallel(),
            )
            .add_stage_after(
                SyncStage::Despawn,
                SyncStage::Components,
                SystemStage::parallel(),
            )
            .add_system_set(
                SystemSet::on_enter(ServerState::Init)
                    .with_system(network_channels_setup.system())
                    .with_system(server_setup_system.system()),
            )
            .add_system(handle_network_events_system.system())
            .add_system(read_network_channels_system.system())
            .add_system(send_packets_system.system())
            .add_system_to_stage(SyncStage::Spawn, spawn_sync_system.system())
            .add_system_to_stage(SyncStage::Spawn, world_sync_system.system())
            .add_system_to_stage(SyncStage::Despawn, despawn_sync_system.system())
            .add_system_set_to_stage(
                SyncStage::Components,
                SystemSet::new()
                    .with_system(broadcast_changes_system::<Position>.system())
                    .with_system(broadcast_changes_system::<Player>.system()),
            );
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
    clients: Query<(Entity, &Client, Option<&Host>)>,
) {
    let clients_map = clients
        .iter()
        .map(|(entity, client, host)| (client.0, (entity, host)))
        .collect::<HashMap<_, _>>();

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
                if let Some(&(entity, host)) = clients_map.get(handle) {
                    cmd.entity(entity).despawn_sync();
                    if host.is_some() {
                        let new_host_entity = clients_map
                            .iter()
                            .find_map(|(_, (entity, host))| host.map(|_| *entity));
                        if let Some(new_host_entity) = new_host_entity {
                            cmd.entity(new_host_entity).insert(Host);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn spawn_sync_system(mut packets: EventWriter<ServerPacket>, query: Query<&Uuid, Changed<Uuid>>) {
    for id in query.iter() {
        info!("Spawn: {:?}", id);
        packets.send(Pack::all(SpawnEvent::Entity(*id)));
    }
}

fn world_sync_system(mut packets: EventWriter<ServerPacket>, mut sync: ResMut<WorldSyncList>) {
    for (client, pack) in sync.iter() {
        packets.send(Pack::single(
            LobbyServerMessage::Welcome(WorldSync {
                entity_package: pack.clone(),
            }),
            *client,
        ));
    }

    sync.clear();
}

fn despawn_sync_system(
    mut packets: EventWriter<ServerPacket>,
    mut despawned: ResMut<DespawnedList>,
) {
    for id in despawned.iter() {
        info!("Despawn: {:?}", id);
        packets.send(Pack::all(ServerMessage::Despawn(*id)));
    }

    despawned.clear();
}

fn read_network_channels_system(
    mut net: ResMut<NetworkResource>,
    mut action_events: EventWriter<ClientEvent<ActionMessage>>,
    mut lobby_events: EventWriter<LobbyEvent>,
    mut loading_events: EventWriter<LoadCompleteEvent>,
    host_query: Query<&Client, With<Host>>,
) {
    let host = host_query.iter().next();
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        let client = Client(*handle);
        while let Some(message) = channels.recv::<ClientMessage>() {
            let is_host = host.map(|host| host == &client).unwrap_or(false);
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

fn broadcast_changes_system<C>(
    mut net: ResMut<NetworkResource>,
    changed: Query<(&Uuid, &C), Changed<C>>,
    // mut sync_pack: ResMut<
) where
    C: Into<EcsCompPacket> + ChannelMessage + std::fmt::Debug + Clone + Copy,
{
    for (id, component) in changed.iter() {
        // info!("Sync component: {:?} => {:?}", id, component);
        let pack = CompSyncPackage {
            comp_updates: vec![(*id, CompUpdateKind::Inserted((*component).into()))],
        };
        let message = ComponentSyncMessage(pack);
        let _ = net.broadcast_message(message);
    }
}

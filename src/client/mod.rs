use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use turbulence::message_channels::ChannelMessage;

use crate::common::messages::{LobbyClientMessage, LobbyServerMessage, ReadyState};
use crate::common::{
    components::{NetworkId, Position},
    messages::{network_channels_setup, ActionMessage, ClientMessage, ServerMessage},
};
use std::collections::HashMap;

pub struct ClientPlugin;

pub struct LocalPlayer;

pub enum InsertPlayerEvent {
    Remote(NetworkId),
    Local(NetworkId),
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(WindowDescriptor {
            width: 800.0,
            height: 600.0,
            ..Default::default()
        })
        .add_event::<InsertPlayerEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(NetworkingPlugin::default())
        .add_startup_system(network_channels_setup.system())
        .add_startup_system(setup_world_system.system())
        .add_startup_system(client_setup_system.system())
        .add_system(spawn_player_system.system())
        .add_system(handle_network_events_system.system())
        .add_system(read_server_message_channel_system.system())
        .add_system_to_stage(CoreStage::PreUpdate, input_system.system())
        .add_system_to_stage(
            CoreStage::PreUpdate,
            read_component_channel_system::<Position>.system(),
        )
        .add_system(update_ball_translation_system.system());
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
        if let NetworkEvent::Connected(handle) = event {
            match net.connections.get_mut(handle) {
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
            }
        }
    }
}

fn setup_world_system(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let map_material = StandardMaterial {
        base_color: Color::rgb(0.15, 0.27, 0.33),
        ..Default::default()
    };

    cmd.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(map_material),
        ..Default::default()
    });

    cmd.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });
    cmd.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(1.0, 5.0, 1.0)),
        ..Default::default()
    });
}

fn spawn_player_system(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<InsertPlayerEvent>,
) {
    let height = 1.0;
    let width = 0.5;

    for event in events.iter() {
        let (transform, local, id) = match event {
            InsertPlayerEvent::Remote(id) => {
                println!("Inserting remote player");
                (Transform::from_xyz(id.0 as f32 * 1.0, 0.0, 0.0), None, *id)
            }
            InsertPlayerEvent::Local(id) => {
                println!("Inserting local player");
                (
                    Transform::from_xyz(id.0 as f32 * 1.0, 0.0, 0.0),
                    Some(LocalPlayer),
                    *id,
                )
            }
        };
        let mut entity = cmd.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -width / 2.0,
                max_x: width / 2.0,
                min_y: 0.0,
                max_y: height,
                min_z: -width / 2.0,
                max_z: width / 2.0,
            })),
            transform,
            material: materials.add(Color::rgb(0.91, 0.44, 0.32).into()),
            ..Default::default()
        });
        entity.insert(Position::default());
        entity.insert(id);
        if local.is_some() {
            entity.insert(LocalPlayer);
        }
    }
}

fn read_server_message_channel_system(
    mut net: ResMut<NetworkResource>,
    mut events: EventWriter<InsertPlayerEvent>,
) {
    for (_, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(message) = channels.recv::<ServerMessage>() {
            match message {
                ServerMessage::LobbyMessage(msg) => match msg {
                    LobbyServerMessage::Welcome(id) => {
                        println!("Welcome message received: {:?}", id);
                    }
                    LobbyServerMessage::SetHost(id) => {
                        println!("Host now is: {:?}", id);
                    }
                    LobbyServerMessage::PlayerJoined(name) => {
                        println!("Player joined lobby: {}", name);
                    }
                    LobbyServerMessage::ReadyState(ready) => {
                        info!("Server Ready state changed: {:?}", ready);
                    }
                    LobbyServerMessage::StartLoading => todo!(),
                    LobbyServerMessage::PlayersList(_) => todo!(),
                },
                ServerMessage::InsertLocalPlayer(id) => {
                    events.send(InsertPlayerEvent::Local(id));
                }
                ServerMessage::InsertPlayer(id) => {
                    events.send(InsertPlayerEvent::Remote(id));
                }
            }
        }
    }
}

fn update_ball_translation_system(mut players: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in players.iter_mut() {
        transform.translation.x = position.0.x;
        transform.translation.z = position.0.z;
    }
}

fn input_system(input: Res<Input<KeyCode>>, mut net: ResMut<NetworkResource>) {
    let mut dir = Vec2::ZERO;
    let speed = 5.0;
    if input.pressed(KeyCode::W) {
        dir.y -= speed;
    }
    if input.pressed(KeyCode::S) {
        dir.y += speed;
    }
    if input.pressed(KeyCode::A) {
        dir.x -= speed;
    }
    if input.pressed(KeyCode::D) {
        dir.x += speed;
    }
    if input.just_pressed(KeyCode::Return) {
        net.broadcast_message(ClientMessage::LobbyMessage(
            LobbyClientMessage::ChangeReadyState(ReadyState::Ready),
        ));
    }
    if input.just_pressed(KeyCode::Space) {
        net.broadcast_message(ClientMessage::LobbyMessage(LobbyClientMessage::StartGame));
    }

    if dir.length() > 0.0 {
        let _ = net.broadcast_message(ClientMessage::Action(ActionMessage::Move(dir)));
    }
}

fn read_component_channel_system<C: ChannelMessage>(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
    players_query: Query<(&NetworkId, Entity, Option<&LocalPlayer>)>,
) {
    let players: HashMap<&NetworkId, (Entity, Option<&LocalPlayer>)> =
        players_query.iter().map(|(b, e, l)| (b, (e, l))).collect();

    for (_, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();
        while let Some((network_id, component)) = channels.recv::<(NetworkId, C)>() {
            match players.get(&network_id) {
                Some((entity, _)) => {
                    cmd.entity(*entity).insert(component);
                }
                None => {
                    println!("No player found");
                }
            }
        }
    }
}

use arena::ArenaPlugin;
use bevy::prelude::*;
use bevy_networking_turbulence::NetworkResource;
use camera::CameraPlugin;
use lobby::LobbyPlugin;
use network::{read_component_channel_system, NetworkPlugin};
use wizardwars_shared::{
    components::{Position, ReadyState},
    messages::client_messages::{ActionMessage, ClientMessage, LobbyClientMessage},
};

mod arena;
mod camera;
mod lobby;
mod network;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(WindowDescriptor {
            width: 800.0,
            height: 600.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(NetworkPlugin)
        .add_plugin(ArenaPlugin)
        .add_plugin(LobbyPlugin)
        .add_system_to_stage(CoreStage::PreUpdate, input_system.system())
        .add_system_to_stage(CoreStage::PreUpdate, network_mock_input_system.system())
        .add_system(update_translation_system.system())
        .add_system_to_stage(
            CoreStage::PreUpdate,
            read_component_channel_system::<Position>.system(),
        );
    }
}

fn update_translation_system(mut players: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (position, mut transform) in players.iter_mut() {
        transform.translation = position.0;
    }
}

fn input_system(input: Res<Input<KeyCode>>, mut net: ResMut<NetworkResource>) {
    let mut dir = Vec2::ZERO;
    if input.pressed(KeyCode::W) {
        dir.y -= 1.0;
    }
    if input.pressed(KeyCode::S) {
        dir.y += 1.0;
    }
    if input.pressed(KeyCode::A) {
        dir.x -= 1.0;
    }
    if input.pressed(KeyCode::D) {
        dir.x += 1.0;
    }

    if dir.length() > 0.0 {
        let _ = net.broadcast_message(ClientMessage::Action(ActionMessage::Move(dir.normalize())));
    }
}

fn network_mock_input_system(input: Res<Input<KeyCode>>, mut net: ResMut<NetworkResource>) {
    if input.just_pressed(KeyCode::Return) {
        net.broadcast_message(ClientMessage::LobbyMessage(
            LobbyClientMessage::ChangeReadyState(ReadyState::Ready),
        ));
    }

    if input.just_pressed(KeyCode::Space) {
        net.broadcast_message(ClientMessage::LobbyMessage(LobbyClientMessage::StartGame));
    }

    if input.just_pressed(KeyCode::Key1) {
        net.broadcast_message(ClientMessage::Action(ActionMessage::FireBall));
    }

    if input.just_pressed(KeyCode::B) {
        net.broadcast_message(ClientMessage::LobbyMessage(LobbyClientMessage::AddBot));
    }
}

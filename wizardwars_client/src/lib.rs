use arena::ArenaPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::{
    DebugCursorPickingPlugin, DebugEventsPickingPlugin, InteractablePickingPlugin, PickingCamera,
    PickingPlugin,
};
use bevy_networking_turbulence::NetworkResource;
use camera::CameraPlugin;
use lobby::LobbyPlugin;
use network::{read_component_channel_system, NetworkPlugin};
use wizardwars_shared::{
    components::{Position, ReadyState},
    messages::client_messages::{ActionMessage, ClientMessage, LobbyClientMessage},
    resources::{ArenaDimensions, CharacterDimensions},
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
        .insert_resource(CharacterDimensions::default())
        .insert_resource(ArenaDimensions::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(DebugCursorPickingPlugin)
        .add_plugin(DebugEventsPickingPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(NetworkPlugin)
        .add_plugin(ArenaPlugin)
        .add_plugin(LobbyPlugin)
        .add_plugin(WorldInspectorPlugin::new())
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

fn input_system(
    mouse_input: Res<Input<MouseButton>>,
    mut net: ResMut<NetworkResource>,
    camera_query: Query<&PickingCamera>,
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        let target = camera_query
            .single()
            .ok()
            .and_then(|camera| camera.intersect_top())
            .map(|(_, intersect)| intersect.position());

        if let Some(target) = target {
            net.broadcast_message(ClientMessage::Action(ActionMessage::Move { target }));
        }
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        let target = camera_query
            .single()
            .ok()
            .and_then(|camera| camera.intersect_top())
            .map(|(_, intersect)| intersect.position());

        if let Some(target) = target {
            net.broadcast_message(ClientMessage::Action(ActionMessage::FireBall(target)));
        }
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

    if input.just_pressed(KeyCode::B) {
        net.broadcast_message(ClientMessage::LobbyMessage(LobbyClientMessage::AddBot));
    }
}

use crate::common::network_channels_setup;
use crate::server::network::{server_setup_system, CurrentId};
use crate::server::states::{print_state_name_system, ServerState};
use bevy::prelude::*;
use bevy_networking_turbulence::NetworkingPlugin;

pub struct InitState;

impl Plugin for InitState {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(CurrentId::default())
            .add_plugin(NetworkingPlugin::default())
            .add_system_set(
                SystemSet::on_enter(ServerState::Init)
                    .with_system(print_state_name_system.system())
                    .with_system(network_channels_setup.system())
                    .with_system(server_setup_system.system()),
            )
            .add_system_set(
                SystemSet::on_update(ServerState::Init).with_system(check_init_system.system()),
            );
    }
}

fn check_init_system(mut app_state: ResMut<State<ServerState>>) {
    app_state
        .set(ServerState::Lobby)
        .expect("Can not change state");
}

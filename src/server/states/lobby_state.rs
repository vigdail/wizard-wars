use crate::server::states::{print_state_name_system, ServerState};
use bevy::prelude::*;

pub struct LobbyState;

impl Plugin for LobbyState {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(ServerState::Lobby).with_system(print_state_name_system.system()),
        );
    }
}

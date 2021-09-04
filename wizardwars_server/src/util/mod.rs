use crate::states::{print_state_name_system, ServerState};
use bevy::prelude::*;

pub struct PrintStateNamesPlugin;

impl Plugin for PrintStateNamesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(ServerState::Init).with_system(print_state_name_system.system()),
        )
        .add_system_set(
            SystemSet::on_enter(ServerState::Lobby).with_system(print_state_name_system.system()),
        )
        .add_system_set(
            SystemSet::on_enter(ServerState::WaitLoading)
                .with_system(print_state_name_system.system()),
        )
        .add_system_set(
            SystemSet::on_enter(ServerState::Shopping)
                .with_system(print_state_name_system.system()),
        )
        .add_system_set(
            SystemSet::on_enter(ServerState::Battle).with_system(print_state_name_system.system()),
        )
        .add_system_set(
            SystemSet::on_enter(ServerState::ShowResult)
                .with_system(print_state_name_system.system()),
        );
    }
}

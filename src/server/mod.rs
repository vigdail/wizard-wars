mod network;
mod states;
mod util;

use crate::common::components::Position;
use crate::server::network::{NetworkHandle, NetworkPlugin};
use crate::server::states::ServerState;
use crate::server::util::PrintStateNamesPlugin;
use bevy::app::ScheduleRunnerSettings;
use bevy::prelude::*;
use std::time::Duration;

enum InputEvent {
    Move(NetworkHandle, Vec2),
}

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(
            1000 / 60,
        )))
        .add_event::<InputEvent>()
        .add_state(ServerState::Init)
        .add_system_set(
            SystemSet::on_update(ServerState::Init).with_system(check_init_system.system()),
        )
        .add_plugins(MinimalPlugins)
        .add_plugin(NetworkPlugin)
        .add_plugin(PrintStateNamesPlugin)
        .add_system(handle_input_events_system.system());
    }
}

fn handle_input_events_system(
    mut events: EventReader<InputEvent>,
    time: Res<Time>,
    mut query: Query<(&NetworkHandle, &mut Position)>,
) {
    for (h, mut position) in query.iter_mut() {
        for event in events.iter() {
            match event {
                InputEvent::Move(handle, dir) => {
                    if h == handle {
                        position.0.x += dir.x * time.delta().as_millis() as f32 / 1000.0;
                        position.0.z += dir.y * time.delta().as_millis() as f32 / 1000.0;
                    }
                }
            }
        }
    }
}

fn check_init_system(mut app_state: ResMut<State<ServerState>>) {
    app_state
        .set(ServerState::Lobby)
        .expect("Can not change state");
}

mod arena;
mod battle;
mod loading;
mod lobby;
mod network;
mod shopping;
mod states;
mod util;

use battle::BattlePlugin;
use bevy::app::ScheduleRunnerSettings;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use loading::WaitLoadingPlugin;
use lobby::LobbyPlugin;
use network::NetworkPlugin;
use shopping::{ShoppingConfig, ShoppingTimerPlugin};
use states::ServerState;
use std::time::Duration;
use util::PrintStateNamesPlugin;
use wizardwars_shared::components::{Client, Position};

enum InputEvent {
    Move(Client, Vec2),
}

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(
            1000 / 60,
        )))
        .insert_resource(ShoppingConfig {
            time_in_seconds: 0.0,
        })
        .add_event::<InputEvent>()
        .add_state(ServerState::Init)
        .add_system_set(
            SystemSet::on_update(ServerState::Init).with_system(check_init_system.system()),
        )
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin::default())
        .add_plugin(NetworkPlugin)
        .add_plugin(LobbyPlugin)
        .add_plugin(WaitLoadingPlugin)
        .add_plugin(ShoppingTimerPlugin)
        .add_plugin(BattlePlugin)
        .add_plugin(PrintStateNamesPlugin)
        .add_system(handle_input_events_system.system());
    }
}

fn handle_input_events_system(
    mut events: EventReader<InputEvent>,
    time: Res<Time>,
    mut query: Query<(&Client, &mut Position)>,
) {
    for event in events.iter() {
        for (h, mut position) in query.iter_mut() {
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

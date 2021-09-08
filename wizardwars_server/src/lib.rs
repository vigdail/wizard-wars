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
use wizardwars_shared::components::{Client, NetworkId, Position};

enum ActionEvent {
    Move(Client, Vec2),
    Attack(Client, NetworkId),
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
        .add_event::<ActionEvent>()
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
        .add_system(handle_move_events_system.system());
    }
}

fn handle_move_events_system(
    mut events: EventReader<ActionEvent>,
    time: Res<Time>,
    mut query: Query<(&Client, &mut Position)>,
) {
    let speed = 5.0;
    for event in events.iter() {
        for (h, mut position) in query.iter_mut() {
            if let ActionEvent::Move(handle, dir) = &event {
                let offset = Vec3::new(dir.x, 0.0, dir.y) * speed;
                if h == handle {
                    position.0 += offset * time.delta().as_millis() as f32 / 1000.0;
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

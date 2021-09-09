use crate::{network::ServerPacket, states::ServerState};
use bevy::prelude::*;
use wizardwars_shared::messages::{ServerMessage, ShoppingServerMessage, TimerInfo};

pub struct ShoppingTimer {
    pub timer: Timer,
}

pub struct ShoppingConfig {
    pub time_in_seconds: f32,
}

pub struct ShoppingTimerPlugin;

impl Plugin for ShoppingTimerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(ServerState::Shopping).with_system(on_enter.system()),
        )
        .add_system_set(
            SystemSet::on_update(ServerState::Shopping)
                .with_system(update_timer.system())
                .with_system(check_timer.system()),
        )
        .add_system_set(SystemSet::on_exit(ServerState::Shopping).with_system(on_exit.system()));
    }
}

fn on_enter(mut cmd: Commands, config: Res<ShoppingConfig>) {
    let duration = config.time_in_seconds;
    cmd.insert_resource(ShoppingTimer {
        timer: Timer::from_seconds(duration, false),
    });
}

fn on_exit(mut cmd: Commands) {
    cmd.remove_resource::<ShoppingTimer>();
}

fn update_timer(
    mut packets: EventWriter<ServerPacket>,
    mut timer: ResMut<ShoppingTimer>,
    time: Res<Time>,
) {
    timer.timer.tick(time.delta());

    let timer_info = TimerInfo {
        duration: timer.timer.duration(),
        elapsed: timer.timer.elapsed(),
    };
    let packet = ServerPacket::all(ServerMessage::Shopping(ShoppingServerMessage::Timer(
        timer_info,
    )));
    packets.send(packet);
}

fn check_timer(timer: Res<ShoppingTimer>, mut state: ResMut<State<ServerState>>) {
    if !timer.timer.finished() {
        return;
    }

    state
        .set(ServerState::Battle)
        .expect("Unable to change state");
}

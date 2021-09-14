use bevy::{app::AppExit, prelude::*};
use bevy_networking_turbulence::NetworkResource;

use crate::states::ServerState;

pub struct ResultPlugin;

impl Plugin for ResultPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(ServerState::ShowResult).with_system(
                send_statistics
                    .system()
                    .chain(disconnect_all_clients.system())
                    .chain(close_app.system()),
            ),
        );
    }
}

fn send_statistics() {
    // TODO
}

fn disconnect_all_clients(mut net: ResMut<NetworkResource>) {
    info!("Disconnecting all clients");
    let connections = net
        .connections
        .iter()
        .map(|(&handle, _)| handle)
        .collect::<Vec<_>>();

    for handle in connections {
        net.disconnect(handle);
    }
}

fn close_app(mut events: EventWriter<AppExit>) {
    events.send(AppExit);
}

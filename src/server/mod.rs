use bevy::prelude::*;
use bevy_networking_turbulence::NetworkingPlugin;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(NetworkingPlugin::default());
    }
}

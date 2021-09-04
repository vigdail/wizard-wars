use bevy::prelude::*;

use wizardwars_server::ServerPlugin;

fn main() {
    App::build().add_plugin(ServerPlugin).run();
}

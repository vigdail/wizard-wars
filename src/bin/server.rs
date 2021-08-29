use bevy::prelude::*;

use wizard_wars::server::ServerPlugin;

fn main() {
    App::build().add_plugin(ServerPlugin).run();
}

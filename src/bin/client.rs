use bevy::prelude::*;

use wizard_wars::client::ClientPlugin;

fn main() {
    App::build().add_plugin(ClientPlugin).run();
}

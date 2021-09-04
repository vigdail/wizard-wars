use bevy::prelude::*;

use wizardwars_client::ClientPlugin;

fn main() {
    App::build().add_plugin(ClientPlugin).run();
}

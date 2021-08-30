pub mod init_state;
pub mod lobby_state;

use bevy::prelude::*;
pub use init_state::InitState;
pub use lobby_state::LobbyState;
use std::fmt::Formatter;

#[allow(dead_code)]
#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum ServerState {
    Init,
    Lobby,
    WaitLoading,
    Shopping,
    Battle,
    ShowResult,
}

impl std::fmt::Display for ServerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerState::Init => write!(f, "Init"),
            ServerState::Lobby => write!(f, "Lobby"),
            ServerState::WaitLoading => write!(f, "Wait Loading"),
            ServerState::Shopping => write!(f, "Shopping"),
            ServerState::Battle => write!(f, "Battle"),
            ServerState::ShowResult => write!(f, "Show Result"),
        }
    }
}

pub fn print_state_name_system(state: Res<State<ServerState>>) {
    println!("Current state: {}", state.current());
}

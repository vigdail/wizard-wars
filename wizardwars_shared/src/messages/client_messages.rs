use crate::components::NetworkId;
use crate::components::ReadyState;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LobbyClientMessage {
    Join(String),
    ChangeReadyState(ReadyState),
    GetPlayerList,
    StartGame,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionMessage {
    Move(Vec2),
    Attack { target: NetworkId },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    LobbyMessage(LobbyClientMessage),
    Loaded,
    Action(ActionMessage),
}

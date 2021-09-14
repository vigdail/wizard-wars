use crate::components::ReadyState;
use crate::components::Uuid;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LobbyClientMessage {
    Join(String),
    ChangeReadyState(ReadyState),
    GetPlayerList,
    AddBot,
    StartGame,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionMessage {
    Move(Vec2),
    Attack { target: Uuid },
    FireBall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    LobbyMessage(LobbyClientMessage),
    Loaded,
    Action(ActionMessage),
}

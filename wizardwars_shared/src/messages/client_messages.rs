use crate::components::{ReadyState, Uuid};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub trait Verify {
    fn verify(&self, _is_host: bool) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LobbyClientMessage {
    Join(String),
    ChangeReadyState(ReadyState),
    GetPlayerList,
    AddBot,
    StartGame,
}

impl Verify for LobbyClientMessage {
    fn verify(&self, is_host: bool) -> bool {
        match self {
            LobbyClientMessage::Join(_)
            | LobbyClientMessage::ChangeReadyState(_)
            | LobbyClientMessage::GetPlayerList => true,
            LobbyClientMessage::AddBot | LobbyClientMessage::StartGame => is_host,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionMessage {
    Move { target: Vec3 },
    Attack { target: Uuid },
    FireBall(Vec3),
}

impl Verify for ActionMessage {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    LobbyMessage(LobbyClientMessage),
    Loaded,
    Action(ActionMessage),
}

impl Verify for ClientMessage {
    fn verify(&self, is_host: bool) -> bool {
        match self {
            ClientMessage::LobbyMessage(message) => message.verify(is_host),
            ClientMessage::Loaded => true,
            ClientMessage::Action(message) => message.verify(is_host),
        }
    }
}

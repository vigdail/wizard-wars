use crate::components::{ReadyState, Uuid};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: proper interface
pub trait Validate {
    fn validate(&self, _client: Option<&Uuid>, _host: &Uuid) -> bool {
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

impl Validate for LobbyClientMessage {
    fn validate(&self, client: Option<&Uuid>, host: &Uuid) -> bool {
        match self {
            LobbyClientMessage::Join(_)
            | LobbyClientMessage::ChangeReadyState(_)
            | LobbyClientMessage::GetPlayerList => true,
            LobbyClientMessage::AddBot | LobbyClientMessage::StartGame => client == Some(host),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionMessage {
    Move(Vec2),
    Attack { target: Uuid },
    FireBall,
}

impl Validate for ActionMessage {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    LobbyMessage(LobbyClientMessage),
    Loaded,
    Action(ActionMessage),
}

impl Validate for ClientMessage {
    fn validate(&self, client: Option<&Uuid>, host: &Uuid) -> bool {
        match self {
            ClientMessage::LobbyMessage(message) => message.validate(client, host),
            ClientMessage::Loaded => true,
            ClientMessage::Action(message) => message.validate(client, host),
        }
    }
}

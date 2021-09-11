use crate::components::{NetworkId, ReadyState};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LobbyServerMessage {
    Welcome(NetworkId),
    PlayerJoined(NetworkId, String),
    PlayersList(Vec<String>),
    ReadyState(ReadyState),
    SetHost(NetworkId),
    StartLoading,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LoadingServerMessage {
    PlayerLoaded(NetworkId),
    LoadingComplete,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimerInfo {
    pub duration: Duration,
    pub elapsed: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ShoppingServerMessage {
    Timer(TimerInfo),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Lobby(LobbyServerMessage),
    Loading(LoadingServerMessage),
    Shopping(ShoppingServerMessage),
    InsertPlayer(NetworkId, Vec3),
    InsertLocalPlayer(NetworkId, Vec3),
    Despawn(NetworkId),
}

impl From<LobbyServerMessage> for ServerMessage {
    fn from(message: LobbyServerMessage) -> Self {
        Self::Lobby(message)
    }
}

impl From<LoadingServerMessage> for ServerMessage {
    fn from(message: LoadingServerMessage) -> Self {
        Self::Loading(message)
    }
}

impl From<ShoppingServerMessage> for ServerMessage {
    fn from(message: ShoppingServerMessage) -> Self {
        Self::Shopping(message)
    }
}

use crate::{
    components::{ReadyState, Uuid},
    enum_from,
    events::SpawnEvent,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LobbyServerMessage {
    Welcome(Uuid),
    PlayerJoined(Uuid, String),
    PlayersList(Vec<String>),
    ReadyState(ReadyState),
    SetHost(Uuid),
    StartLoading,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LoadingServerMessage {
    PlayerLoaded(Uuid),
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
    InsertPlayer(Uuid, Vec3),
    InsertLocalPlayer(Uuid, Vec3),
    Spawn(SpawnEvent),
    Despawn(Uuid),
}

enum_from!(ServerMessage, Lobby, LobbyServerMessage);
enum_from!(ServerMessage, Loading, LoadingServerMessage);
enum_from!(ServerMessage, Shopping, ShoppingServerMessage);

use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_networking_turbulence::{
    ConnectionChannelsBuilder, MessageChannelMode, MessageChannelSettings, NetworkResource,
    ReliableChannelSettings,
};
use serde::{Deserialize, Serialize};

use crate::common::components::{NetworkId, Position};

pub mod components;

pub const CLIENT_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
    channel: 0,
    channel_mode: MessageChannelMode::Reliable {
        reliability_settings: ReliableChannelSettings {
            bandwidth: 4096,
            recv_window_size: 1024,
            send_window_size: 1024,
            burst_bandwidth: 1024,
            init_send: 512,
            wakeup_time: Duration::from_millis(100),
            initial_rtt: Duration::from_millis(200),
            max_rtt: Duration::from_secs(2),
            rtt_update_factor: 0.1,
            rtt_resend_factor: 1.5,
        },
        max_message_len: 1024,
    },
    message_buffer_size: 8,
    packet_buffer_size: 8,
};

pub const SERVER_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
    channel: 1,
    channel_mode: MessageChannelMode::Reliable {
        reliability_settings: ReliableChannelSettings {
            bandwidth: 4096,
            recv_window_size: 1024,
            send_window_size: 1024,
            burst_bandwidth: 1024,
            init_send: 512,
            wakeup_time: Duration::from_millis(100),
            initial_rtt: Duration::from_millis(200),
            max_rtt: Duration::from_secs(2),
            rtt_update_factor: 0.1,
            rtt_resend_factor: 1.5,
        },
        max_message_len: 1024,
    },
    message_buffer_size: 8,
    packet_buffer_size: 8,
};

fn player_component_message_settings(channel: u8) -> MessageChannelSettings {
    MessageChannelSettings {
        channel,
        channel_mode: MessageChannelMode::Unreliable,
        message_buffer_size: 8,
        packet_buffer_size: 8,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionMessage {
    Move(Vec2),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Hello(String),
    StartGame,
    Loaded,
    Action(ActionMessage),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Welcome(NetworkId),
    PlayerJoined(String),
    SetHost(NetworkId),
    StartLoading,
    InsertPlayer(NetworkId),
    InsertLocalPlayer(NetworkId),
}

pub fn network_channels_setup(mut net: ResMut<NetworkResource>) {
    net.set_channels_builder(|builder: &mut ConnectionChannelsBuilder| {
        builder
            .register::<ClientMessage>(CLIENT_MESSAGE_SETTINGS)
            .unwrap();
        builder
            .register::<ServerMessage>(SERVER_MESSAGE_SETTINGS)
            .unwrap();
        builder
            .register::<(NetworkId, Position)>(player_component_message_settings(2))
            .unwrap();
    });
}

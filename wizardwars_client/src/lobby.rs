use bevy::prelude::*;
use wizardwars_shared::messages::ClientMessage;

pub enum LobbyEvent {
    StartLoading,
}

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<LobbyEvent>()
            .add_system(handle_events_system.system());
    }
}

fn handle_events_system(
    mut lobby_events: EventReader<LobbyEvent>,
    mut packets: EventWriter<ClientMessage>,
) {
    for event in lobby_events.iter() {
        match &event {
            LobbyEvent::StartLoading => {
                packets.send(ClientMessage::Loaded);
            }
        }
    }
}

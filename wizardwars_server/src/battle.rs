use std::collections::HashMap;

use bevy::prelude::*;
use wizardwars_shared::{
    components::{Client, Dead, Health, NetworkId, Position},
    messages::ServerMessage,
    network::Dest,
};

use crate::{arena::Arena, network::ServerPacket, states::ServerState, ActionEvent};

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(ServerState::Battle).with_system(setup_clients.system()),
        )
        .add_system_set(
            SystemSet::on_update(ServerState::Battle)
                .with_system(handle_attack_events_system.system())
                .with_system(handle_health_system.system())
                .with_system(debug_health_change_system.system())
                .with_system(debug_dead_message_system.system()),
        );
    }
}

fn setup_clients(
    mut cmd: Commands,
    arena: Res<Arena>,
    mut packets: EventWriter<ServerPacket>,
    clients: Query<(Entity, &NetworkId, &Client)>,
) {
    let spawn_points = arena.spawn_points();
    clients
        .iter()
        .zip(spawn_points.iter())
        .for_each(|((entity, id, client), point)| {
            cmd.entity(entity)
                .insert(Health {
                    current: 20,
                    maximum: 20,
                })
                .insert(Position(*point));
            packets.send(ServerPacket::new(
                ServerMessage::InsertPlayer(*id, *point),
                Dest::AllExcept(*client),
            ));
            packets.send(ServerPacket::new(
                ServerMessage::InsertLocalPlayer(*id, *point),
                Dest::Single(*client),
            ));
        });
}

fn handle_attack_events_system(
    mut events: EventReader<ActionEvent>,
    mut query: Query<(&NetworkId, &mut Health), Without<Dead>>,
) {
    let mut map = query
        .iter_mut()
        .map(|(id, health)| (id, health))
        .collect::<HashMap<_, _>>();

    for event in events.iter() {
        if let ActionEvent::Attack(_, target) = &event {
            if let Some(target_health) = map.get_mut(target) {
                target_health.current = target_health.current.saturating_sub(10);
            }
        }
    }
}

fn handle_health_system(mut cmd: Commands, query: Query<(Entity, &Health), Changed<Health>>) {
    for (entity, health) in query.iter() {
        if health.current == 0 {
            cmd.entity(entity).insert(Dead);
        }
    }
}

fn debug_health_change_system(query: Query<&Health, Changed<Health>>) {
    for component in query.iter() {
        info!("Changed: {:?}", component);
    }
}

fn debug_dead_message_system(query: Query<&NetworkId, Changed<Dead>>) {
    for id in query.iter() {
        info!("Entity with {:?} is dead", id);
    }
}

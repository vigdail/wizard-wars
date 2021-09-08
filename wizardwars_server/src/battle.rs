use std::collections::HashMap;

use bevy::prelude::*;
use wizardwars_shared::{
    components::{Client, Dead, Health, NetworkId, Position, Winner},
    messages::ServerMessage,
    network::Dest,
};

use crate::{arena::Arena, network::ServerPacket, states::ServerState, ActionEvent};

pub struct BattlePlugin;

pub struct PreparationTimer(Timer);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BattleState {
    None,
    Prepare,
    Battle,
}

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_state(BattleState::None)
            .add_system_set(
                SystemSet::on_enter(ServerState::Battle).with_system(setup_clients.system()),
            )
            .add_system_set(
                SystemSet::on_update(BattleState::Battle)
                    .with_system(handle_attack_events_system.system())
                    .with_system(handle_health_system.system())
                    .with_system(check_winer_system.system())
                    .with_system(check_switch_state_system.system())
                    .with_system(debug_health_change_system.system())
                    .with_system(debug_winner_change_system.system())
                    .with_system(debug_dead_message_system.system()),
            )
            .add_system_set(
                SystemSet::on_exit(ServerState::Battle).with_system(cleanup_system.system()),
            )
            .add_system_set(
                SystemSet::on_enter(BattleState::Prepare)
                    .with_system(start_preparation_timer.system()),
            )
            .add_system_set(
                SystemSet::on_update(BattleState::Prepare)
                    .with_system(check_preparation_timer.system()),
            )
            .add_system_set(
                SystemSet::on_update(BattleState::Battle)
                    .with_system(handle_move_events_system.system()),
            );
    }
}

fn setup_clients(
    mut cmd: Commands,
    arena: Res<Arena>,
    mut battle_state: ResMut<State<BattleState>>,
    mut packets: EventWriter<ServerPacket>,
    clients: Query<(Entity, &NetworkId, &Client)>,
) {
    battle_state
        .overwrite_set(BattleState::Prepare)
        .expect("Unable to switch battle state");
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

fn handle_move_events_system(
    mut events: EventReader<ActionEvent>,
    time: Res<Time>,
    mut query: Query<(&Client, &mut Position)>,
) {
    let speed = 5.0;
    for event in events.iter() {
        for (h, mut position) in query.iter_mut() {
            if let ActionEvent::Move(handle, dir) = &event {
                let offset = Vec3::new(dir.x, 0.0, dir.y) * speed;
                if h == handle {
                    position.0 += offset * time.delta().as_millis() as f32 / 1000.0;
                }
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

fn check_winer_system(mut cmd: Commands, query: Query<Entity, (With<Client>, Without<Dead>)>) {
    if let Ok(entity) = query.single() {
        cmd.entity(entity).insert(Winner);
    }
}

fn check_switch_state_system(
    mut state: ResMut<State<ServerState>>,
    mut arena: ResMut<Arena>,
    query: Query<&NetworkId, (With<Winner>, Changed<Winner>)>,
) {
    if query.iter().next().is_some() {
        let next_state = if arena.is_last_round() {
            ServerState::ShowResult
        } else {
            ServerState::Shopping
        };

        arena.next_round();

        state
            .set(next_state)
            .expect("Unable to switch server state");
    }
}

fn debug_health_change_system(query: Query<&Health, Changed<Health>>) {
    for health in query.iter() {
        info!("Changed: {:?}", health);
    }
}

fn debug_winner_change_system(query: Query<&NetworkId, Changed<Winner>>) {
    for id in query.iter() {
        info!("Player with {:?} is winner", id);
    }
}

fn debug_dead_message_system(query: Query<&NetworkId, Changed<Dead>>) {
    for id in query.iter() {
        info!("Entity with {:?} is dead", id);
    }
}

fn cleanup_system(
    mut cmd: Commands,
    mut battle_state: ResMut<State<BattleState>>,
    query: Query<Entity, With<Position>>,
) {
    battle_state
        .overwrite_set(BattleState::None)
        .expect("Unable to switch battle state");

    for entity in query.iter() {
        cmd.entity(entity)
            .remove::<Position>()
            .remove::<Dead>()
            .remove::<Winner>();
    }
}

fn start_preparation_timer(mut cmd: Commands) {
    let prepatation_timeout = 0.0;
    cmd.insert_resource(PreparationTimer(Timer::from_seconds(
        prepatation_timeout,
        false,
    )));
}

fn check_preparation_timer(
    mut timer: ResMut<PreparationTimer>,
    time: Res<Time>,
    mut battle_state: ResMut<State<BattleState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        battle_state
            .set(BattleState::Battle)
            .expect("Unable to switch battle state");
    }
}

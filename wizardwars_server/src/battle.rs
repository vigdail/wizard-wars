use crate::{arena::Arena, network::ServerPacket, states::ServerState, ActionEvent};
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use wizardwars_shared::{
    components::{
        damage::{Attack, FireBall},
        Bot, Client, Dead, Health, Player, Position, Uuid, Velocity, Waypoint, Winner,
    },
    messages::server_messages::ServerMessage,
    systems::{apply_damage_system, attack_system, collision_system, move_system, CollisionEvent},
};

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
            .add_event::<CollisionEvent>()
            .add_system_set(
                SystemSet::on_enter(ServerState::Battle).with_system(setup_players.system()),
            )
            .add_system_set(
                SystemSet::on_update(BattleState::Battle)
                    .with_system(handle_attack_events_system.system())
                    .with_system(handle_health_system.system())
                    .with_system(check_winer_system.system())
                    .with_system(apply_damage_system.system())
                    .with_system(move_system.system())
                    .with_system(attack_system.system())
                    .with_system(collision_system.system())
                    .with_system(check_switch_state_system.system())
                    .with_system(debug_health_change_system.system())
                    .with_system(debug_winner_change_system.system())
                    .with_system(debug_dead_message_system.system())
                    .with_system(bot_waypoint_system.system())
                    .with_system(move_to_waypoint_system.system()),
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

fn setup_players(
    mut cmd: Commands,
    arena: Res<Arena>,
    mut battle_state: ResMut<State<BattleState>>,
    mut packets: EventWriter<ServerPacket>,
    clients: Query<(Entity, &Uuid, Option<&Client>), With<Player>>,
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
                .insert(Health::new(20))
                .insert(Position(*point));

            if let Some(client) = client {
                packets.send(ServerPacket::except(
                    ServerMessage::InsertPlayer(*id, *point),
                    *client,
                ));
                packets.send(ServerPacket::single(
                    ServerMessage::InsertLocalPlayer(*id, *point),
                    *client,
                ));
            } else {
                packets.send(ServerPacket::all(ServerMessage::InsertPlayer(*id, *point)));
            }
        });
}

fn handle_attack_events_system(
    mut cmd: Commands,
    mut events: EventReader<ActionEvent>,
    query: Query<(&Position, &Client), (With<Health>, Without<Dead>)>,
) {
    let map = query.iter().map(|(p, c)| (c, p)).collect::<HashMap<_, _>>();
    for event in events.iter() {
        if let ActionEvent::FireBall(client) = &event {
            // if let Some(target_health) = map.get_mut(target) {
            //     target_health.change_by(-10);
            // }
            let attacker = **map.get(client).unwrap();
            for (target_position, c) in query.iter() {
                if c == client {
                    continue;
                }
                let dir = (attacker.0 - target_position.0).normalize();
                cmd.spawn()
                    .insert(attacker)
                    .insert(Velocity(-dir * 5.0))
                    .insert(FireBall {
                        attack: Attack::new(10),
                    });

                info!("attacker: {:?}", attacker);
                info!("target: {:?}", attacker);
                info!("dir: {:?}", dir);

                break;
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
                    position.0 += offset * time.delta_seconds();
                }
            }
        }
    }
}

fn handle_health_system(mut cmd: Commands, query: Query<(Entity, &Health), Changed<Health>>) {
    for (entity, health) in query.iter() {
        if health.should_die() {
            cmd.entity(entity).insert(Dead);
        }
    }
}

fn check_winer_system(mut cmd: Commands, query: Query<Entity, (With<Player>, Without<Dead>)>) {
    if let Ok(entity) = query.single() {
        cmd.entity(entity).insert(Winner);
    }
}

fn check_switch_state_system(
    mut state: ResMut<State<ServerState>>,
    mut arena: ResMut<Arena>,
    query: Query<&Uuid, (With<Winner>, Changed<Winner>)>,
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

fn debug_winner_change_system(query: Query<&Uuid, Changed<Winner>>) {
    for id in query.iter() {
        info!("Player with {:?} is winner", id);
    }
}

fn debug_dead_message_system(query: Query<&Uuid, Changed<Dead>>) {
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

fn bot_waypoint_system(mut cmd: Commands, query: Query<Entity, (With<Bot>, Without<Waypoint>)>) {
    let mut rng = rand::thread_rng();
    for entity in query.iter() {
        let target_position = Vec3::new(rng.gen_range(-5.0..5.0), 0.0, rng.gen_range(-5.0..5.0));
        cmd.entity(entity).insert(Waypoint(target_position));
    }
}

fn move_to_waypoint_system(
    mut cmd: Commands,
    mut query: Query<(Entity, &mut Position, &Waypoint)>,
    time: Res<Time>,
) {
    let speed = 1.0;
    for (entity, mut position, waypoint) in query.iter_mut() {
        let target = waypoint.0;
        let dir = (target - position.0).normalize();
        position.0 += dir * time.delta_seconds() * speed;
        if (position.0 - target).length() < 0.01 {
            cmd.entity(entity).remove::<Waypoint>();
        }
    }
}

use crate::{
    arena::Arena,
    network::{IdFactory, ServerPacket},
    states::ServerState,
};
use bevy::prelude::*;
use bevy_rapier3d::{
    physics::{ColliderBundle, IntoEntity, IntoHandle, RigidBodyBundle, RigidBodyPositionSync},
    prelude::{
        ActiveEvents, ColliderShape, ColliderType, NarrowPhase, RigidBodyForces,
        RigidBodyMassProps, RigidBodyMassPropsFlags, RigidBodyPosition, RigidBodyVelocity,
    },
};
use rand::Rng;
use std::collections::HashMap;
use wizardwars_shared::{
    components::{
        damage::{Attack, FireBall},
        Bot, Client, Dead, Health, LifeTime, Owner, Player, Position, Uuid, Waypoint, Winner,
    },
    events::{ClientEvent, InsertPlayerEvent, SpawnEvent},
    messages::{client_messages::ActionMessage, server_messages::ServerMessage},
    network::Pack,
    resources::{CharacterDimensions, PlayerColors},
    systems::apply_damage_system,
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
            .add_system_set(
                SystemSet::on_enter(ServerState::Battle).with_system(setup_players.system()),
            )
            .add_system_set(
                SystemSet::on_update(BattleState::Battle)
                    .with_system(handle_attack_events_system.system())
                    .with_system(handle_health_system.system())
                    .with_system(check_winer_system.system())
                    .with_system(apply_damage_system.system())
                    .with_system(check_switch_state_system.system())
                    .with_system(debug_health_change_system.system())
                    .with_system(debug_winner_change_system.system())
                    .with_system(debug_dead_message_system.system())
                    .with_system(bot_waypoint_system.system())
                    .with_system(move_to_waypoint_system.system())
                    .with_system(track_lifetime_system.system())
                    .with_system(fireball_collision_system.system())
                    .with_system(position_sync_system.system()),
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
    player_colors: Res<PlayerColors>,
    character_dimensions: Res<CharacterDimensions>,
    mut battle_state: ResMut<State<BattleState>>,
    mut packets: EventWriter<ServerPacket>,
    clients: Query<(Entity, &Uuid, Option<&Client>), With<Player>>,
) {
    battle_state
        .overwrite_set(BattleState::Prepare)
        .expect("Unable to switch battle state");
    let spawn_points = arena.spawn_points();
    let player_radius = character_dimensions.radius();
    let player_halfheight = character_dimensions.half_height();
    let y1 = player_radius;
    let y2 = player_radius + player_halfheight;

    itertools::izip!(
        clients.iter(),
        spawn_points.iter(),
        player_colors.colors.iter()
    )
    .for_each(|((entity, id, client), point, color)| {
        let collider = ColliderBundle {
            collider_type: ColliderType::Solid,
            shape: ColliderShape::capsule(
                [0.0, y1, 0.0].into(),
                [0.0, y2, 0.0].into(),
                player_radius,
            ),
            flags: (ActiveEvents::INTERSECTION_EVENTS).into(),
            ..Default::default()
        };

        let rigidbody = RigidBodyBundle {
            position: (*point).into(),
            mass_properties: RigidBodyMassProps {
                flags: RigidBodyMassPropsFlags::ROTATION_LOCKED_X
                    | RigidBodyMassPropsFlags::ROTATION_LOCKED_Y
                    | RigidBodyMassPropsFlags::ROTATION_LOCKED_Z,
                ..Default::default()
            },
            ..Default::default()
        };

        cmd.entity(entity)
            .insert(Health::new(20))
            .insert(Position(*point))
            .insert(Transform::default())
            .insert_bundle(collider)
            .insert_bundle(rigidbody)
            .insert(RigidBodyPositionSync::Discrete);

        let insert_player = InsertPlayerEvent {
            id: *id,
            position: *point,
            is_local: false,
            color: *color,
        };

        if let Some(client) = client {
            packets.send(ServerPacket::except(
                ServerMessage::InsertPlayer(insert_player.clone()),
                *client,
            ));
            packets.send(ServerPacket::single(
                ServerMessage::InsertPlayer(InsertPlayerEvent {
                    is_local: true,
                    ..insert_player
                }),
                *client,
            ));
        } else {
            packets.send(ServerPacket::all(ServerMessage::InsertPlayer(
                insert_player,
            )));
        }
    });
}

#[allow(clippy::type_complexity)]
fn handle_attack_events_system(
    mut cmd: Commands,
    mut id_factory: ResMut<IdFactory>,
    mut events: EventReader<ClientEvent<ActionMessage>>,
    query: Query<(Entity, &Position, &Client), (With<Health>, Without<Dead>)>,
    mut packets: EventWriter<ServerPacket>,
) {
    let map = query
        .iter()
        .map(|(e, p, c)| (c, (e, p)))
        .collect::<HashMap<_, _>>();
    for event in events.iter() {
        if let ActionMessage::FireBall(target) = &event.event() {
            let offset = 0.5;
            let (attacker_entity, attacker_position) = map.get(event.client()).unwrap();
            let origin = attacker_position.0 + Vec3::Y * offset;
            let target = Vec3::new(target.x, offset, target.z);
            let dir = (origin - target).normalize();

            let id = id_factory.generate();
            let collier = ColliderBundle {
                collider_type: ColliderType::Sensor,
                shape: ColliderShape::ball(0.1),
                flags: (ActiveEvents::INTERSECTION_EVENTS).into(),
                ..Default::default()
            };
            let rigidbody = RigidBodyBundle {
                position: origin.into(),
                velocity: RigidBodyVelocity {
                    linvel: (-dir * 5.0).into(),
                    ..Default::default()
                },
                forces: RigidBodyForces {
                    gravity_scale: 0.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            cmd.spawn()
                .insert(Position(origin))
                .insert(FireBall {
                    attack: Attack::new(10, 100.0),
                })
                .insert(Owner::new(*attacker_entity))
                .insert(LifeTime::from_seconds(5.0))
                .insert(Transform::default())
                .insert_bundle(collier)
                .insert_bundle(rigidbody)
                .insert(RigidBodyPositionSync::Discrete)
                .insert(id);

            cmd.entity(*attacker_entity).remove::<Waypoint>();

            packets.send(Pack::all(ServerMessage::Spawn(SpawnEvent::Projectile(id))));
        }
    }
}

// TODO: Should be called right after some rigidbody_positions_sync system
fn position_sync_system(mut query: Query<(&Transform, &mut Position), Changed<Transform>>) {
    for (transform, mut position) in query.iter_mut() {
        position.0 = transform.translation;
    }
}

fn fireball_collision_system(
    mut cmd: Commands,
    mut packets: EventWriter<ServerPacket>,
    fireballs: Query<(Entity, &FireBall, &Owner, &Uuid)>,
    healths: Query<&Health>,
    mut rigidbodies: Query<&mut RigidBodyForces>,
    rigidbody_props: Query<(&RigidBodyPosition, &RigidBodyVelocity)>,
    narrow_phase: Res<NarrowPhase>,
) {
    for (fireball_entity, fireball, fireball_owner, fireball_id) in fireballs.iter() {
        for (collider1, collider2, _) in narrow_phase.intersections_with(fireball_entity.handle()) {
            let target_entity = {
                let e1 = collider1.entity();
                let e2 = collider2.entity();

                if e1 == fireball_entity {
                    e2
                } else {
                    e1
                }
            };

            if fireball_owner.entity() == target_entity {
                continue;
            }

            let rigidbody = rigidbodies.get_mut(target_entity).ok();

            if let Some(mut rigidbody_force) = rigidbody {
                let (fireball_position, fireball_velocity) =
                    rigidbody_props.get(fireball_entity).unwrap();
                let (target_position, target_velocity) =
                    rigidbody_props.get(target_entity).unwrap();

                let mut diff_position: Vec3 = (fireball_position.position.translation.vector
                    - target_position.position.translation.vector)
                    .into();
                diff_position.y = 0.0;

                let mut diff_velocity: Vec3 =
                    (fireball_velocity.linvel - target_velocity.linvel).into();
                diff_velocity.y = 0.0;

                let dot = diff_position
                    .normalize()
                    .dot(diff_velocity.normalize_or_zero());

                let normal = diff_position.normalize() * dot * fireball.attack.knockback_force();
                let force = [normal.x, 0.0, normal.z].into();

                rigidbody_force.apply_force_at_point(
                    &Default::default(),
                    force,
                    [0.0, 0.0, 0.0].into(),
                );
            }

            let health = healths.get(target_entity).ok();

            if health.is_some() {
                cmd.entity(target_entity).insert(fireball.attack.damage());
            }

            cmd.entity(fireball_entity).despawn();
            packets.send(Pack::all(ServerMessage::Despawn(*fireball_id)));
        }
    }
}

fn handle_move_events_system(
    mut cmd: Commands,
    mut events: EventReader<ClientEvent<ActionMessage>>,
    query: Query<(Entity, &Client)>,
) {
    let clients = query
        .iter()
        .map(|(entity, client)| (client, entity))
        .collect::<HashMap<_, _>>();

    for event in events.iter() {
        if let ActionMessage::Move { target } = &event.event() {
            if let Some(&entity) = clients.get(event.client()) {
                cmd.entity(entity).insert(Waypoint(*target));
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
    mut query: Query<(Entity, &mut RigidBodyPosition, &Waypoint)>,
    time: Res<Time>,
) {
    let speed = 2.0;
    for (entity, mut position, waypoint) in query.iter_mut() {
        let target = waypoint.0;
        let dir = (target - position.position.translation.into()).normalize();
        let translation = dir * time.delta_seconds() * speed;
        position
            .position
            .append_translation_mut(&[translation.x, translation.y, translation.z].into());
        if (Vec3::from(position.position.translation) - target).length() < 0.02 {
            cmd.entity(entity).remove::<Waypoint>();
        }
    }
}

fn track_lifetime_system(
    mut cmd: Commands,
    mut query: Query<(Entity, &mut LifeTime, Option<&Uuid>)>,
    mut packets: EventWriter<ServerPacket>,
    time: ResMut<Time>,
) {
    for (entity, mut lifetime, id) in query.iter_mut() {
        if lifetime.timer.tick(time.delta()).just_finished() {
            cmd.entity(entity).despawn();
            if let Some(&id) = id {
                packets.send(Pack::all(ServerMessage::Despawn(id)));
            }
        }
    }
}

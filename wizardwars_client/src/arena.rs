use std::collections::HashMap;

use crate::camera::{CameraTarget, FollowCamera};
use bevy::prelude::*;
use bevy_mod_picking::{PickableBundle, PickingCameraBundle};
use wizardwars_shared::{components::Uuid, events::SpawnEvent, resources::ArenaDimensions};

pub struct InsertPlayerEvent {
    pub id: Uuid,
    pub position: Vec3,
    pub is_local: bool,
}

pub struct LocalPlayer;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<InsertPlayerEvent>()
            .add_event::<SpawnEvent>()
            .add_startup_system(setup_world_system.system())
            .add_system(spawn_player_system.system())
            .add_system(handle_spawn_events.system());
    }
}

fn setup_world_system(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    arena_dimensions: Res<ArenaDimensions>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let map_material = StandardMaterial {
        base_color: Color::rgb(0.25, 0.07, 0.03),
        roughness: 0.9,
        ..Default::default()
    };

    cmd.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: arena_dimensions.radius * 2.0,
        })),
        material: materials.add(map_material),
        ..Default::default()
    })
    .insert_bundle(PickableBundle::default());

    cmd.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    })
    .insert_bundle(PickingCameraBundle::default())
    .insert(FollowCamera {
        target: Vec3::ZERO,
        vertical_offset: 0.0,
        distance: 15.0,
    });
    cmd.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(1.0, 5.0, 1.0)),
        ..Default::default()
    });
}

fn spawn_player_system(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<InsertPlayerEvent>,
    query: Query<(Entity, &Uuid)>,
) {
    let height = 1.0;
    let width = 0.5;

    let clients = query
        .iter()
        .map(|(e, id)| (id, e))
        .collect::<HashMap<_, _>>();

    for event in events.iter() {
        let InsertPlayerEvent {
            id,
            position,
            is_local,
        } = *event;

        if let Some(&entity) = clients.get(&id) {
            cmd.entity(entity)
                .insert_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box {
                        min_x: -width / 2.0,
                        max_x: width / 2.0,
                        min_y: 0.0,
                        max_y: height,
                        min_z: -width / 2.0,
                        max_z: width / 2.0,
                    })),
                    transform: Transform::from_xyz(position.x, position.y, position.z),
                    material: materials.add(Color::rgb(0.32, 0.44, 0.91).into()),
                    ..Default::default()
                })
                .insert(id);
            if is_local {
                cmd.entity(entity).insert(LocalPlayer).insert(CameraTarget);
            }
        }
    }
}

fn handle_spawn_events(
    mut events: EventReader<SpawnEvent>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.iter() {
        match event {
            SpawnEvent::Projectile(id) => {
                spawn_projectile(&mut cmd, &mut meshes, &mut materials, *id)
            }
        }
    }
}

fn spawn_projectile(
    cmd: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    id: Uuid,
) {
    cmd.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere {
            radius: 0.1,
            subdivisions: 2,
        })),
        transform: Transform::identity(),
        material: materials.add(Color::rgb(0.9, 0.3, 0.2).into()),
        ..Default::default()
    })
    .insert(id);
}

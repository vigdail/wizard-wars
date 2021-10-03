use std::collections::HashMap;

use crate::camera::{CameraTarget, FollowCamera};
use bevy::prelude::*;
use bevy_mod_picking::{PickableBundle, PickingCameraBundle};
use wizardwars_shared::{components::Uuid, events::SpawnEvent, resources::CharacterDimensions};

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
            .add_system(apply_pickable.system())
            .add_system(spawn_player_system.system())
            .add_system(handle_spawn_events.system());
    }
}

// TODO: Find out a proper way to do this
fn apply_pickable(mut cmd: Commands, query: Query<(Entity, &Children, &Name), Changed<Name>>) {
    for (entity, children, name) in query.iter() {
        if name.as_str() == "arena" {
            cmd.entity(entity).insert_bundle(PickableBundle::default());
            for child in children.iter() {
                cmd.entity(*child).insert_bundle(PickableBundle::default());
            }
        }
    }
}

fn setup_world_system(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn_scene(asset_server.load("Arena1.gltf#Scene0"));

    cmd.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    })
    .insert_bundle(PickingCameraBundle::default())
    .insert(FollowCamera {
        target: Vec3::ZERO,
        vertical_offset: 0.0,
        distance: 20.0,
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
    character_dimensions: Res<CharacterDimensions>,
    query: Query<(Entity, &Uuid)>,
) {
    let height = character_dimensions.height();
    let width = character_dimensions.width();

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

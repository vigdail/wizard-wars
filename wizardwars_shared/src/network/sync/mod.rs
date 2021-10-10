pub mod packet;

use crate::{
    components::{Client, Player, Position, Uuid},
    messages::ecs_message::EcsCompPacket,
    resources::{DespawnedList, IdFactory, WorldSyncList},
};
use bevy::{
    ecs::{
        component::Component,
        system::{Command, EntityCommands},
    },
    prelude::*,
};

use self::packet::EntityPackage;

struct InsertUid {
    entity: Entity,
}

impl Command for InsertUid {
    fn write(self: Box<Self>, world: &mut World) {
        if let Some(mut factory) = world.get_resource_mut::<IdFactory>() {
            let id = factory.generate();
            world.entity_mut(self.entity).insert(id);
        }
    }
}

struct DespawnSync {
    entity: Entity,
}

impl Command for DespawnSync {
    fn write(self: Box<Self>, world: &mut World) {
        if let Some((id, mut list)) = world
            .entity(self.entity)
            .get::<Uuid>()
            .copied()
            .zip(world.get_resource_mut::<DespawnedList>())
        {
            list.push(id);
        }
    }
}

struct SyncWorld {
    entity: Entity,
}

impl Command for SyncWorld {
    fn write(self: Box<Self>, world: &mut World) {
        let client = world.get::<Client>(self.entity).copied().unwrap();
        let mut packs = Vec::new();

        for (entity, &uid) in world.query::<(Entity, &Uuid)>().iter(world) {
            let mut comps = Vec::new();
            fetch::<Position>(world, entity, &mut comps);
            fetch::<Player>(world, entity, &mut comps);

            let package = EntityPackage::<EcsCompPacket> { uid, comps };
            packs.push(package);
        }

        let mut list = world.get_resource_mut::<WorldSyncList>().unwrap();
        for package in packs {
            list.push(client, package);
        }
    }
}

fn fetch<C: Into<EcsCompPacket> + Component + Clone + std::fmt::Debug>(
    world: &World,
    entity: Entity,
    comps: &mut Vec<EcsCompPacket>,
) {
    if let Some(c) = world.get::<C>(entity).cloned() {
        info!("fetchind: {:?}", c);
        comps.push(c.into());
    }
}

pub trait CommandsSync<'a> {
    fn spawn_sync(&mut self) -> EntityCommands<'a, '_>;
}

pub trait EntityCommandsSync<'a, 'b> {
    fn despawn_sync(&mut self);
    fn sync_world(&mut self);
}

impl<'a> CommandsSync<'a> for Commands<'a> {
    fn spawn_sync(&mut self) -> EntityCommands<'a, '_> {
        let mut commands = self.spawn();
        let entity = commands.id();
        commands.commands().add(InsertUid { entity });

        commands
    }

    // fn sync_world(&mut self, entity: Entity) {
    //     self.add(SyncWorld { entity });
    // }
}

impl<'a, 'b> EntityCommandsSync<'a, 'b> for EntityCommands<'a, 'b> {
    fn despawn_sync(&mut self) {
        let entity = self.id();
        self.commands().add(DespawnSync { entity });
        self.despawn();
    }

    fn sync_world(&mut self) {
        let entity = self.id();
        self.commands().add(SyncWorld { entity });
    }
}

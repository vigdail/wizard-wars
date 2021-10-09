use crate::{
    components::Uuid,
    resources::{DespawnedList, IdFactory},
};
use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};

pub struct InsertUid {
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

pub struct DespawnSync {
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

pub trait CommandsSync<'a> {
    fn spawn_sync(&mut self) -> EntityCommands<'a, '_>;
}

pub trait EntityCommandsSync<'a, 'b> {
    fn despawn_sync(&mut self);
}

impl<'a> CommandsSync<'a> for Commands<'a> {
    fn spawn_sync(&mut self) -> EntityCommands<'a, '_> {
        let mut commands = self.spawn();
        let entity = commands.id();
        commands.commands().add(InsertUid { entity });

        commands
    }
}

impl<'a, 'b> EntityCommandsSync<'a, 'b> for EntityCommands<'a, 'b> {
    fn despawn_sync(&mut self) {
        let entity = self.id();
        self.commands().add(DespawnSync { entity });
        self.despawn();
    }
}

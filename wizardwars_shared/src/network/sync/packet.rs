use bevy::{ecs::component::Component, prelude::*};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

use crate::components::Uuid;

pub trait CompPacket: Clone + Debug + Send + 'static {
    type Phantom: Clone + Debug + Serialize + DeserializeOwned;

    fn apply_insert(self, entity: Entity, cmd: &mut Commands);
    fn apply_modify(self, entity: Entity, cmd: &mut Commands);
    fn apply_remove(_: Self::Phantom, entity: Entity, cmd: &mut Commands);
}

pub fn handle_insert<C: Component>(comp: C, entity: Entity, cmd: &mut Commands) {
    cmd.entity(entity).insert(comp);
}

pub fn handle_modify<C: Component + Debug>(comp: C, entity: Entity, cmd: &mut Commands) {
    // TODO: handle modify differently?
    cmd.entity(entity).insert(comp);
}

pub fn handle_remove<C: Component>(entity: Entity, cmd: &mut Commands) {
    cmd.entity(entity).remove::<C>();
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CompUpdateKind<P: CompPacket> {
    Inserted(P),
    Modified(P),
    Removed(P::Phantom),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompSyncPackage<P: CompPacket> {
    pub comp_updates: Vec<(Uuid, CompUpdateKind<P>)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityPackage<P: CompPacket> {
    pub uid: Uuid,
    pub comps: Vec<P>,
}

use super::components::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Dest {
    Single(Client),
    AllExcept(Client),
    All,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pack<T> {
    pub msg: T,
    pub dest: Dest,
}

impl<T> Pack<T> {
    pub fn new(msg: T, dest: Dest) -> Self {
        Self { msg, dest }
    }
}

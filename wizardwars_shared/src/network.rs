use crate::components::Client;
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

    pub fn all(msg: T) -> Self {
        Self {
            msg,
            dest: Dest::All,
        }
    }

    pub fn single(msg: T, client: Client) -> Self {
        Self {
            msg,
            dest: Dest::Single(client),
        }
    }

    pub fn except(msg: T, client: Client) -> Self {
        Self {
            msg,
            dest: Dest::AllExcept(client),
        }
    }
}

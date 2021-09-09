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
    pub fn new<M: Into<T>>(msg: M, dest: Dest) -> Self {
        Self {
            msg: msg.into(),
            dest,
        }
    }

    pub fn all<M: Into<T>>(msg: M) -> Self {
        Self {
            msg: msg.into(),
            dest: Dest::All,
        }
    }

    pub fn single<M: Into<T>>(msg: M, client: Client) -> Self {
        Self {
            msg: msg.into(),
            dest: Dest::Single(client),
        }
    }

    pub fn except<M: Into<T>>(msg: M, client: Client) -> Self {
        Self {
            msg: msg.into(),
            dest: Dest::AllExcept(client),
        }
    }
}

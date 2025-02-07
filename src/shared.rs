use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::{persist::PersistenceManager, Args};

#[derive(Clone, Debug)]
pub struct SharedResources {
    pub args: Args,
    pub persistent: PersistenceManager<SaveData>,
}

impl SharedResources {
    pub fn new(args: Args) -> SharedResources {
        let persistent = PersistenceManager::new(&args.save_path);
        SharedResources { args, persistent }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub ip: Option<IpAddr>,
    pub port: u16,
    pub username: String,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            ip: None,
            port: 11069,
            username: String::new(),
        }
    }
}

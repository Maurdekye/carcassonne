use crate::{persist::PersistenceManager, Args};

#[derive(Clone, Debug)]
pub struct SharedResources {
    pub args: Args,
    pub persistent: PersistenceManager,
}

impl SharedResources {
    pub fn new(args: Args) -> SharedResources {
        let persistent = PersistenceManager::new(&args);
        SharedResources { args, persistent }
    }
}

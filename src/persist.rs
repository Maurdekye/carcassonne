use std::{
    cell::{Ref, RefCell, RefMut},
    error::Error,
    fs::File,
    net::IpAddr,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf}, rc::Rc,
};

use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

use crate::Args;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentData {
    pub ip: Option<IpAddr>,
    pub port: u16,
    pub username: String,
}

impl Default for PersistentData {
    fn default() -> Self {
        Self {
            ip: None,
            port: 11069,
            username: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct DataMutGuard<'a> {
    data: RefMut<'a, PersistentData>,
    save_path: &'a Path,
}

impl Deref for DataMutGuard<'_> {
    type Target = PersistentData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for DataMutGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Drop for DataMutGuard<'_> {
    fn drop(&mut self) {
        debug!("saving to {}", self.save_path.display());
        let result: Result<(), Box<dyn Error>> = try {
            let mut file = File::create(self.save_path)?;
            serde_json::to_writer_pretty(&mut file, &*self.data)?;
        };
        if let Err(err) = result {
            error!("Error saving persistent data: {err}");
        }
    }
}

#[derive(Debug, Clone)]
pub struct PersistenceManager {
    data: Rc<RefCell<PersistentData>>,
    save_path: PathBuf,
}

impl PersistenceManager {
    pub fn new(args: &Args) -> PersistenceManager {
        let save_path = args.save_path.clone();
        let result: Result<PersistentData, Box<dyn Error>> = try {
            let mut file = File::open(&save_path)?;
            serde_json::from_reader(&mut file)?
        };
        let data = match result {
            Ok(data) => data,
            Err(err) => {
                warn!("Error loading persistent data: {err}");
                warn!("Loading defaults");
                let data = PersistentData::default();
                let result: Result<(), Box<dyn Error>> = try {
                    let mut file = File::create(&save_path)?;
                    serde_json::to_writer(&mut file, &data)?;
                };
                if let Err(err) = result {
                    error!("Error saving default config data: {err}");
                }
                data
            }
        };
        let data = Rc::new(RefCell::new(data));
        PersistenceManager { data, save_path }
    }

    pub fn borrow(&self) -> Ref<'_, PersistentData> {
        self.data.borrow()
    }

    pub fn borrow_mut(&mut self) -> DataMutGuard<'_> {
        DataMutGuard {
            data: self.data.borrow_mut(),
            save_path: &self.save_path,
        }
    }
}

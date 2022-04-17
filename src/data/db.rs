use std::fs;
use std::sync::Arc;

use sled::Db;

pub struct DbService;

pub const BASE_STORAGE_PATH: &str = "./storage";

impl DbService {

    pub fn create() -> Arc<Db> {
        Self::with_namespace("pooly")
    }

    pub fn with_namespace(namespace: &str) -> Arc<Db> {
        Arc::new(sled::open(Self::build_path(namespace)).unwrap())
    }

    pub (crate) fn clear(namespace: &str) -> Result<(), ()> {
        fs::remove_dir_all(Self::build_path(namespace)).map_err(
            |err| {
                println!("{}", err);

                ()
            }
        )
    }

    fn build_path(namespace: &str) -> String {
        BASE_STORAGE_PATH.to_owned() + "/" + namespace
    }

}

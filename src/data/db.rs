use std::sync::Arc;

use sled::Db;

pub struct DbBuilder;

pub const BASE_STORAGE_PATH: &str = "./stored";

impl DbBuilder {

    pub fn new() -> Arc<Db> {
        Arc::new(sled::open(BASE_STORAGE_PATH.to_owned() + "/pooly_configs").unwrap())
    }

}

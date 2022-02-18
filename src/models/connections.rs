use serde::{Deserialize, Serialize};
use zeroize::Zeroize;
use crate::models::versioned::Versioned;

pub type VersionedConnectionConfig = Versioned<ConnectionConfig>;

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct ConnectionConfig {

    pub id: String,
    pub hosts: Vec<String>,
    pub ports: Vec<u16>,
    pub db_name: String,
    pub user: String,
    pub password: String,
    pub max_connections: i32

}

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone)]
pub struct ZeroizeWrapper {

    value: Vec<u8>

}

impl ZeroizeWrapper {

    pub fn new(value: Vec<u8>) -> ZeroizeWrapper {
        ZeroizeWrapper {
            value
        }
    }

    pub fn get_value(&self) -> &Vec<u8> {
        &self.value
    }

}

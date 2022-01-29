use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct ConnectionConfig {

    pub hosts: Vec<String>,
    pub db_name: String,
    pub user: String,
    pub password: String,
    pub max_connections: i32

}

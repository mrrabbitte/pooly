use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
#[derive(PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct ConnectionConfig {

    pub hosts: Vec<String>,
    pub db_name: String,
    pub user_enc: String,
    pub pass_enc: String,
    pub max_connections: i32

}
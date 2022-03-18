use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct AppConfig {

    initialize_api_key: String

}

impl AppConfig {

    pub fn get_initialize_api_key(&self) -> &str {
        &self.initialize_api_key
    }

}

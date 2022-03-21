use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::models::app::AppConfig;

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct InitializeApiKey {

    value: String

}

impl InitializeApiKey {

    pub fn new(value: String) -> InitializeApiKey {
        InitializeApiKey {
            value
        }
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

}

impl From<&AppConfig> for InitializeApiKey {
    fn from(config: &AppConfig) -> Self {
        InitializeApiKey {
            value: config.get_initialize_api_key().into()
        }
    }
}


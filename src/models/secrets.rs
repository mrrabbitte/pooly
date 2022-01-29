use base64::DecodeError;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

pub const KEY_LENGTH: usize = 32;

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct EncryptionKey {
    value: Vec<u8>
}

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct MasterKey {
    value: Vec<u8>
}

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct MasterKeyShare {
    value: Vec<u8>
}

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct MasterKeySharePayload {
    value: String
}

impl EncryptionKey {

    pub fn new(value: Vec<u8>) -> EncryptionKey {
        EncryptionKey {
            value
        }
    }

    pub fn empty() -> EncryptionKey {
        EncryptionKey {
            value: vec![]
        }
    }

    pub fn get_value(&self) -> &Vec<u8> {
        &self.value
    }

}

impl MasterKey {

    pub fn new(value: Vec<u8>) -> MasterKey {
        MasterKey {
            value
        }
    }

    pub fn get_value(&self) -> &Vec<u8> {
        &self.value
    }

}

impl MasterKeyShare {

    pub fn new(value: Vec<u8>) -> MasterKeyShare {
        MasterKeyShare {
            value
        }
    }

    pub fn get_value(&self) -> &[u8] {
        &self.value
    }

}

impl From<MasterKeyShare> for MasterKeySharePayload {
    fn from(key_share: MasterKeyShare) -> Self {
        MasterKeySharePayload {
            value: base64::encode(&key_share.value)
        }
    }
}

impl TryFrom<MasterKeySharePayload> for MasterKeyShare {
    type Error = DecodeError;

    fn try_from(payload: MasterKeySharePayload) -> Result<Self, Self::Error> {
        Ok(
            MasterKeyShare {
                value: base64::decode(&payload.value)?
            }
        )
    }
}

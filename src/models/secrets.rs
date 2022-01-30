use base64::DecodeError;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

pub const KEY_LENGTH: usize = 32;

#[derive(Zeroize)]
#[zeroize(drop)]
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
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

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct EncryptedPayload {

    nonce: Vec<u8>,
    payload: Vec<u8>

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

impl EncryptedPayload {

    pub fn new(nonce: Vec<u8>,
               payload: Vec<u8>) -> EncryptedPayload {
        EncryptedPayload {
            nonce,
            payload
        }
    }

    pub fn get_nonce(&self) -> &Vec<u8> {
        &self.nonce
    }

    pub fn get_payload(&self) -> &Vec<u8> {
        &self.payload
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

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

use crate::models::errors::StorageError;

use serde::{Deserialize, Serialize};

pub type VersionedVec = Versioned<Vec<u8>>;

#[derive(Clone, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct Versioned<T> {

    version: u32,
    value: T

}

impl<T> Versioned<T> {

    pub fn new(value: T) -> Versioned<T> {
        Versioned { version: 0, value }
    }

    pub fn replace<U>(self, value: U) -> Versioned<U> {
        Versioned { version: self.version, value }
    }

    pub fn update(&self,
                  new: Versioned<T>) -> Result<Versioned<T>, StorageError> {
        if new.version != self.version {
            return Err(StorageError::OptimisticLockingError(self.version, new.version));
        }

        Ok(Versioned { version: self.version + 1, value: new.value } )
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }

}

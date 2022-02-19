use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::models::errors::StorageError;
use crate::models::time;

pub type VersionedVec = Versioned<Vec<u8>>;

#[derive(Clone, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct Versioned<T> {

    created_at: u128,
    version: u32,
    value: T

}

impl<T> Versioned<T> {

    pub fn new(value: T) -> Versioned<T> {
        Versioned { created_at: time::now_nanos(), version: 0, value }
    }

    pub fn with_new_value<U>(&self, value: U) -> Versioned<U> {
        Versioned { created_at: self.created_at, version: self.version, value }
    }

    pub fn update(&self,
                  new: Versioned<T>) -> Result<Versioned<T>, StorageError> {
        if new.version != self.version || new.created_at != self.created_at {
            return Err(StorageError::OptimisticLockingError{
                old_created_at: self.created_at,
                new_created_at: new.created_at,
                old_version: self.version,
                new_version: new.version
            });
        }

        Ok(Versioned { created_at: self.created_at, version: self.version + 1, value: new.value } )
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }

}

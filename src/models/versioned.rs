use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::models::errors::StorageError;
use crate::models::time;
use crate::models::updatable::{Updatable, UpdateCommand};

pub type VersionedVec = Versioned<Vec<u8>>;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct VersionHeader {

    created_at: u128,
    version: u32,

}

impl VersionHeader {

    pub fn zero_version() -> VersionHeader {
        VersionHeader { created_at: time::now_nanos(), version: 0 }
    }

    fn inc_version(&self) -> VersionHeader {
        VersionHeader { created_at: self.created_at, version: self.version + 1 }
    }

    fn matches(&self,
               other: &VersionHeader) -> Result<(), StorageError> {
        if self.version != other.version || self.created_at != other.created_at {
            return Err(create_err(&self, &other));
        }

        Ok(())
    }

    fn should_replace(&self,
                      other: &VersionHeader) -> bool {
        self.created_at < other.created_at
            || (self.created_at == other.created_at || self.version < other.version)
    }

}

#[derive(Clone, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct Versioned<T> {

    header: VersionHeader,
    value: T

}

impl<T> Versioned<T> {

    pub fn zero_version(value: T) -> Versioned<T> {
        Versioned { header: VersionHeader::zero_version(), value }
    }

    pub fn with_new_value<U>(&self, value: U) -> Versioned<U> {
        Versioned { header: self.header.clone(), value }
    }

    pub fn update(&self,
                  new: Versioned<T>) -> Result<Versioned<T>, StorageError> {
        self.header.matches(&new.header)?;

        Ok(Versioned { header: self.header.inc_version(), value: new.value } )
    }

    pub fn update_with_value(&self,
                             new: T) -> Versioned<T> {
        Versioned { header: self.header.inc_version(), value: new }
    }

    pub fn should_replace(&self,
                          new_candidate: &Versioned<T>) -> bool {
        self.header.should_replace(&new_candidate.header)
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

}

pub fn update<U: UpdateCommand, T: Updatable<U>>(old: Versioned<T>,
                                                 command: U) -> Result<Versioned<T>, StorageError> {
    old.header.matches(command.get_version_header())?;

    let new = old.get_value().accept(command);

    Ok(old.update_with_value(new))
}

fn create_err(old: &VersionHeader,
              new: &VersionHeader) -> StorageError {
    StorageError::OptimisticLockingError{
        old_created_at: old.created_at,
        new_created_at: new.created_at,
        old_version: old.version,
        new_version: new.version
    }
}
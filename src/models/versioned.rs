use crate::models::errors::StorageError;

pub struct Versioned<T> {

    version: u32,
    value: T

}

impl<T> Versioned<T> {

    pub fn new(value: T) -> Versioned<T> {
        Versioned { version: 0, value }
    }

    pub fn update(&self,
                  version: u32,
                  new_value: T) -> Result<Versioned<T>, StorageError> {
        if version != self.version {
            return Err(StorageError::OptimisticLockingError(self.version, version));
        }

        Ok(Versioned { version: version + 1, value: new_value } )
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }

}
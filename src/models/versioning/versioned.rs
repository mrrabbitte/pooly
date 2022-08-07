use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::models::errors::StorageError;
use crate::models::utils::time;
use crate::models::versioning::updatable::{Updatable, UpdateCommand};

pub type VersionedVec = Versioned<Vec<u8>>;

#[derive(Zeroize)]
#[zeroize(drop)]
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

    fn check_current_version(&self,
                             other: &VersionHeader) -> Result<(), StorageError> {
        if self.version != other.version || self.created_at != other.created_at {
            return Err(create_err(&self, &other));
        }

        Ok(())
    }

    pub fn check_next_version(&self, other: &VersionHeader) -> Result<(), StorageError> {
        if self.created_at != other.created_at || self.version + 1 != other.version {
            return Err(create_err(&self, &other));
        }

        Ok(())
    }

    fn should_be_replaced(&self,
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

    pub fn update_with_next_version(&self,
                                    new: Versioned<T>) -> Result<Versioned<T>, StorageError> {
        self.header.check_next_version(&new.header)?;

        Ok(self.update_with_value(new.value))
    }

    fn update_with_value(&self,
                         new: T) -> Versioned<T> {
        Versioned { header: self.header.inc_version(), value: new }
    }

    pub fn should_be_replaced(&self,
                              new_candidate: &Versioned<T>) -> bool {
        self.header.should_be_replaced(&new_candidate.header)
    }

    pub fn get_header(&self) -> &VersionHeader {
        &self.header
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

}

pub fn update<U: UpdateCommand, T: Updatable<U>>(old: Versioned<T>,
                                                 command: U) -> Result<Versioned<T>, StorageError> {
    old.header.check_current_version(command.get_version_header())?;

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

#[cfg(test)]
mod tests {
    use crate::models::utils::time;
    use crate::models::versioning::versioned::{Versioned, VersionHeader};

    type TestVersioned = Versioned<String>;

    #[test]
    fn test_creates_zero_version_correctly() {
        let now = time::now_nanos();

        let value = "some-value-1".to_string();

        let zero_version = TestVersioned::zero_version(value.clone());

        assert_eq!(zero_version.value, value);

        assert_eq!(zero_version.get_header().version, 0);
        assert!(zero_version.get_header().created_at > now);
    }

    #[test]
    fn test_rewrites_value_type_without_version_update_correctly() {
        let value = "some-value-1".to_string();
        let versioned = TestVersioned::zero_version(value.clone());

        let new_value = 128;
        let new_versioned = versioned.with_new_value(new_value);

        assert_eq!(new_versioned.get_header(), versioned.get_header());
    }

    #[test]
    fn test_updates_with_next_version_correctly() {
        let versioned =
            TestVersioned::zero_version("some-value-1".to_string());

        let new_value = "new-value-1".to_string();
        let next_version = TestVersioned {
            header: versioned.get_header().inc_version(),
            value: new_value.clone()
        };

        let new_versioned = versioned
            .update_with_next_version(next_version.clone())
            .expect("Got storage error.");

        assert_eq!(next_version.get_value().as_str(), new_value.as_str());
        assert_eq!(next_version.get_header().created_at, versioned.get_header().created_at);
        assert!(next_version.get_header().version > versioned.get_header().version);
    }

    #[test]
    fn test_determines_should_replace_correctly() {
        let old_versioned =
            TestVersioned::zero_version("other-value-1".to_string());
        let versioned =
            TestVersioned::zero_version("some-value-1".to_string());
        let next_versioned = versioned.update_with_next_version(
            TestVersioned {
                header: versioned.get_header().inc_version(),
                value: "other-value-2".to_string()
            }
        ).unwrap();
        let other_root = TestVersioned {
            header: VersionHeader::zero_version().inc_version(),
            value: "other-value-3".to_string()
        };

        assert!(!versioned.should_be_replaced(&old_versioned));

        assert!(old_versioned.should_be_replaced(&versioned));

        assert!(next_versioned.should_be_replaced(&versioned));

        assert!(!other_root.should_be_replaced(&versioned));
    }

}

use std::collections::HashSet;
use std::hash::Hash;

use serde::{de, Deserialize, Serialize};

use crate::models::errors::StorageError;
use crate::models::versioning::versioned::{Versioned, VersionHeader};
use crate::models::utils::wildcards::WildcardPattern;

pub type StringSetCommand = SetCommand<String>;
pub type WildcardPatternSetCommand = SetCommand<WildcardPattern>;

pub trait Updatable<U: UpdateCommand> {

    fn get_id(&self) -> &str;
    fn accept(&self, update: U) -> Self;

}

pub trait UpdateCommand {

    fn get_version_header(&self) -> &VersionHeader;

}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct SetCommand<T: Eq + Hash + Clone + Serialize> {

    cmd_type: SetCommandType,
    header: VersionHeader,

    #[serde(bound(deserialize = "T: Deserialize<'de>"))]
    elements: HashSet<T>,

}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum SetCommandType {

    Add,
    Remove,
    Replace

}

impl<T: Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>> SetCommand<T> {

    pub fn apply(self,
                 target: &HashSet<T>) -> HashSet<T> {
        match self.cmd_type {
            SetCommandType::Add =>
                self.elements.union(target).cloned().collect(),
            SetCommandType::Remove =>
                self.elements.difference(target).cloned().collect(),
            SetCommandType::Replace =>
                self.elements,
        }
    }

}

impl<T: Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>> UpdateCommand for SetCommand<T> {
    fn get_version_header(&self) -> &VersionHeader {
        &self.header
    }
}

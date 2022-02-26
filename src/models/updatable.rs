use std::collections::HashSet;
use std::hash::Hash;
use crate::models::versioned::VersionHeader;
use crate::models::wildcards::WildcardPattern;

pub trait Updatable<U: UpdateCommand> {
    fn accept(&self, update: U) -> Self;
}

pub trait UpdateCommand {

    fn get_version_header(&self) -> &VersionHeader;

}

pub type StringSetCommand = SetCommand<String>;
pub type WildcardPatternSetCommand = SetCommand<WildcardPattern>;

pub struct SetCommand<T: Eq + Hash + Clone> {

    cmd_type: SetCommandType,
    header: VersionHeader,
    elements: HashSet<T>,

}

pub enum SetCommandType {

    Add,
    Remove,
    Replace

}

impl<T: Eq + Hash + Clone> SetCommand<T> {

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

impl<T: Eq + Hash + Clone> UpdateCommand for SetCommand<T> {
    fn get_version_header(&self) -> &VersionHeader {
        &self.header
    }
}

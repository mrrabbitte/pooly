use std::collections::HashSet;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::models::utils::wildcards::WildcardPattern;
use crate::models::versioning::versioned::VersionHeader;

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
                target.difference(&self.elements).cloned().collect(),
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


#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::models::versioning::updatable::{SetCommandType, StringSetCommand};
    use crate::models::versioning::versioned::VersionHeader;

    #[test]
    fn test_adds_elements_correctly() {
        assert_eq!(add_command(&["a", "b"]).apply(&hash_set(&[])),
                   hash_set(&["a", "b"]));
        assert_eq!(add_command(&[]).apply(&hash_set(&["a", "b"])),
                   hash_set(&["a", "b"]));

        assert_eq!(add_command(&["a", "b"]).apply(&hash_set(&["a"])),
                   hash_set(&["a", "b"]));
        assert_eq!(add_command(&["a", "b"]).apply(&hash_set(&["a", "b"])),
                   hash_set(&["a", "b"]));
        assert_eq!(add_command(&["a", "b"]).apply(&hash_set(&["c"])),
                   hash_set(&["a", "b", "c"]));
    }

    #[test]
    fn test_removes_elements_correctly() {
        assert_eq!(remove_command(&["a", "b"]).apply(&hash_set(&[])),
                   hash_set(&[]));
        assert_eq!(remove_command(&[]).apply(&hash_set(&["a", "b"])),
                   hash_set(&["a", "b"]));

        assert_eq!(remove_command(&["a", "b"]).apply(&hash_set(&["a"])),
                   hash_set(&[]));
        assert_eq!(remove_command(&["a", "b"]).apply(&hash_set(&["a", "b"])),
                   hash_set(&[]));

        assert_eq!(remove_command(&["a", "b"]).apply(&hash_set(&["c"])),
                   hash_set(&["c"]));
        assert_eq!(remove_command(&["c"]).apply(&hash_set(&["a", "b"])),
                   hash_set(&["a", "b"]));
    }

    #[test]
    fn test_replaces_elements_correctly() {
        assert_eq!(replace_command(&["a", "b"]).apply(&hash_set(&[])),
                   hash_set(&["a", "b"]));
        assert_eq!(replace_command(&[]).apply(&hash_set(&["a", "b"])),
                   hash_set(&[]));
        assert_eq!(replace_command(&["c"]).apply(&hash_set(&["a", "b"])),
                   hash_set(&["c"]));
    }

    fn add_command(elements: &[&str]) -> StringSetCommand {
        set_command(hash_set(elements), SetCommandType::Add)
    }

    fn remove_command(elements: &[&str]) -> StringSetCommand {
        set_command(hash_set(elements), SetCommandType::Remove)
    }

    fn replace_command(elements: &[&str]) -> StringSetCommand {
        set_command(hash_set(elements), SetCommandType::Replace)
    }

    fn set_command(elements: HashSet<String>,
                   cmd_type: SetCommandType) -> StringSetCommand {
        StringSetCommand {
            cmd_type,
            header: VersionHeader::zero_version(),
            elements
        }
    }

    fn hash_set(elements: &[&str]) -> HashSet<String> {
        elements.iter().map(|element| element.to_string()).collect()
    }
}
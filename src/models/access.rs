use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::models::updatable::{StringSetCommand, Updatable, WildcardPatternSetCommand};
use crate::models::versioned::{Versioned, VersionHeader};
use crate::models::wildcards::WildcardPattern;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConnectionAccessControlEntry {

    client_id: String,
    connection_ids: Versioned<HashSet<String>>,
    connection_id_patterns: Versioned<HashSet<WildcardPattern>>

}

impl ConnectionAccessControlEntry {

    pub fn new(client_id: String,
               connection_ids: Versioned<HashSet<String>>,
               connection_id_patterns: Versioned<HashSet<WildcardPattern>>) -> ConnectionAccessControlEntry {
        ConnectionAccessControlEntry {
            client_id,
            connection_ids,
            connection_id_patterns
        }
    }

    pub fn matches(&self,
                   client_id: &str,
                   connection_id: &str) -> bool {
        if !client_id.eq(&self.client_id) {
            return false;
        }

        let connection_ids = self.connection_ids.get_value();
        let connection_id_patterns = self.connection_id_patterns.get_value();

        if !connection_ids.is_empty() && connection_ids.contains(connection_id) {
            return true;
        }

        for pattern in connection_id_patterns {
            if pattern.matches(connection_id) {
                return true;
            }
        }

        false
    }

    pub fn with_connection_ids(self,
                               connection_ids: Versioned<HashSet<String>>)
        -> ConnectionAccessControlEntry {
        if self.connection_ids.should_replace(&connection_ids) {
            return ConnectionAccessControlEntry {
                client_id: self.client_id,
                connection_ids,
                connection_id_patterns: self.connection_id_patterns
            };
        }

        self
    }

    pub fn with_connection_id_patterns(self,
                                       connection_id_patterns: Versioned<HashSet<WildcardPattern>>)
        -> ConnectionAccessControlEntry {
        if self.connection_id_patterns.should_replace(&connection_id_patterns) {
            return ConnectionAccessControlEntry {
                client_id: self.client_id,
                connection_ids: self.connection_ids,
                connection_id_patterns
            };
        }

        self
    }

    pub fn is_empty(&self) -> bool {
        self.connection_ids.get_value().is_empty()
            && self.connection_id_patterns.get_value().is_empty()
    }

}

pub trait ConnectionIdAccessEntry {

    fn matches(&self,
               client_id: &str,
               connection_id: &str) -> bool;

}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LiteralConnectionIdAccessEntry {

    client_id: String,
    connection_ids: HashSet<String>

}

impl ConnectionIdAccessEntry for LiteralConnectionIdAccessEntry {
    fn matches(&self, client_id: &str, connection_id: &str) -> bool {
        if !client_id.eq(&self.client_id) {
            return false;
        }

        !self.connection_ids.is_empty() && self.connection_ids.contains(connection_id)
    }
}

impl Updatable<StringSetCommand> for LiteralConnectionIdAccessEntry {
    fn accept(&self, update: StringSetCommand) -> Self {
        LiteralConnectionIdAccessEntry {
            client_id: self.client_id.clone(),
            connection_ids: update.apply(&self.connection_ids)
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PatternConnectionIdAccessEntry {

    client_id: String,
    patterns: HashSet<WildcardPattern>

}

impl ConnectionIdAccessEntry for PatternConnectionIdAccessEntry {
    fn matches(&self, client_id: &str, connection_id: &str) -> bool {
        if !client_id.eq(&self.client_id) {
            return false;
        }

        for pattern in &self.patterns {
            if pattern.matches(connection_id) {
                return true;
            }
        }

        false
    }
}

impl Updatable<WildcardPatternSetCommand> for PatternConnectionIdAccessEntry {
    fn accept(&self, update: WildcardPatternSetCommand) -> Self {
        PatternConnectionIdAccessEntry {
            client_id: self.client_id.clone(),
            patterns: update.apply(&self.patterns)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::models::access::ConnectionAccessControlEntry;
    use crate::models::versioned::Versioned;
    use crate::models::wildcards::WildcardPattern;

    const CLIENT_ID: &str = "client-id-1";

    const FIRST_CONNECTION_ID: &str = "first-connection-id";
    const SECOND_CONNECTION_ID: &str = "second-connection-id";
    const THIRD_CONNECTION_ID: &str = "third___connection___id";
    const FOURTH_CONNECTION_ID: &str = "fourth-connection-id";

    const FIRST_CONN_ID: &str = "first_conn_id";

    #[test]
    fn test_matches_correctly_exact() {
        let mut should_match = get_should_match();

        let ace = ConnectionAccessControlEntry::new(
            CLIENT_ID.to_string(),
            Versioned::zero_version(should_match.clone()),
            Versioned::zero_version(HashSet::new()));

        for connection_id in &should_match {
            assert!(ace.matches(CLIENT_ID, &connection_id));
        }

        assert!(should_match.remove(FIRST_CONNECTION_ID));

        let ace = ConnectionAccessControlEntry::new(
            CLIENT_ID.to_string(),
            Versioned::zero_version(should_match.clone()),
            Versioned::zero_version(HashSet::new()));

        for connection_id in should_match {
            assert_eq!(ace.matches(CLIENT_ID, &connection_id),
                       !connection_id.eq(FIRST_CONNECTION_ID));
        }
    }

    #[test]
    fn test_matches_by_patterns() {
        let mut patterns = HashSet::new();

        patterns.insert(WildcardPattern::parse("*connection-id").unwrap());
        patterns.insert(WildcardPattern::parse("*connection*").unwrap());
        patterns.insert(WildcardPattern::parse("first*").unwrap());

        let ace = ConnectionAccessControlEntry::new(
            CLIENT_ID.to_string(),
            Versioned::zero_version(HashSet::new()),
            Versioned::zero_version(patterns));

        for connection_id in get_should_match() {
            assert!(ace.matches(CLIENT_ID, &connection_id));
        }
    }

    #[test]
    fn test_does_not_match_on_client_id_mismatch() {
        let should_match = get_should_match();

        let ace = ConnectionAccessControlEntry::new(
            "not".to_string() + CLIENT_ID,
            Versioned::zero_version(should_match.clone()),
            Versioned::zero_version(HashSet::new()));

        for connection_id in &should_match {
            assert!(!ace.matches(CLIENT_ID, &connection_id));
        }
    }

    #[test]
    fn empty_never_matches() {
        let ace = ConnectionAccessControlEntry::new(
            CLIENT_ID.to_string(),
            Versioned::zero_version(HashSet::new()),
            Versioned::zero_version(HashSet::new()));

        for connection_id in get_should_match() {
            assert!(!ace.matches(CLIENT_ID, &connection_id));
        }
    }

    fn get_should_match() -> HashSet<String> {
        let mut response = HashSet::new();

        response.insert(FIRST_CONNECTION_ID.to_string());
        response.insert(SECOND_CONNECTION_ID.to_string());
        response.insert(THIRD_CONNECTION_ID.to_string());
        response.insert(FOURTH_CONNECTION_ID.to_string());
        response.insert(FIRST_CONN_ID.to_string());

        response
    }

}
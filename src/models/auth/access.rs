use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::models::utils::wildcards::WildcardPattern;
use crate::models::versioning::updatable::{StringSetCommand, Updatable, WildcardPatternSetCommand};

pub trait ConnectionIdAccessEntry {

    fn is_allowed(&self,
                  client_id: &str,
                  connection_id: &str) -> bool;

}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LiteralConnectionIdAccessEntry {

    client_id: String,
    connection_ids: HashSet<String>

}

impl LiteralConnectionIdAccessEntry {

    pub fn new(client_id: &str,
               connection_ids: HashSet<String>) -> LiteralConnectionIdAccessEntry {
        LiteralConnectionIdAccessEntry {
            client_id: client_id.into(),
            connection_ids
        }
    }

    pub fn one(client_id: &str,
               connection_id: &str) -> LiteralConnectionIdAccessEntry {
        let mut connection_ids = HashSet::new();

        connection_ids.insert(connection_id.into());

        LiteralConnectionIdAccessEntry::new(client_id, connection_ids)
    }

}

impl ConnectionIdAccessEntry for LiteralConnectionIdAccessEntry {
    fn is_allowed(&self, client_id: &str, connection_id: &str) -> bool {
        if !client_id.eq(&self.client_id) {
            return false;
        }

        !self.connection_ids.is_empty() && self.connection_ids.contains(connection_id)
    }
}

impl Updatable<StringSetCommand> for LiteralConnectionIdAccessEntry {
    fn get_id(&self) -> &str {
        &self.client_id
    }

    fn accept(&self, update: StringSetCommand) -> Self {
        LiteralConnectionIdAccessEntry {
            client_id: self.client_id.clone(),
            connection_ids: update.apply(&self.connection_ids)
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WildcardPatternConnectionIdAccessEntry {

    client_id: String,
    patterns: HashSet<WildcardPattern>

}

impl WildcardPatternConnectionIdAccessEntry {

    pub fn new(client_id: &str,
               patterns: HashSet<WildcardPattern>) -> WildcardPatternConnectionIdAccessEntry {
        WildcardPatternConnectionIdAccessEntry {
            client_id: client_id.into(),
            patterns
        }
    }

}

impl ConnectionIdAccessEntry for WildcardPatternConnectionIdAccessEntry {
    fn is_allowed(&self, client_id: &str, connection_id: &str) -> bool {
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

impl Updatable<WildcardPatternSetCommand> for WildcardPatternConnectionIdAccessEntry {
    fn get_id(&self) -> &str {
        &self.client_id
    }

    fn accept(&self, update: WildcardPatternSetCommand) -> Self {
        WildcardPatternConnectionIdAccessEntry {
            client_id: self.client_id.clone(),
            patterns: update.apply(&self.patterns)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::models::auth::access::{ConnectionIdAccessEntry, LiteralConnectionIdAccessEntry, WildcardPatternConnectionIdAccessEntry};
    use crate::models::utils::wildcards::WildcardPattern;
    use crate::models::versioning::versioned::Versioned;

    const CLIENT_ID: &str = "client-id-1";
    const NOT_CLIENT_ID: &str = "not-client-id-1";

    const FIRST_CONNECTION_ID: &str = "first-connection-id";
    const SECOND_CONNECTION_ID: &str = "second-connection-id";
    const THIRD_CONNECTION_ID: &str = "third___connection___id";
    const FOURTH_CONNECTION_ID: &str = "fourth-connection-id";

    const FIRST_CONN_ID: &str = "first_conn_id";

    #[test]
    fn test_matches_correctly_exact() {
        let mut should_match = get_should_match();

        let ace = LiteralConnectionIdAccessEntry::new(
            CLIENT_ID,
            should_match.clone());

        for connection_id in &should_match {
            assert!(ace.is_allowed(CLIENT_ID, &connection_id));
        }

        assert!(should_match.remove(FIRST_CONNECTION_ID));

        let ace = LiteralConnectionIdAccessEntry::new(
            CLIENT_ID, should_match.clone());

        for connection_id in should_match {
            assert_eq!(ace.is_allowed(CLIENT_ID, &connection_id),
                       !connection_id.eq(FIRST_CONNECTION_ID));
        }
    }

    #[test]
    fn test_matches_by_patterns() {
        let mut patterns = HashSet::new();

        patterns.insert(WildcardPattern::parse("*connection-id").unwrap());
        patterns.insert(WildcardPattern::parse("*connection*").unwrap());
        patterns.insert(WildcardPattern::parse("first*").unwrap());

        let ace = WildcardPatternConnectionIdAccessEntry::new(
            CLIENT_ID, patterns);

        for connection_id in get_should_match() {
            assert!(ace.is_allowed(CLIENT_ID, &connection_id));
        }
    }

    #[test]
    fn test_does_not_match_on_client_id_mismatch() {
        let mut patterns = HashSet::new();

        patterns.insert(WildcardPattern::parse("*connection-id").unwrap());
        patterns.insert(WildcardPattern::parse("*connection*").unwrap());
        patterns.insert(WildcardPattern::parse("first*").unwrap());


        let should_match = get_should_match();

        let wildcard_ace = WildcardPatternConnectionIdAccessEntry::new(
            NOT_CLIENT_ID, patterns);
        let literal_ace = LiteralConnectionIdAccessEntry::new(
            NOT_CLIENT_ID, should_match.clone());

        for connection_id in &should_match {
            assert!(!wildcard_ace.is_allowed(CLIENT_ID, &connection_id));
            assert!(!literal_ace.is_allowed(CLIENT_ID, &connection_id));
        }
    }

    #[test]
    fn empty_never_matches() {
        let wildcard_ace = WildcardPatternConnectionIdAccessEntry::new(
            CLIENT_ID, HashSet::new());

        let literal_ace = LiteralConnectionIdAccessEntry::new(
            CLIENT_ID,
            HashSet::new());

        for connection_id in get_should_match() {
            assert!(!wildcard_ace.is_allowed(CLIENT_ID, &connection_id));
            assert!(!literal_ace.is_allowed(CLIENT_ID, &connection_id));
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
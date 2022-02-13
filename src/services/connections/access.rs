use std::collections::HashSet;
use crate::models::trie::Trie;

struct ConnectionIdAccessControlEntry {

    client_id: String,
    ace_trie: Trie,
    raw_values: HashSet<String>

}

impl ConnectionIdAccessControlEntry {

    fn matches(&self,
               client_id: &str,
               connection_id: &str) -> bool {
        if !client_id.eq(&self.client_id) {
            return false;
        }

        self.ace_trie.matches(connection_id)
    }

}









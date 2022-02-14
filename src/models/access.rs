use std::collections::HashSet;

use crate::models::wildcards::WildcardPattern;

/// If you can't fit the policy in 20 patterns,
/// you may have other problems than this const.
const MAX_PATTERNS: usize = 20;

struct ConnectionIdAccessControlEntry {

    client_id: String,
    patterns: HashSet<WildcardPattern>

}

impl ConnectionIdAccessControlEntry {

    pub fn matches(&self,
               client_id: &str,
               connection_id: &str) -> bool {
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

    pub fn add(&mut self, pattern: WildcardPattern) -> Result<bool, ()> {
        if self.patterns.len() >= MAX_PATTERNS {
            return Err(());
        }

        Ok(self.patterns.insert(pattern))
    }

    pub fn remove(&mut self, pattern: &WildcardPattern) -> bool {
        self.patterns.remove(pattern)
    }

}

#[cfg(test)]
mod tests {



}
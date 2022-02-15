use std::collections::HashSet;

use crate::models::wildcards::WildcardPattern;

const MAX_NUM_PATTERNS: usize = 40;

struct ConnectionIdAccessControlEntry {

    client_id: String,
    version: u32,
    exact_values: HashSet<String>,
    patterns: HashSet<WildcardPattern>

}

impl ConnectionIdAccessControlEntry {

    pub fn matches(&self,
               client_id: &str,
               connection_id: &str) -> bool {
        if !client_id.eq(&self.client_id) {
            return false;
        }

        if self.exact_values.contains(connection_id) {
            return true;
        }

        for pattern in &self.patterns {
            if pattern.matches(connection_id) {
                return true;
            }
        }

        false
    }

    pub fn add_pattern(&mut self, pattern: WildcardPattern) -> Result<bool, ()> {
        if self.patterns.len() >= MAX_NUM_PATTERNS {
            return Err(());
        }

        Ok(self.patterns.insert(pattern))
    }

    pub fn remove_pattern(&mut self, pattern: &WildcardPattern) -> bool {
        self.patterns.remove(pattern)
    }

}

#[cfg(test)]
mod tests {



}
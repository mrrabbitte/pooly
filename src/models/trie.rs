use chacha20poly1305::aead::AeadMut;

use crate::models::trie::TrieError::{ContainsMoreThanOneStarInSequence, ContainsNonAscii, ContainsWhitespace, EmptyString};

pub struct Trie {

    children: Vec<TrieNode>,
    min_length: usize,
    has_kleene_star: bool

}

impl Trie {

    pub(crate) fn matches(&self,
                          value: &str) -> bool {
        if !self.has_kleene_star && value.len() < self.min_length {
            return false;
        }

        for child in &self.children {
            if child.do_match(value, 0) {
                return true;
            }
        }

        false
    }

    fn new(children: Vec<TrieNode>) -> Trie {
        let min_length = TrieNode::compute_min_length_for_all(&children);
        let has_kleene_star = TrieNode::has_any_kleene_star(&children);

        Trie {
            children,
            min_length,
            has_kleene_star
        }
    }

    fn parse(trimmed: &str) -> Trie {
        let split = trimmed.split('*');

        let mut current = Option::None;

        for (i, piece) in split.rev().enumerate() {
            if i % 2  == 1 {
                let children = Trie::wrap(current);

                current = Some(TrieNode::AnyCharsNode(AnyCharsNode::new(children)));
            } else if !piece.is_empty() {
                let children = Trie::wrap(current);

                current = Some(TrieNode::SubstringNode(
                    SubstringNode::new(piece.into(), children)));
            }
        }

        Trie::new(vec![])
    }

    fn wrap(node_maybe: Option<TrieNode>) -> Vec<TrieNode> {
        node_maybe.map(|node| vec![node]).unwrap_or(Vec::new())
    }

}

pub enum TrieError {

    ContainsNonAscii,
    ContainsMoreThanOneStarInSequence,
    ContainsWhitespace,
    EmptyString,

}

impl TryFrom<String> for Trie {
    type Error = TrieError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(EmptyString);
        }

        if !trimmed.is_ascii() {
            return Err(ContainsNonAscii);
        }

        if trimmed.contains(|c: char| c.is_ascii_whitespace()) {
            return Err(ContainsWhitespace);
        }

        if trimmed.contains("**") {
            return Err(ContainsMoreThanOneStarInSequence)
        }

        Ok(Trie::parse(trimmed))
    }
}

impl TryFrom<&str> for Trie {
    type Error = TrieError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_owned().try_into()
    }
}

enum TrieNode {

    SubstringNode(SubstringNode),
    AnyCharsNode(AnyCharsNode)

}

impl TrieNode {

    #[inline]
    fn compute_min_length_for_all(children: &Vec<TrieNode>) -> usize {
        children.iter().map(TrieNode::get_min_length).min().unwrap_or(0)
    }

    #[inline]
    fn has_any_kleene_star(children: &Vec<TrieNode>) -> bool {
        for child in children {
            if child.has_kleene_star() {
                return true;
            }
        }

        false
    }

}

trait Matchable {

    fn matches(&self, value:&str, start_idx: usize) -> bool {
        if !self.has_kleene_star() && self.get_min_length() > value[start_idx..].len() {
            return false;
        }

        self.do_match(value, start_idx)
    }

    fn do_match(&self,
                value: &str,
                start_idx: usize) -> bool;

    fn get_min_length(&self) -> usize;

    fn has_kleene_star(&self) -> bool;
}

impl Matchable for TrieNode {
    fn do_match(&self,
                value: &str,
                start_idx: usize) -> bool {
        match self {
            TrieNode::SubstringNode(node) => node.do_match(value, start_idx),
            TrieNode::AnyCharsNode(node) => node.do_match(value, start_idx)
        }
    }

    fn get_min_length(&self) -> usize {
        match self {
            TrieNode::SubstringNode(node) => node.get_min_length(),
            TrieNode::AnyCharsNode(node) => node.get_min_length(),
        }
    }

    fn has_kleene_star(&self) -> bool {
        match self {
            TrieNode::SubstringNode(node) => node.has_kleene_star(),
            TrieNode::AnyCharsNode(node) => node.has_kleene_star(),
        }
    }
}

struct SubstringNode {

    substring: String,
    children: Vec<TrieNode>,
    min_length: usize,
    has_kleene_star: bool

}

impl SubstringNode {

    fn new(substring: String,
           children: Vec<TrieNode>) -> SubstringNode {
        let min_length =
            substring.len() + TrieNode::compute_min_length_for_all(&children);
        let has_kleene_star = TrieNode::has_any_kleene_star(&children);

        SubstringNode {
            substring,
            children,
            min_length,
            has_kleene_star
        }
    }

}

impl Matchable for SubstringNode {
    fn do_match(&self,
                value: &str,
                start_idx: usize) -> bool {
        let substring_len = self.substring.len();

        if start_idx >= value.len() {
            return false;
        }

        if !value[start_idx..substring_len].eq(&self.substring) {
            return false;
        }

        let next_idx = start_idx + substring_len;

        for child in &self.children {
            if child.matches(value, next_idx) {
                return true;
            }
        }

        false
    }

    fn get_min_length(&self) -> usize {
        self.min_length
    }

    fn has_kleene_star(&self) -> bool {
        self.has_kleene_star
    }
}

struct AnyCharsNode {

    children: Vec<TrieNode>,
    min_length: usize

}

impl AnyCharsNode {

    fn new(children: Vec<TrieNode>) -> AnyCharsNode {
        let min_length = TrieNode::compute_min_length_for_all(&children);

        AnyCharsNode {
            children,
            min_length
        }
    }

}

impl Matchable for AnyCharsNode {
    fn do_match(&self,
                value: &str,
                start_idx: usize) -> bool {
        if self.children.is_empty() && start_idx >= value.len() {
            return true;
        }

        let value_len = value.len();

        for i in start_idx..value_len {
            for child in &self.children {
                if child.matches(value, i) {
                    return true;
                }
            }
        }

        false
    }

    fn get_min_length(&self) -> usize {
        self.min_length
    }

    fn has_kleene_star(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::models::trie::{Trie, TrieError};

    #[test]
    fn test_returns_errors_on_invalid_inputs() {
        assert!(matches!(try_build_trie(""), Err(TrieError::EmptyString)));

        assert!(matches!(try_build_trie("a lf"), Err(TrieError::ContainsWhitespace)));
        assert!(matches!(try_build_trie("a  lf"), Err(TrieError::ContainsWhitespace)));
        assert!(matches!(try_build_trie("a l f"), Err(TrieError::ContainsWhitespace)));

        assert!(matches!(try_build_trie("ą ł f"), Err(TrieError::ContainsNonAscii)));

        assert!(matches!(try_build_trie("a*l**lf"), Err(TrieError::ContainsMoreThanOneStarInSequence)));
        assert!(matches!(try_build_trie("al***lf"), Err(TrieError::ContainsMoreThanOneStarInSequence)));
    }

    #[test]
    fn test_builds_from_correct_input() {
        assert!(matches!(try_build_trie("alf*"), Ok(_)));
        assert!(matches!(try_build_trie("*alf*loves*cats*"), Ok(_)));
        assert!(matches!(try_build_trie("*alf_loves_eating_cats*"), Ok(_)));
    }

    fn try_build_trie(val: &str) -> Result<Trie, TrieError> {
        val.try_into()
    }

}

use serde::{Deserialize, Serialize};

use crate::models::errors::WildcardPatternError;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WildcardPattern {

    Any,
    Contains(String),
    EndsWith(String),
    Exact(String),
    StartsWith(String),
    StartsAndEndsWith(String, String)

}

const STAR: &str = "*";

impl WildcardPattern {

    pub fn parse(value: &str) -> Result<WildcardPattern, WildcardPatternError> {
        if value == STAR {
            return Ok(WildcardPattern::Any);
        }

        let num_stars = value.matches(STAR).count();

        match num_stars {
            0 => Ok(WildcardPattern::Exact(value.into())),
            1 => Ok(WildcardPattern::parse_one_star(value)),
            2 => WildcardPattern::parse_two_star(value),
            _ => Err(WildcardPatternError::TooManyStars)
        }
    }

    pub fn matches(&self, target: &str) -> bool {
        match self {
            WildcardPattern::Any => true,
            WildcardPattern::Contains(val) => target.contains(val),
            WildcardPattern::EndsWith(val) => target.ends_with(val),
            WildcardPattern::Exact(val) => target.eq(val),
            WildcardPattern::StartsWith(val) => target.starts_with(val),
            WildcardPattern::StartsAndEndsWith(first, second) =>
                target.starts_with(first) && target.ends_with(second)
        }
    }

    fn parse_one_star(value: &str) -> WildcardPattern {
        if value.ends_with(STAR) {
            return WildcardPattern::StartsWith(value.replace(STAR, ""));
        }

        if value.starts_with(STAR) {
            return WildcardPattern::EndsWith(value.replace(STAR, ""));
        }

        let split: Vec<&str> = value.split(STAR).collect();

        WildcardPattern::StartsAndEndsWith(split[0].into(), split[1].into())
    }

    fn parse_two_star(value: &str) -> Result<WildcardPattern, WildcardPatternError> {
        if value.starts_with(STAR) && value.ends_with(STAR) {
            return Ok(WildcardPattern::Contains(value.replacen(STAR, "", 2)));
        }

        Err(WildcardPatternError::UnsupportedPattern)
    }

}

#[cfg(test)]
mod tests {
    use crate::models::errors::WildcardPatternError;
    use crate::models::wildcards::WildcardPattern;

    #[test]
    fn test_matches_any_correctly() {
        check(
            &WildcardPattern::Any,
            vec!["alf-loves-cats", "alfcats", "alf+cats", "", "something"],
            vec![]);
    }

    #[test]
    fn test_matches_exact_correctly() {
        check(
            &WildcardPattern::Exact("alfie".into()),
            vec!["alfie"],
            vec!["alf-loves-cats", "alfcats", "alf+cats", "", "something"]);
    }

    #[test]
    fn test_matches_starts_and_ends_with_correctly() {
        check(
            &WildcardPattern::StartsAndEndsWith("alf".into(), "cats".into()),
            vec!["alf-loves-cats", "alfcats", "alf+cats"],
            vec!["alf_loves_ats", "", "cats", "not-only-alf-loves-cats"]);
    }

    #[test]
    fn test_matches_ends_with_correctly() {
        check(
            &WildcardPattern::EndsWith("alf".into()),
            vec!["cats_dont_like_alf", "alf-loves-alf", "alf"],
            vec!["alf_loves_cats", "", "cats", "not-only-alf-loves-cats"]);
    }

    #[test]
    fn test_matches_starts_with_correctly() {
        check(
            &WildcardPattern::StartsWith("alf".into()),
            vec!["alf_loves_cats", "alf-loves-eating-cats", "alf"],
            vec!["al", "", "cats", "not-only-alf-loves-cats"]);
    }

    #[test]
    fn test_matches_contains_correctly() {
        check(
            &WildcardPattern::Contains("alf".into()),
            vec!["alf_loves_cats", "not-only-alf-loves-cats", "alf"],
            vec!["al", "", "cats"]);
    }

    #[test]
    fn test_builds_wildcard_patterns_correctly() {
        check_ok("*", WildcardPattern::Any);
        check_ok("alf_loves*cats",
                 WildcardPattern::StartsAndEndsWith(
                     "alf_loves".into(), "cats".into()));
        check_ok("*alf*", WildcardPattern::Contains("alf".into()));
        check_ok("alf_loves_cats", WildcardPattern::Exact("alf_loves_cats".into()));
        check_ok("alf*", WildcardPattern::StartsWith("alf".into()));
        check_ok("*alf", WildcardPattern::EndsWith("alf".into()));
    }

    #[test]
    fn test_returns_err_on_illegal_patterns() {
        check_err("*alf*loves*cats*", WildcardPatternError::TooManyStars);
        check_err("*alf*loves*cats", WildcardPatternError::TooManyStars);

        check_err("*alf*loves_cats", WildcardPatternError::UnsupportedPattern);
        check_err("**alf", WildcardPatternError::UnsupportedPattern);
    }

    fn check_err(value: &str,
                 expected_err: WildcardPatternError) {
        assert!(matches!(WildcardPattern::parse(value), Err(err) if err == expected_err));
    }

    fn check_ok(value: &str, expected: WildcardPattern) {
        assert!(matches!(WildcardPattern::parse(value), Ok(ok) if ok == expected));
    }

    fn check(pattern: &WildcardPattern,
             should_match: Vec<&str>,
             should_not_match: Vec<&str>) {
        for val in should_match {
            assert!(pattern.matches(val));
        }

        for val in should_not_match {
            assert!(!pattern.matches(val));
        }
    }

}
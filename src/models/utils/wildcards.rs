use serde::{Deserialize, Serialize};

use crate::models::errors::WildcardPatternError;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub enum WildcardPattern {

    Any,
    Contains {
        pattern: String,
        contains: String
    },
    EndsWith {
        pattern: String,
        ends_with: String
    },
    StartsWith {
        pattern: String,
        starts_with: String
    },
    StartsAndEndsWith{
        pattern: String,
        starts_with: String,
        ends_with: String
    }

}

const STAR: &str = "*";

impl WildcardPattern {

    pub fn matches(&self, target: &str) -> bool {
        match self {
            WildcardPattern::Any => true,
            WildcardPattern::Contains { pattern: _, contains } =>
                target.contains(contains),
            WildcardPattern::EndsWith { pattern: _, ends_with } =>
                target.ends_with(ends_with),
            WildcardPattern::StartsWith { pattern: _, starts_with } =>
                target.starts_with(starts_with),
            WildcardPattern::StartsAndEndsWith { pattern: _, starts_with, ends_with } =>
                target.starts_with(starts_with) && target.ends_with(ends_with)
        }
    }

    pub fn parse(value: &str) -> Result<WildcardPattern, WildcardPatternError> {
        if value == STAR {
            return Ok(WildcardPattern::Any);
        }

        let num_stars = value.matches(STAR).count();

        match num_stars {
            0 => Err(WildcardPatternError::NoStars),
            1 => Ok(WildcardPattern::parse_one_star(value)),
            2 => WildcardPattern::parse_two_star(value),
            _ => Err(WildcardPatternError::TooManyStars)
        }
    }

    fn parse_one_star(value: &str) -> WildcardPattern {
        if value.ends_with(STAR) {
            return WildcardPattern::StartsWith {
                pattern: value.into(),
                starts_with: value.replace(STAR, "")
            };
        }

        if value.starts_with(STAR) {
            return WildcardPattern::EndsWith {
                pattern: value.into(),
                ends_with: value.replace(STAR, "")
            };
        }

        let split: Vec<&str> = value.split(STAR).collect();

        WildcardPattern::StartsAndEndsWith {
            pattern: value.into(),
            starts_with: split[0].into(),
            ends_with: split[1].into()
        }
    }

    fn parse_two_star(value: &str) -> Result<WildcardPattern, WildcardPatternError> {
        if value.starts_with(STAR) && value.ends_with(STAR) {
            return Ok(
                WildcardPattern::Contains {
                    pattern: value.into(),
                    contains: value.replacen(STAR, "", 2)
                }
            );
        }

        Err(WildcardPatternError::UnsupportedPattern)
    }

}

impl TryFrom<String> for WildcardPattern {
    type Error = WildcardPatternError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        WildcardPattern::parse(&value)
    }
}

impl Into<String> for WildcardPattern {
    fn into(self) -> String {
        match self {
            WildcardPattern::Any => STAR.into(),
            WildcardPattern::Contains { pattern, contains: _ } => pattern,
            WildcardPattern::EndsWith { pattern, ends_with: _ } => pattern,
            WildcardPattern::StartsWith { pattern, starts_with: _ } => pattern,
            WildcardPattern::StartsAndEndsWith { pattern, starts_with: _, ends_with: _ } => pattern
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::errors::WildcardPatternError;
    use crate::models::utils::wildcards::WildcardPattern;

    #[test]
    fn test_matches_any_correctly() {
        check(
            &WildcardPattern::Any,
            vec!["alf-loves-cats", "alfcats", "alf+cats", "", "something"],
            vec![]);
    }

    #[test]
    fn test_matches_starts_and_ends_with_correctly() {
        check(
            &WildcardPattern::StartsAndEndsWith {
                pattern: "alf*cats".into(),
                starts_with: "alf".into(),
                ends_with: "cats".into()
            },
            vec!["alf-loves-cats", "alfcats", "alf+cats"],
            vec!["alf_loves_ats", "", "cats", "not-only-alf-loves-cats"]);
    }

    #[test]
    fn test_matches_ends_with_correctly() {
        check(
            &WildcardPattern::EndsWith {
                pattern: "*alf".into(),
                ends_with: "alf".into()
            },
            vec!["cats_dont_like_alf", "alf-loves-alf", "alf"],
            vec!["alf_loves_cats", "", "cats", "not-only-alf-loves-cats"]);
    }

    #[test]
    fn test_matches_starts_with_correctly() {
        check(
            &WildcardPattern::StartsWith {
                pattern: "alf*".into(),
                starts_with: "alf".into()
            },
            vec!["alf_loves_cats", "alf-loves-eating-cats", "alf"],
            vec!["al", "", "cats", "not-only-alf-loves-cats"]);
    }

    #[test]
    fn test_matches_contains_correctly() {
        check(
            &WildcardPattern::Contains {
                pattern: "*alf*".into(),
                contains: "alf".into()
            },
            vec!["alf_loves_cats", "not-only-alf-loves-cats", "alf"],
            vec!["al", "", "cats"]);
    }

    #[test]
    fn test_builds_wildcard_patterns_correctly() {
        check_ok("*", WildcardPattern::Any);
        check_ok("alf_loves*cats", WildcardPattern::StartsAndEndsWith {
            pattern: "alf_loves*cats".into(),
            starts_with: "alf_loves".into(),
            ends_with: "cats".into()
        });
        check_ok("*alf*", WildcardPattern::Contains {
            pattern: "*alf*".into(),
            contains: "alf".into()
        });
        check_ok("alf*", WildcardPattern::StartsWith {
            pattern: "alf*".into(),
            starts_with: "alf".into()
        });
        check_ok("*alf", WildcardPattern::EndsWith {
            pattern: "*alf".into(),
            ends_with: "alf".into()
        });
    }

    #[test]
    fn test_returns_err_on_illegal_patterns() {
        check_err("alf_loves_cats", WildcardPatternError::NoStars);

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
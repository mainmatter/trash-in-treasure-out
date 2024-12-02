use std::sync::LazyLock;

use nutype::nutype;
use regex::Regex;
use validator::{ValidateEmail, ValidateRegex};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Name(String);

#[derive(Debug, thiserror::Error)]
#[error("Error parsing name: {0}")]
pub struct ParseNameError(String);

impl TryFrom<String> for Name {
    type Error = ParseNameError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        static NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[a-z]$").unwrap());

        // See PR: <https://github.com/Keats/validator/pull/361>
        if !s.validate_regex(&NAME_REGEX) {
            return Err(ParseNameError(s));
        }
        Ok(Self(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "String")]
pub struct Email(String);

#[derive(Debug, thiserror::Error)]
#[error("Error parsing email: {0}")]
pub struct ParseEmailError(String);

impl TryFrom<String> for Email {
    type Error = ParseEmailError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if !s.validate_email() {
            return Err(ParseEmailError(s));
        }
        Ok(Self(s))
    }
}

/// The regex is oversimplified and incomplete. Taken from an example at
/// <https://docs.rs/nutype/latest/nutype/#regex-validation>
#[nutype(
    validate(regex = "^[0-9]{3}-[0-9]{3}$"),
    derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)
)]
pub struct PhoneNumber(String);

#[cfg(test)]
mod tests {
    use super::{PhoneNumber, PhoneNumberError};
    use test_case::test_case;

    #[test_case("☎️" => Err(PhoneNumberError::RegexViolated))]
    #[test_case("0612345678" => Err(PhoneNumberError::RegexViolated))]
    #[test_case("123-456" => Ok(()))]
    fn test_parse_phone_number(number: &str) -> Result<(), PhoneNumberError> {
        PhoneNumber::try_new(number)?;
        Ok(())
    }
}

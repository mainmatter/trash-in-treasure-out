use std::sync::LazyLock;

use nutype::nutype;
use regex::Regex;
use validator::{Validate, ValidateRegex, ValidationErrors};

/// This struct's content get's validated using the [`valitator`] crate,
/// specifically whether the name matches a certain regular expression.
/// In this case, we're explicitly calling [`ValidateRegex::validate_regex`] on
/// the [`String`]. We get a lot of flexibility this way.
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

/// This time, we're `#[derive]`ing the `validator::Validate` trait, allowing us
/// to call `Email::validate` within the `TryFrom<String>` application.
/// Sadly, deriving [`validator::Validate`] on tuple structs is
/// not possible at the time of writing, so we're explicitly instrucing serde
/// to use the `TryFrom<String>` and `Into<String>` implementations when
/// (de)serializing `Email`s.
/// 
/// This implementation does a bit more for us, such as formatting an
/// informative error message in case the string doesn't represent a valid
/// email.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, validator::Validate,
)]
#[serde(try_from = "String", into = "String")]
pub struct Email {
    #[validate(email)]
    email: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Error parsing email: {0}")]
pub struct ParseEmailError(String);

impl TryFrom<String> for Email {
    type Error = ValidationErrors;

    fn try_from(email: String) -> Result<Self, Self::Error> {
        let this = Self { email };
        this.validate()?;
        Ok(this)
    }
}

impl From<Email> for String {
    fn from(Email { email }: Email) -> Self {
        email
    }
}

/// This struct is transformed by the `#[nutype]` macro, which, among other
/// things, creates a new module and places the struct inside of that. As we're
/// unable to modify that module, that ensures that we can't instantiate the
/// [`PhoneNumber`] directly, but are forced to apply the validation everywhere.
/// The main limitaion is that [`nutype`] doesn't allow you to implement or
/// derive traits that are not supported by that crate for a good reason:
/// those traits may access or modify the wrapped data without validation.
/// 
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

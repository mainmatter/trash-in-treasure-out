#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "String")]
pub struct Location(String);

impl Location {
    pub fn is_valid_location(location: &str) -> bool {
        const VALID_LOCATIONS: &[&str] = &[
            "Amsterdam Centraal",
            "Paris Nord",
            "Berlin Hbf",
            "London Waterloo",
        ];

        VALID_LOCATIONS.contains(&location)
    }
}

impl TryFrom<String> for Location {
    type Error = ParseLocationError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if !Self::is_valid_location(&s) {
            return Err(ParseLocationError(s));
        }
        Ok(Self(s))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Error parsing location: {0}")]
pub struct ParseLocationError(String);

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

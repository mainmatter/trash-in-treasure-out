#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
/// This one's quite simple. It will only successfully deserialize from
/// "first" or "second", and will also serialize to those values.
pub enum Class {
    First,
    Second,
}

use std::ops::Add;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum DepartureOrArrival {
    Departure(FutureTimestamp),
    Arrival(FutureTimestamp),
}

#[derive(Debug, thiserror::Error)]
pub enum TimeError {
    #[error("Arrival or departure time is in the past")]
    TimeInPast,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "DateTime<Utc>")]
pub struct FutureTimestamp(DateTime<Utc>);

impl TryFrom<DateTime<Utc>> for FutureTimestamp {
    type Error = TimeError;

    fn try_from(dt: DateTime<Utc>) -> Result<Self, Self::Error> {
        if Utc::now() > dt {
            return Err(TimeError::TimeInPast);
        }
        Ok(Self(dt))
    }
}

impl Add<chrono::Duration> for FutureTimestamp {
    type Output = Self;

    fn add(self, rhs: chrono::Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl From<FutureTimestamp> for DateTime<Utc> {
    fn from(FutureTimestamp(dt): FutureTimestamp) -> Self {
        dt
    }
}

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use super::{departure_or_arrival::DepartureOrArrival, location::Location};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TripId(Uuid);

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Trip {
    pub id: TripId,
    pub origin: Location,
    pub destination: Location,
    pub departure: DateTime<Utc>,
    pub arrival: DateTime<Utc>,
}

impl Trip {
    pub fn list_matching(
        origin: Location,
        destiniation: Location,
        time: DepartureOrArrival,
    ) -> Vec<Self> {
        // Come up with some fake trips matching the requirements and
        // that are in the future
        let departure = match time {
            DepartureOrArrival::Departure(t) => t.into(),
            DepartureOrArrival::Arrival(t) => DateTime::<Utc>::from(t) + Duration::hours(-2),
        };

        std::iter::repeat_with(|| Trip {
            id: TripId(Uuid::new_v4()),
            origin: origin.clone(),
            destination: destiniation.clone(),
            departure,
            arrival: departure + Duration::hours(2),
        })
        .enumerate()
        .map(|(i, trip)| Trip {
            departure: trip.departure + Duration::hours(i as i64),
            arrival: trip.arrival + Duration::hours(i as i64),
            ..trip
        })
        .filter(|t| Utc::now() < t.departure)
        .take(10)
        .collect()
    }
}

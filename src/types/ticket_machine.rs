use crate::types::location::Location;
use crate::Result;

use super::{
    class::Class,
    customer_details::{Email, Name, PhoneNumber},
    departure_or_arrival::DepartureOrArrival,
    payment_info::PaymentInfo,
    trip::TripId,
};

#[derive(Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TicketMachine {
    pub origin: Option<Location>,
    pub destination: Option<Location>,
    pub time: Option<DepartureOrArrival>,
    pub trip: Option<TripId>,
    pub class: Option<Class>,
    pub name: Option<Name>,
    pub email: Option<Email>,
    pub phone_number: Option<PhoneNumber>,
    pub payment_info: Option<PaymentInfo>,
}

impl TicketMachine {
    pub fn book(&self) -> Result<()> {
        println!("🚂 Trip booked! Choo choo!");
        Ok(())
    }
}

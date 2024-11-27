#[derive(Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TicketMachine {
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub departure: Option<String>,
    pub arrival: Option<String>,
    pub trip: Option<String>,
    pub class: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub payment_info: Option<String>,
}

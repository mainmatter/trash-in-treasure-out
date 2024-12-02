#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(into = "String")]
pub struct PaymentInfo(String);

impl std::fmt::Display for PaymentInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<SECRET>")
    }
}

impl std::fmt::Debug for PaymentInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(stringify!(PaymentInfo))
            .field(&"<SECRET>")
            .finish()
    }
}

impl From<PaymentInfo> for String {
    fn from(p: PaymentInfo) -> Self {
        p.to_string()
    }
}

impl TryFrom<String> for PaymentInfo {
    type Error = std::convert::Infallible;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        // IRL you'd do input validation here
        Ok(Self(s))
    }
}

#[tokio::test]
async fn test_payment_details_debug_impl() {
    use crate::types::ticket_machine::TicketMachine;
    use std::fmt::Write;

    let ticket_machine = TicketMachine {
        origin: None,
        destination: None,
        time: None,
        trip: None,
        class: None,
        name: None,
        email: None,
        phone_number: None,
        payment_info: Some("💰💰💰".to_owned().try_into().unwrap()),
    };
    let mut dbg_output = String::new();
    write!(&mut dbg_output, "{ticket_machine:?}").unwrap();

    assert_eq!(
        dbg_output,
        concat!(
            "TicketMachine { origin: None, destination: None, time: None, ",
            "trip: None, class: None, name: None, email: None, ",
            r#"phone_number: None, payment_info: Some(PaymentInfo("<SECRET>")) }"#
        )
    )
}

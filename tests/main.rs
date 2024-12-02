use std::{borrow::Cow, sync::LazyLock};

use axum::http::HeaderValue;
use chrono::{Duration, Utc};
use reqwest::Body;
use serde::Serialize;
use serde_json::json;
use takeoff::types::{
    class::Class, departure_or_arrival::DepartureOrArrival, location::Location,
    ticket_machine::TicketMachine, trip::Trip,
};
use test_case::test_case;
use url::Url;

static BASE_URL: LazyLock<Url> = LazyLock::new(|| Url::parse("http://localhost:3000/").unwrap());

fn http_client() -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn send_post_request<Res: serde::de::DeserializeOwned>(
    http_client: &reqwest::Client,
    path: &str,
    body: impl Into<Body>,
) -> Res {
    let res = http_client
        .post(BASE_URL.join(path).expect("Invalid URL"))
        .body(body)
        .send()
        .await
        .expect("Error sending request");

    if let Err(e) = res.error_for_status_ref() {
        panic!(
            "Received error response ({e:?}): '{}'",
            res.text().await.unwrap()
        );
    }

    res.json().await.expect("JSON deserialisation error")
}

async fn send_get_request<Res: serde::de::DeserializeOwned>(
    http_client: &reqwest::Client,
    path: &str,
) -> Res {
    let res = http_client
        .get(BASE_URL.join(path).expect("Invalid URL"))
        .send()
        .await
        .expect("Error sending request");

    if let Err(e) = res.error_for_status_ref() {
        panic!(
            "Received error response ({e:?}): '{}'",
            res.text().await.unwrap()
        );
    }

    res.json().await.expect("JSON deserialisation error")
}

fn json_bytes(s: impl Serialize) -> Cow<'static, [u8]> {
    serde_json::to_vec(&s).unwrap().into()
}

#[test_case(json_bytes("Amsterdam") => panics ""; "Non-existent station")]
#[test_case(json_bytes("🚂-🛒-🛒-🛒") => panics ""; "Emojional roller coaster")]
#[test_case([0xE0, 0x80, 0x80].as_slice().into() => panics "" ; "Non-UTF-8 sequence")]
#[test_case(b"Amsterdam Centraal".into() => panics ""; "Invalid JSON")]
#[test_case(json_bytes("Amsterdam Centraal"); "Valid station")]
#[tokio::test]
async fn test_set_origin(origin: Cow<'static, [u8]>) {
    let origin = origin.to_vec();
    let body: TicketMachine = send_post_request(&http_client(), "/origin", origin.clone()).await;

    let origin: String = serde_json::from_slice(&origin).expect(
        "The request should have failed at this point as `origin` was not valid JSON anyway",
    );
    let origin: Location = origin.try_into().unwrap();

    assert_eq!(
        body,
        TicketMachine {
            origin: Some(origin),
            ..Default::default()
        }
    )
}

#[tokio::test]
async fn test_hiding_payment_details() {
    let client = http_client();
    let origin = json!("Amsterdam Centraal");
    // Set up the session
    let _: TicketMachine =
        send_post_request(&client, "/origin", serde_json::to_vec(&origin).unwrap()).await;

    // Totally not _my_ credit card
    let payment_info = json!({
        "card_number": "1234 5678 9012 3456",
        "cvc": "123",
        "exp": "12/34",
    })
    .to_string();
    // Deserialize into a Value, so that we can skip any input validation on
    // the test side.
    let state: serde_json::Value = send_post_request(
        &client,
        "/book_trip",
        serde_json::to_vec(&payment_info).unwrap(),
    )
    .await;

    assert_eq!(state["payment_info"], "<SECRET>");
}

enum DepartureOrArrivalBytes {
    Departure(Cow<'static, [u8]>),
    Arrival(Cow<'static, [u8]>),
}

#[test_case(
    json_bytes("Amsterdam Centraal"),
    json_bytes("London Waterloo"),
    DepartureOrArrivalBytes::Departure(json_bytes(json!(Utc::now() + Duration::minutes(30)))),
    None,
    json_bytes(Class::First),
    json_bytes("Henk"),
    json_bytes("fake@example.com"),
    json_bytes("123-456"),
    json_bytes(serde_json::to_string(&json!({
        "card_number": "1234 5678 9012 3456",
        "cvc": "123",
        "exp": "12/34",
    })).unwrap())
    ; "Valid flow with departure time")]
#[test_case(
    json_bytes("Amsterdam Centraal"),
    json_bytes("London Waterloo"),
    DepartureOrArrivalBytes::Arrival(json_bytes(json!(Utc::now() + Duration::minutes(30)))),
    None,
    json_bytes(Class::Second),
    json_bytes("Henk"),
    json_bytes("fake@example.com"),
    json_bytes("123-456"),
    json_bytes(serde_json::to_string(&json!({
        "card_number": "1234 5678 9012 3456",
        "cvc": "123",
        "exp": "12/34",
    })).unwrap())
    ; "Valid flow with arrival time")]
#[tokio::test]
async fn complete_flow(
    origin: Cow<'static, [u8]>,
    destination: Cow<'static, [u8]>,
    time: DepartureOrArrivalBytes,
    trip: Option<Cow<'static, [u8]>>,
    class: Cow<'static, [u8]>,
    name: Cow<'static, [u8]>,
    email: Cow<'static, [u8]>,
    phone_number: Cow<'static, [u8]>,
    payment_details: Cow<'static, [u8]>,
) {
    let client = http_client();
    let state: TicketMachine = send_post_request(&client, "/origin", origin.to_vec()).await;
    let expected_origin = Some(serde_json::from_slice(&origin).unwrap());
    assert_eq!(
        state,
        TicketMachine {
            origin: expected_origin.clone(),
            ..Default::default()
        }
    );

    let state: TicketMachine =
        send_post_request(&client, "/destination", destination.to_vec()).await;
    let expected_destination = Some(serde_json::from_slice(&destination).unwrap());
    assert_eq!(
        state,
        TicketMachine {
            origin: expected_origin.clone(),
            destination: expected_destination.clone(),
            ..Default::default()
        }
    );

    let expected_time = match time {
        DepartureOrArrivalBytes::Departure(departure) => {
            let state: TicketMachine =
                send_post_request(&client, "/departure", departure.to_vec()).await;
            let expected_departure = Some(DepartureOrArrival::Departure(
                serde_json::from_slice(&departure).unwrap(),
            ));
            assert_eq!(
                state,
                TicketMachine {
                    origin: expected_origin.clone(),
                    destination: expected_destination.clone(),
                    time: expected_departure.clone(),
                    ..Default::default()
                }
            );
            expected_departure
        }
        DepartureOrArrivalBytes::Arrival(arrival) => {
            let state: TicketMachine =
                send_post_request(&client, "/arrival", arrival.to_vec()).await;
            let expected_arrival = Some(DepartureOrArrival::Arrival(
                serde_json::from_slice(&arrival).unwrap(),
            ));
            assert_eq!(
                state,
                TicketMachine {
                    origin: expected_origin.clone(),
                    destination: expected_destination.clone(),
                    time: expected_arrival.clone(),
                    ..Default::default()
                }
            );
            expected_arrival
        }
    };

    let trips: Vec<Trip> = send_get_request(&client, "/trips").await;
    let trip = trip.unwrap_or(serde_json::to_vec(&trips[0].id).unwrap().into());
    let state: TicketMachine = send_post_request(&client, "/trip", trip.to_vec()).await;
    let expected_trip = Some(serde_json::from_slice(&trip).unwrap());
    assert_eq!(
        state,
        TicketMachine {
            origin: expected_origin.clone(),
            destination: expected_destination.clone(),
            time: expected_time.clone(),
            trip: expected_trip.clone(),
            ..Default::default()
        }
    );

    let state: TicketMachine = send_post_request(&client, "/class", class.to_vec()).await;
    let expected_class = Some(serde_json::from_slice(&class).unwrap());
    assert_eq!(
        state,
        TicketMachine {
            origin: expected_origin.clone(),
            destination: expected_destination.clone(),
            time: expected_time.clone(),
            trip: expected_trip.clone(),
            class: expected_class.clone(),
            ..Default::default()
        }
    );

    let state: TicketMachine = send_post_request(&client, "/name", name.to_vec()).await;
    let expected_name = Some(serde_json::from_slice(&name).unwrap());
    assert_eq!(
        state,
        TicketMachine {
            origin: expected_origin.clone(),
            destination: expected_destination.clone(),
            time: expected_time.clone(),
            trip: expected_trip.clone(),
            class: expected_class.clone(),
            name: expected_name.clone(),
            ..Default::default()
        }
    );

    let state: TicketMachine = send_post_request(&client, "/email", email.to_vec()).await;
    let expected_email = Some(serde_json::from_slice(&email).unwrap());
    assert_eq!(
        state,
        TicketMachine {
            origin: expected_origin.clone(),
            destination: expected_destination.clone(),
            time: expected_time.clone(),
            trip: expected_trip.clone(),
            class: expected_class.clone(),
            name: expected_name.clone(),
            email: expected_email.clone(),
            ..Default::default()
        }
    );

    let state: TicketMachine =
        send_post_request(&client, "/phone_number", phone_number.to_vec()).await;
    let expected_phone_number = Some(serde_json::from_slice(&phone_number).unwrap());
    assert_eq!(
        state,
        TicketMachine {
            origin: expected_origin.clone(),
            destination: expected_destination.clone(),
            time: expected_time.clone(),
            trip: expected_trip.clone(),
            class: expected_class.clone(),
            name: expected_name.clone(),
            email: expected_email.clone(),
            phone_number: expected_phone_number.clone(),
            ..Default::default()
        }
    );

    let _: TicketMachine = send_post_request(&client, "/book_trip", payment_details.to_vec()).await;
}

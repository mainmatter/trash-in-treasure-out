use std::{borrow::Cow, sync::LazyLock};

use axum::http::HeaderValue;
use reqwest::Body;
use takeoff::types::{location::Location, ticket_machine::TicketMachine};
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

fn json_string_bytes(s: &str) -> Cow<'static, [u8]> {
    serde_json::to_vec(s).unwrap().into()
}

#[test_case(json_string_bytes("Amsterdam") => panics ""; "Non-existent station")]
#[test_case(json_string_bytes("🚂-🛒-🛒-🛒") => panics ""; "Emojional roller coaster")]
#[test_case([0xE0, 0x80, 0x80].as_slice().into() => panics "" ; "Non-UTF-8 sequence")]
#[test_case(b"Amsterdam Centraal".into() => panics ""; "Invalid JSON")]
#[test_case(json_string_bytes("Amsterdam Centraal"); "Valid station")]
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

use std::sync::LazyLock;

use axum::http::HeaderValue;
use reqwest::Body;
use takeoff::ticket_machine::TicketMachine;
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

#[tokio::test]
async fn test_set_origin() {
    let body: TicketMachine = send_post_request(&http_client(), "/origin", "Amsterdam").await;
    assert_eq!(
        body,
        TicketMachine {
            origin: Some("Amsterdam".to_owned()),
            ..Default::default()
        }
    )
}

use axum::{
    routing::{get, post},
    Json,
};
use axum_session::{SessionConfig, SessionLayer, SessionNullSessionStore, SessionStore};
use error::Error;
use session::{Session, SessionExt};
use ticket_machine::TicketMachine;
use tokio::net::TcpListener;

pub mod error;
pub mod session;
pub mod ticket_machine;

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn is_valid_location(location: &str) -> bool {
    const VALID_LOCATIONS: &[&str] = &[
        "Amsterdam Centraal",
        "Paris Nord",
        "Berlin Hbf",
        "London Waterloo",
    ];

    VALID_LOCATIONS.contains(&location)
}

pub async fn run() -> Result<()> {
    // Setup router
    let router = axum::Router::new()
        .route("/origin", post(set_origin))
        .route("/destination", post(set_destination))
        .route("/departure", post(set_departure))
        .route("/arrival", post(set_arrival))
        .route("/trips", get(list_trips))
        .route("/trip", post(set_trip))
        .route("/class", post(set_class))
        .route("/name", post(set_name))
        .route("/email", post(set_email))
        .route("/phone_number", post(set_phone_number))
        .route("/book_trip", post(book_trip));

    // Create in-memory session store
    let session_store: SessionNullSessionStore = SessionStore::new(None, SessionConfig::default())
        .await
        .unwrap();

    // Stitch them together
    let app = router
        .layer(SessionLayer::new(session_store))
        .into_make_service();

    // Aand serve!
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn set_origin(session: Session, origin: String) -> Result<Json<TicketMachine>> {
    if !is_valid_location(&origin) {
        return Err(Error::BadRequest("Invalid origin!"));
    }

    Ok(session.get_or_init_state(|s| {
        s.origin = Some(origin);
    }))
    .map(Json)
}

async fn set_destination(session: Session, destination: String) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.destination = Some(destination))
        .ok_or(Error::BadRequest("Set origin first"))
        .map(Json)
}

async fn set_departure(session: Session, departure: String) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.departure = Some(departure))
        .ok_or(Error::BadRequest("Set destination first"))
        .map(Json)
}

async fn set_arrival(session: Session, arrival: String) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.arrival = Some(arrival))
        .ok_or(Error::BadRequest("Set destination first"))
        .map(Json)
}

async fn list_trips(session: Session) -> Result<()> {
    let _state = session
        .try_get_state()
        .ok_or(Error::BadRequest("Set trip details first"))?;
    Ok(())
}

async fn set_trip(session: Session, trip: String) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.trip = Some(trip))
        .ok_or(Error::BadRequest("Set departure or arrival time first"))
        .map(Json)
}

async fn set_class(session: Session, class: String) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.class = Some(class))
        .ok_or(Error::BadRequest("Select a trip first"))
        .map(Json)
}

async fn set_name(session: Session, name: String) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.name = Some(name))
        .ok_or(Error::BadRequest("Set class first"))
        .map(Json)
}

async fn set_email(session: Session, email: String) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.email = Some(email))
        .ok_or(Error::BadRequest("Set name first"))
        .map(Json)
}

async fn set_phone_number(session: Session, phone_number: String) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.phone_number = Some(phone_number))
        .ok_or(Error::BadRequest("Set email first"))
        .map(Json)
}

async fn book_trip(session: Session, payment_info: String) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| {
            println!("🚂 Trip booked! Choo choo!");
            s.payment_info = Some(payment_info)
        })
        .ok_or(Error::BadRequest("Set phone_number first"))
        .map(Json)
}

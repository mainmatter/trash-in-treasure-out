use axum::{
    routing::{get, post},
    Json,
};
use axum_session::{SessionConfig, SessionLayer, SessionNullSessionStore, SessionStore};
use error::Error;
use session::{Session, SessionExt};

use tokio::net::TcpListener;
use types::{
    class::Class,
    customer_details::{Email, Name, PhoneNumber},
    departure_or_arrival::{DepartureOrArrival, FutureTimestamp},
    location::Location,
    payment_info::PaymentInfo,
    ticket_machine::TicketMachine,
    trip::{Trip, TripId},
};

pub mod error;
pub mod session;
pub mod types;

pub type Result<T> = std::result::Result<T, error::Error>;

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

async fn set_origin(session: Session, Json(origin): Json<Location>) -> Result<Json<TicketMachine>> {
    Ok(session.get_or_init_state(|s| {
        s.origin = Some(origin);
    }))
    .map(Json)
}

async fn set_destination(
    session: Session,
    Json(destination): Json<Location>,
) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.destination = Some(destination))
        .ok_or(Error::BadRequest("Set origin first"))
        .map(Json)
}

async fn set_departure(
    session: Session,
    Json(departure): Json<FutureTimestamp>,
) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.time = Some(DepartureOrArrival::Departure(departure)))
        .ok_or(Error::BadRequest("Set destination first"))
        .map(Json)
}

async fn set_arrival(
    session: Session,
    Json(arrival): Json<FutureTimestamp>,
) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.time = Some(DepartureOrArrival::Arrival(arrival)))
        .ok_or(Error::BadRequest("Set destination first"))
        .map(Json)
}

async fn list_trips(session: Session) -> Result<Json<Vec<Trip>>> {
    let state = session
        .try_get_state()
        .ok_or(Error::BadRequest("Set trip details first"))?;

    let ((origin, destination), time) = state
        .origin
        .zip(state.destination)
        .zip(state.time)
        .ok_or(Error::BadRequest("Trip details incomplete"))?;

    Ok(Trip::list_matching(origin, destination, time)).map(Json)
}

async fn set_trip(session: Session, Json(trip_id): Json<TripId>) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.trip = Some(trip_id))
        .ok_or(Error::BadRequest("Set departure or arrival time first"))
        .map(Json)
}

async fn set_class(session: Session, Json(class): Json<Class>) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.class = Some(class))
        .ok_or(Error::BadRequest("Select a trip first"))
        .map(Json)
}

async fn set_name(session: Session, Json(name): Json<Name>) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.name = Some(name))
        .ok_or(Error::BadRequest("Set class first"))
        .map(Json)
}

async fn set_email(session: Session, Json(email): Json<Email>) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.email = Some(email))
        .ok_or(Error::BadRequest("Set name first"))
        .map(Json)
}

async fn set_phone_number(
    session: Session,
    Json(phone_number): Json<PhoneNumber>,
) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| s.phone_number = Some(phone_number))
        .ok_or(Error::BadRequest("Set email first"))
        .map(Json)
}

async fn book_trip(
    session: Session,
    Json(payment_info): Json<PaymentInfo>,
) -> Result<Json<TicketMachine>> {
    session
        .update_state(|s| {
            s.payment_info = Some(payment_info);
        })
        .ok_or(Error::BadRequest("Set phone_number first"))
        .map(|t| {
            t.book()?;
            Ok(t)
        })?
        .map(Json)
}

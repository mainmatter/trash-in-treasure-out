#[tokio::main]
pub async fn main() -> Result<(), takeoff::error::Error> {
    takeoff::run().await
}

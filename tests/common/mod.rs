use sqlx::postgres::PgPoolOptions;
use std::sync::Once;
use tokio::net::TcpListener;
use tracing::info;

static INIT: Once = Once::new();

pub async fn setup() {
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to port 8080");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://myuser:mypass@localhost/mydb")
        .await
        .expect("couldn't connect to the database");
    INIT.call_once(move || {
        tokio::spawn(blog::run(listener, pool));
        info!("--> finished setup");
    });
}

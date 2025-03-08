use sqlx::postgres::PgPoolOptions;
use std::sync::Once;
use tokio::net::TcpListener;
use tracing::info;
use blog::config::Configuration;

static INIT: Once = Once::new();

pub async fn setup() {
    let config = Configuration::from_env().expect("Could not get config from env");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port))
        .await
        .expect("Failed to bind address");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(config.db.get_connection_string().as_str())
        .await
        .expect("couldn't connect to the database");
    INIT.call_once(move || {
        tokio::spawn(blog::run(listener, pool));
        info!("--> finished setup");
    });
}

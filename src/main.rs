use blog::run;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::*, util::*};

mod config;
use config::Configuration;

#[tokio::main]
async fn main() {
    // app config
    let config = Configuration::from_env().expect("Could not get config from env");
    let port = config.port;
    let address = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(address)
        .await
        .expect("Failed to bind address");

    // logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug");
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            format!(
                "postgres://{}:{}@{}/mydb",
                config.db.user, config.db.pass, config.db.host
            )
            .as_str(),
        )
        .await
        .expect("couldn't connect to the database");

    // start the server
    run(listener, pool).await
}

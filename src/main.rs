
use blog::run;
use std::env;
use tokio::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::*, util::*};

#[tokio::main]
async fn main() {

    // app config
    let port = env::var("PORT").unwrap_or_else(|_| String::from("8080"));
    let address = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(address)
        .await
        .expect("Failed to bind address");

    // logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug");
    }

    /*
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();*/

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://myuser:mypass@localhost/mydb")
        .await
        .expect("couldn't connect to the database");

    // start the server
    run(listener, pool).await
}


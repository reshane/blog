use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let args: Vec<String> = env::args().collect();

    let contents = std::fs::read_to_string(&args[2]).expect("File must exist and be accessible");

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect("postgres://myuser:mypass@localhost/mydb")
        .await
        .expect("Could not create db connection pool");

    let _row: (i32,) = sqlx::query_as(
        "insert into myposts (post_title, post_body) values ($1, $2) returning post_id",
    )
    .bind(&args[1])
    .bind(contents)
    .fetch_one(&pool)
    .await?;

    Ok(())
}

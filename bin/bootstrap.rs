use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let db_host = std::env::var("DB_HOST").expect("DB_HOST env var required but not found");
    let db_user = std::env::var("DB_USER").expect("DB_USER env var required but not found");
    let db_pass = std::env::var("DB_PASS").expect("DB_PASS env var required but not found");
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(
            format!(
                "postgres://{}:{}@{}/mydb", db_user, db_pass, db_host
            ).as_str()
        )
        .await
        .expect("Could not create db connection pool");

    let commands = std::fs::read_to_string("./db/scripts/bootstrap.sql").unwrap();
    let commands = commands.split("-- @COMMAND");
    for command in commands {
        let command = command.trim();
        if !command.is_empty() {
            println!("EXECUTING: [{}]", command);
            let _row = sqlx::query(command).execute(&pool).await?;
        }
    }

    Ok(())
}

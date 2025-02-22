use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect("postgres://myuser:mypass@localhost/mydb")
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

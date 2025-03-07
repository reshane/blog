use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let post_contents =
        std::fs::read_to_string("./bin/post.md").expect("File must exist and be accessible");
    let recipe_contents =
        std::fs::read_to_string("./bin/recipe.md").expect("File must exist and be accessible");

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

    let (_post_id,): (i32,) =
        sqlx::query_as("insert into post (title, body) values ($1, $2) returning id")
            .bind("First Post")
            .bind(post_contents)
            .fetch_one(&pool)
            .await?;

    let (recipe_id,): (i32,) =
        sqlx::query_as("insert into recipe (dish_name, instructions) values ($1, $2) returning id")
            .bind("Roasted Potatoes")
            .bind(recipe_contents)
            .fetch_one(&pool)
            .await?;

    let (ingredient_id,): (i32,) =
        sqlx::query_as("insert into ingredient (name) values ($1) returning id")
            .bind("Potatoes")
            .fetch_one(&pool)
            .await?;

    let _row: (i32,) = sqlx::query_as(
        "insert into recipe_ingredient (recipe_id, ingredient_id) values ($1, $2) returning id",
    )
    .bind(recipe_id)
    .bind(ingredient_id)
    .fetch_one(&pool)
    .await?;

    Ok(())
}

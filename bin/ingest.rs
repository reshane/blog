use sqlx::postgres::PgPoolOptions;
use blog::config::Configuration;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let post_contents =
        std::fs::read_to_string("./bin/post.md").expect("File must exist and be accessible");
    let recipe_contents =
        std::fs::read_to_string("./bin/recipe.md").expect("File must exist and be accessible");

    let config = Configuration::from_env().expect("Could not get config from env");
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(config.db.get_connection_string().as_str())
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

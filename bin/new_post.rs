use sqlx::postgres::PgPoolOptions;
use blog::config::Configuration;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("Error: path to markdown post required");
        return Ok(())
    }
    let file_name = args[1].clone();
    let post_contents =
        std::fs::read_to_string(&file_name).expect("File must exist and be accessible");

    let post_title_snake = &file_name[
        file_name.find("_").expect("File must be prefixed with date")+1
        ..
        file_name.find(".md").expect("File must be markdown format")
    ];

    let post_title = post_title_snake.split("_")
        .map(|word| {
            let first = &mut word[0..1].to_string();
            first.make_ascii_uppercase();
            let rest = &word[1..];
            format!("{}{}", first, rest)
        })
        .collect::<Vec<String>>()
        .join(" ");

    let config = Configuration::from_env().expect("Could not get config from env");
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(config.db.get_connection_string().as_str())
        .await
        .expect("Could not create db connection pool");

    println!("{}", post_title);
    let (_post_id,): (i32,) =
        sqlx::query_as("insert into post (title, body) values ($1, $2) returning id")
            .bind(post_title)
            .bind(post_contents)
            .fetch_one(&pool)
            .await?;
    Ok(())
}

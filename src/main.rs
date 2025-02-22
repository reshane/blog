use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use sqlx::{postgres::PgPool, postgres::PgPoolOptions, types::time::Date, FromRow};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::fmt::{format::FmtSpan, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

#[derive(Template)]
#[template(path = "posts.html")]
pub struct PostTemplate<'a> {
    pub title: &'a str,
    pub post_date: String,
    pub post_body: &'a str,
}

#[derive(FromRow, Debug, Clone)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub publish_date: Date,
    pub body: String,
}

#[derive(FromRow, Debug, Clone)]
pub struct RecipeRow {
    pub id: i32,
    pub dish_name: String,
    pub instructions: String,
}

pub struct Recipe {
    pub dish_name: String,
    pub instructions: String,
    pub ingredients: Vec<String>,
}

#[derive(FromRow, Debug, Clone)]
pub struct Ingredient {
    pub name: String,
}

#[derive(Template)]
#[template(path = "recipes.html")]
pub struct RecipeTemplate<'a> {
    pub title: &'a str,
    pub dish_name: &'a str,
    pub instructions: &'a str,
    pub ingredients: Vec<String>,
}

async fn get_recipe(Path(dish_name): Path<String>, State(state): State<Arc<PgPool>>) -> impl IntoResponse {

    let pool = state.clone();

    let recipe_rows = sqlx::query_as::<_, RecipeRow>(
        "select id, dish_name, instructions from recipe where dish_name = ($1)",
    )
    .bind(dish_name.replace("-", " "))
    .fetch_all(&*pool)
    .await
    .unwrap();

    let recipe_row = recipe_rows[0].clone();
    let recipe_id = recipe_row.id;

    let ingredients = sqlx::query_as::<_, Ingredient>(
        "select name from ingredient left join recipe_ingredient on ingredient.id = recipe_ingredient.ingredient_id where recipe_ingredient.recipe_id = ($1)"
    )
    .bind(recipe_id)
    .fetch_all(&*pool)
    .await
    .unwrap();

    let recipe = Recipe {
        dish_name: recipe_row.dish_name,
        instructions: recipe_row.instructions,
        ingredients: ingredients
            .iter()
            .map(|i| i.name.clone())
            .collect::<Vec<String>>(),
    };

    let template = Some(RecipeTemplate {
        title: &recipe.dish_name,
        dish_name: &recipe.dish_name,
        instructions: &recipe.instructions,
        ingredients: recipe.ingredients,
    });

    match template {
        None => (StatusCode::NOT_FOUND, "404 not found").into_response(),
        Some(template) => match template.render() {
            Ok(html) => Html(html).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Server Error").into_response(),
        },
    }
}

async fn post(
    Path(query_title): Path<String>,
    State(state): State<Arc<PgPool>>,
) -> impl IntoResponse {
    let pool = state.clone();
    let posts =
        sqlx::query_as::<_, Post>("select id, title, publish_date, body from post where title = ($1)")
            .bind(&(query_title.replace("-", " ")))
            .fetch_all(&*pool)
            .await
            .unwrap();

    let post = match posts.len() {
        0 => None,
        _ => Some(posts[0].clone()),
    };

    match post {
        Some(post) => {
            let template = PostTemplate {
                title: &post.title,
                post_date: post.publish_date.to_string(),
                post_body: &post.body,
            }.render();
            match template {
                Ok(html) => Html(html).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Server Error").into_response(),
            }
        },
        None => (StatusCode::NOT_FOUND, "404 not found").into_response(),
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub index_title: String,
    pub index_links: &'a Vec<String>,
    pub recipe_links: &'a Vec<String>,
}

async fn index(State(state): State<Arc<PgPool>>) -> impl IntoResponse {
    let pool = state.clone();
    let mut posts =
        sqlx::query_as::<_, Post>("select id, title, publish_date, body from post")
            .fetch_all(&*pool)
            .await
            .unwrap();

    let rlinks = sqlx::query_as::<_, RecipeRow>("select id, dish_name, instructions from recipe")
        .fetch_all(&*pool)
        .await
        .unwrap()
        .iter()
        .map(|r| format!("{}", r.dish_name.replace(" ", "-")))
        .collect::<Vec<String>>();

    for i in 0..posts.len() {
        posts[i].title = posts[i].title.replace(" ", "-");
    }

    let s = posts;
    let mut plinks: Vec<String> = Vec::new();

    for i in 0..s.len() {
        plinks.push(s[i].title.clone());
    }

    let template = IndexTemplate {
        index_title: String::from("Blog"),
        index_links: &plinks,
        recipe_links: &rlinks,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to render template. Error {}", err),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    let fmt_layer = Layer::default().with_span_events(FmtSpan::CLOSE);
    tracing_subscriber::registry().with(fmt_layer).init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://myuser:mypass@localhost/mydb")
        .await
        .expect("couldn't connect to the database");

    let shared_state = Arc::new(pool);

    let app = Router::new()
        .route("/", get(index))
        .route("/post/{query_title}", get(post))
        .route("/recipe/{id}", get(get_recipe))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("{:<12} - {:?}", "LISTENING", listener.local_addr());
    axum::serve(listener, app).await.unwrap();
}

mod filters {
    pub fn rmdashes(title: &str) -> askama::Result<String> {
        Ok(title.replace("-", " ").into())
    }
}

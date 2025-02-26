use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use sqlx::{postgres::PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;
use tokio::net::TcpListener;

mod error;
mod web;
mod recipe;
mod post;
mod filters;
mod db;

use recipe::{RecipeRow, Recipe, RecipeTemplate};
use post::{Post, PostTemplate};

async fn get_recipe(Path(dish_name): Path<String>, State(state): State<Arc<PgPool>>) -> impl IntoResponse {

    let pool = state.clone();

    let recipe = Recipe::get_by_name(dish_name, pool).await;

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

    let post = Post::get_by_title(query_title, state).await;

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

fn routes_static() -> Router {
    Router::new()
        .nest_service("/assets/", ServeDir::new("./assets"))
}

async fn routes_data() -> Router {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://myuser:mypass@localhost/mydb")
        .await
        .expect("couldn't connect to the database");

    let shared_state = Arc::new(pool);

    Router::new()
        .route("/", get(index))
        .route("/post/{query_title}", get(post))
        .route("/recipe/{id}", get(get_recipe))
        .with_state(shared_state)
}

pub async fn run(listener: TcpListener) {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = Router::new()
        .merge(routes_data().await)
        .merge(web::routes_login::routes())
        .fallback_service(routes_static());

    println!("--> {:<12} - {:?}", "LISTENING", listener.local_addr());
    axum::serve(listener, app).await.unwrap();
}


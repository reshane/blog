use askama::Template;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use sqlx::{FromRow, postgres::PgPoolOptions, types::time::Date};
use std::sync::Arc;

#[derive(Template)]
#[template(path = "posts.html")]
pub struct PostTemplate<'a> {
    pub post_title: &'a str,
    pub post_date: String,
    pub post_body: &'a str,
}

#[derive(FromRow, Debug, Clone)]
pub struct Post {
    pub post_title: String,
    pub post_date: Date,
    pub post_body: String,
}

async fn post(
    Path(query_title): Path<String>,
    State(state): State<Arc<Vec<Post>>>,
) -> impl IntoResponse {
    let mut template = None;

    for i in 0..state.len() {
        if query_title == state[i].post_title {
            template = Some(PostTemplate {
                post_title: &state[i].post_title,
                post_date: state[i].post_date.to_string(),
                post_body: &state[i].post_body,
            });
            break;
        }
    }

    match template {
        None => (StatusCode::NOT_FOUND, "404 not found").into_response(),
        Some(template) => match template.render() {
            Ok(html) => Html(html).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Server Error").into_response(),
        },
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub index_title: String,
    pub index_links: &'a Vec<String>,
}

async fn index(State(state): State<Arc<Vec<Post>>>) -> impl IntoResponse {
    let s = state.clone();
    let mut plinks: Vec<String> = Vec::new();

    for i in 0..s.len() {
        plinks.push(s[i].post_title.clone());
    }

    let template = IndexTemplate {
        index_title: String::from("Blog"),
        index_links: &plinks,
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
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://myuser:mypass@localhost/mydb")
        .await
        .expect("couldn't connect to the database");
    let mut posts =
        sqlx::query_as::<_, Post>("select post_title, post_date, post_body from myposts")
            .fetch_all(&pool)
            .await
            .unwrap();

    for i in 0..posts.len() {
        posts[i].post_title = posts[i].post_title.replace(" ", "-");
    }

    let shared_state = Arc::new(posts);

    let app = Router::new()
        .route("/", get(index))
        .route("/post/{query_title}", get(post))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod filters {
    pub fn rmdashes(title: &str) -> askama::Result<String> {
        Ok(title.replace("-", " ").into())
    }
}

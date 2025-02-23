
use askama::Template;
use sqlx::{postgres::PgPool, types::time::Date, FromRow};
use std::sync::Arc;

#[derive(Template)]
#[template(path = "posts.html")]
pub struct PostTemplate<'a> {
    pub title: &'a str,
    pub post_date: String,
    pub post_body: &'a str,
}

#[derive(FromRow, Debug, Clone)]
pub struct Post {
    #[allow(unused)]
    pub id: i32,
    pub title: String,
    pub publish_date: Date,
    pub body: String,
}

impl Post {
    pub async fn get_by_title(query_title: String, pool: Arc<PgPool>) -> Option<Self> {
        let pool = (*pool).clone();
        let posts =
        sqlx::query_as::<_, Post>("select id, title, publish_date, body from post where title = ($1)")
            .bind(&(query_title.replace("-", " ")))
            .fetch_all(&pool)
            .await
            .unwrap();
        match posts.len() {
            0 => None,
            _ => Some(posts[0].clone()),
        }
    }
}

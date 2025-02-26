use crate::filters;
use askama::Template;
use sqlx::{postgres::PgPool, FromRow};
use std::sync::Arc;

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

impl Recipe {
    pub async fn get_by_name(dish_name: String, pool: Arc<PgPool>) -> Self {
        let pool = (*pool).clone();
        let recipe_rows = sqlx::query_as::<_, RecipeRow>(
            "select id, dish_name, instructions from recipe where dish_name = ($1)",
        )
        .bind(dish_name.replace("-", " "))
        .fetch_all(&pool)
        .await
        .unwrap();

        let recipe_row = recipe_rows[0].clone();
        let recipe_id = recipe_row.id;

        let ingredients = sqlx::query_as::<_, Ingredient>(
            "select name from ingredient left join recipe_ingredient on ingredient.id = recipe_ingredient.ingredient_id where recipe_ingredient.recipe_id = ($1)"
        )
        .bind(recipe_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        Recipe {
            dish_name: recipe_row.dish_name,
            instructions: recipe_row.instructions,
            ingredients: ingredients
                .iter()
                .map(|i| i.name.clone())
                .collect::<Vec<String>>(),
        }
    }
}

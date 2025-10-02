use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, prelude::FromRow};
use std::{collections::BTreeMap, env, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct Todo {
    id: String,
    text: String,
    completed: bool,
}

struct AppState {
    db_pool: SqlitePool,
    handlebars: Handlebars<'static>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let mut handlebars = Handlebars::new();

    handlebars.set_dev_mode(true);

    handlebars
        .register_templates_directory("./templates", DirectorySourceOptions::default())
        .unwrap();

    let app_state = Arc::new(AppState {
        db_pool: pool,
        handlebars,
    });
    let app = Router::new()
        .route("/", get(index))
        .route("/api/todos", get(get_todos).post(create_todo))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("localhost:5000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // If your template does not use any variables, pass an empty map or unit value
    let result = state
        .handlebars
        .render("index", &BTreeMap::<String, String>::new())
        .unwrap();
    Html(result)
}

async fn get_todos(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let todos = sqlx::query_as::<_, Todo>(r#"SELECT * FROM todos"#)
        .fetch_all(&state.db_pool)
        .await
        .unwrap();

    println!("todos {:?}", todos);

    Json(todos)
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateTodo {
    text: String,
}

async fn create_todo(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTodo>,
) -> impl IntoResponse {
    let new_todo = Todo {
        id: Uuid::new_v4().to_string(),
        text: payload.text,
        completed: false,
    };

    sqlx::query!(
        r#"INSERT INTO todos (id, text, completed) VALUES (?1, ?2, ?3)"#,
        new_todo.id,
        new_todo.text,
        new_todo.completed
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    (StatusCode::CREATED, Json(new_todo))
}

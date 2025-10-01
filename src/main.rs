use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, SqlitePool, prelude::FromRow};
use std::env;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct Todo {
    id: String,
    text: String,
    completed: bool,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let todos = sqlx::query!(r#"SELECT * FROM todos"#)
        .fetch_all(&pool)
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(todos_index).post(create_todo))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("localhost:5000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn todos_index(State(SqlitePool): State<SqlitePool>) -> impl IntoResponse {
    let todos = sqlx::query_as::<_, Todo>(r#"SELECT * FROM todos"#)
        .fetch_all(&SqlitePool)
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
    State(pool): State<SqlitePool>,
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
    .execute(&pool)
    .await
    .unwrap();

    (StatusCode::CREATED, Json(new_todo))
}

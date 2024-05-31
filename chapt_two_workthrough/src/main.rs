use axum::{
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use deadpool_postgres::{tokio_postgres::NoTls, Pool};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use tokio_postgres::row::Row;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Question {
    id: Uuid,
    text: String,
    answer: Option<String>,
    source: Option<String>,
}

type DbPool = Pool;

async fn get_question(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let client = pool.get().await.unwrap();
    let row = client
        .query_opt("SELECT * FROM questions WHERE id = $1", &[&id.to_string()])
        .await
        .unwrap();

    match row {
        Some(row) => {
            let question = row_to_question(row);
            let json = serde_json::to_string(&question).unwrap();
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(json))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Question not found"))
            .unwrap(),
    }
}

async fn add_question(
    State(pool): State<DbPool>,
    new_question: Json<Question>,
) -> impl IntoResponse {
    let client = pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO questions (id, question, answer, source) VALUES ($1, $2, $3, $4)",
            &[
                &new_question.id.to_string(),
                &new_question.text,
                &new_question.answer,
                &new_question.source,
            ],
        )
        .await
        .unwrap();
    (StatusCode::OK, Json("Inserted successfully".to_string()))
}

async fn update_question(
    State(pool): State<DbPool>,
    updated_question: Json<Question>,
) -> impl IntoResponse {
    let client = pool.get().await.unwrap();
    let rows_affected = client
        .execute(
            "UPDATE questions SET question = $1, answer = $2, source = $3 WHERE id = $4",
            &[
                &updated_question.text,
                &updated_question.answer,
                &updated_question.source,
                &updated_question.id.to_string(),
            ],
        )
        .await
        .unwrap();

    if rows_affected == 1 {
        (StatusCode::OK, Json("Question updated".to_string()))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json("Question not found".to_string()),
        )
    }
}

async fn delete_question(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let client = pool.get().await.unwrap();
    let rows_affected = client
        .execute("DELETE FROM questions WHERE id = $1", &[&id.to_string()])
        .await
        .unwrap();

    if rows_affected == 1 {
        (StatusCode::OK, Json("Question deleted".to_string()))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json("Question not found".to_string()),
        )
    }
}

async fn add_answer(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    answer: Json<String>,
) -> impl IntoResponse {
    let client = pool.get().await.unwrap();
    let rows_affected = client
        .execute(
            "UPDATE questions SET answer = $1 WHERE id = $2",
            &[&answer.to_string(), &id.to_string()],
        )
        .await
        .unwrap();

    if rows_affected == 1 {
        (StatusCode::OK, Json("Answer added".to_string()))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json("Question not found".to_string()),
        )
    }
}

async fn index() -> impl IntoResponse {
    let html = r#"
        <html>
        <body>
            <h1>Questions and Answers Web Server!</h1>
            <button onclick="window.location.href='/question'">Add Question</button>
            <button onclick="window.location.href='/question/123'">Get Question</button>
            <button onclick="window.location.href='/question/123/answer'">Add Answer</button>
            <button onclick="window.location.href='/question/123'">Update Question</button>
            <button onclick="window.location.href='/question/123'">Delete Question</button>
        </body>
        </html>
    "#;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(html))
        .unwrap()
}

fn row_to_question(row: Row) -> Question {
    Question {
        id: Uuid::parse_str(row.get("id")).unwrap(),
        text: row.get("question"),
        answer: row.get("answer"),
        source: row.get("source"),
    }
}

async fn init_db(pool: &DbPool) {
    let client = pool.get().await.unwrap();
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS questions (
            id TEXT PRIMARY KEY,
            question TEXT NOT NULL,
            answer TEXT,
            source TEXT
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT REFERENCES questions(id),
            tag TEXT NOT NULL
        );",
        )
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let password = std::fs::read_to_string(&env::var("PG_PASSWORDFILE").expect("PG_PASSWORDFILE must be set"))
        .expect("Failed to read password file");

    let mut config = tokio_postgres::Config::new();
    config.user(&env::var("PG_USER").expect("PG_USER must be set"));
    config.password(password.trim());
    config.dbname(&env::var("PG_DBNAME").expect("PG_DBNAME must be set"));
    config.host(&env::var("PG_HOST").expect("PG_HOST must be set"));
    config.port(5432);

    let manager = deadpool_postgres::Manager::new(config, NoTls);
    let pool = Pool::builder(manager).max_size(16).build().unwrap();

    init_db(&pool).await;

    let app = Router::new()
        .route("/question/:id", get(get_question))
        .route("/question/:id", put(update_question))
        .route("/question/:id", delete(delete_question))
        .route("/question/:id/answer", post(add_answer))
        .route("/question", post(add_question))
        .route("/", get(index)) // Route for serving HTML content
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
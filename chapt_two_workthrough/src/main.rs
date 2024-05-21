use axum::{
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Question {
    id: Uuid,
    text: String,
    answer: Option<String>,
}

type Questions = Arc<Mutex<HashMap<Uuid, Question>>>;

async fn get_question(
    State(questions): State<Questions>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let questions = questions.lock().await;
    match questions.get(&id) {
        Some(question) => {
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
    State(questions): State<Questions>,
    new_question: Json<Question>,
) -> impl IntoResponse {
    let mut questions = questions.lock().await;
    let question = Question {
        id: new_question.id,
        text: new_question.text.clone(),
        answer: None,
    };
    questions.insert(question.id, question);
    (StatusCode::OK, Json("Inserted successfully".to_string()))
}

async fn update_question(
    State(questions): State<Questions>,
    updated_question: Json<Question>,
) -> impl IntoResponse {
    let mut questions = questions.lock().await;
    if let Some(question) = questions.get_mut(&updated_question.id) {
        *question = Question {
            id: updated_question.id,
            text: updated_question.text.clone(),
            answer: updated_question.answer.clone(),
        };
        (StatusCode::OK, Json("Question updated".to_string()))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json("Question not found".to_string()),
        )
    }
}

async fn delete_question(
    State(questions): State<Questions>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut questions = questions.lock().await;
    if questions.remove(&id).is_some() {
        (StatusCode::OK, Json("Question deleted".to_string()))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json("Question not found".to_string()),
        )
    }
}

async fn add_answer(
    State(questions): State<Questions>,
    Path(id): Path<Uuid>,
    answer: Json<String>,
) -> impl IntoResponse {
    let mut questions = questions.lock().await;
    if let Some(question) = questions.get_mut(&id) {
        question.answer = Some(answer.to_string());
        (StatusCode::OK, Json("Answer added".to_string()))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json("Question not found".to_string()),
        )
    }
}

// Define a function to serve HTML content
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

#[tokio::main]
async fn main() {
    let questions = Questions::default();

    let app = Router::new()
        .route("/question/:id", get(get_question))
        .route("/question/:id", put(update_question))
        .route("/question/:id", delete(delete_question))
        .route("/question/:id/answer", post(add_answer))
        .route("/question", post(add_question))
       // .route("/", get(index)) // Route for serving HTML content
        .with_state(questions);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5433")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

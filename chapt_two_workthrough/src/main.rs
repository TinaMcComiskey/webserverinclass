use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use hyper::server;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use uuid::Uuid;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
};

#[derive(Serialize, Deserialize)]
struct Question {
    id: Uuid,
    text: String,
    answer: Option<String>,
}

type Questions = Arc<Mutex<HashMap<Uuid, Question>>>;

mod handlers {
    use super::*;
    use axum::response::Json;

    pub async fn get_question(
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

    pub async fn add_question(
        State(questions): State<Questions>,
        new_question: Json<Question>,
    ) -> impl IntoResponse {
        let mut questions = questions.lock().await;
        let question = Question {
            id: new_question.id.clone(),
            text: new_question.text.clone(),
            answer: None,
        };
        questions.insert(question.id, question);
        (StatusCode::OK, Json("Inserted successfully".to_string()))
    }

    pub async fn update_question(
        State(questions): State<Questions>,
        updated_question: Json<Question>,
    ) -> impl IntoResponse {
        let mut questions = questions.lock().await;
        if let Some(question) = questions.get_mut(&updated_question.id) {
            *question = Question {
                id: updated_question.id.clone(),
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

    pub async fn delete_question(
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

    pub async fn add_answer(
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
}

mod api {
    use super::*;
    use axum::{Router, Extension};

    pub fn routes(questions: Questions) -> Router<Body> {
        Router::new()
            .route("/question/:id", get(handlers::get_question))
            .route("/question/:id", put(handlers::update_question))
            .route("/question/:id", delete(handlers::delete_question))
            .route("/question/:id/answer", post(handlers::add_answer))
            .route("/question", post(handlers::add_question))
            .layer(Extension::new(questions))
    }
}

#[tokio::main]
async fn main() {
    let questions = Questions::default();

    let app = api::routes(questions);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
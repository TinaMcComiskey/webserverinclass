use axum::{
    routing::{get, post, delete, put}, 
    Router, Server, Json, 
    http::{StatusCode, Response},
    body::Body
};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Question {
    id: Uuid,
    text: String,
    answer: Option<String>,
}

type Questions = Arc<Mutex<HashMap<Uuid, Question>>>;

async fn get_question(questions: Questions, id: String) -> impl IntoResponse {
    let id = Uuid::parse_str(&id).unwrap();
    let questions = questions.lock().await;
    match questions.get(&id) {
         Some(question) => {
            let json = serde_json::to_string(&question).unwrap();
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(json))
                .unwrap()
        },
        None => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Question not found"))
                .unwrap()
        },
    }
}

async fn add_question(questions: Questions, new_question: Json<Question>) -> impl IntoResponse {
    let mut questions = questions.lock().await;
    let question = new_question.0;
     let id = question.id;
    questions.insert(id, question);
    Json(id)
}

async fn update_question(
    questions: Questions, 
    id: String, 
    updated_question: Json<Question>
) -> impl IntoResponse {
    let id = Uuid::parse_str(&id).unwrap();
    let mut questions = questions.lock().await;
    if let Some(question) = questions.get_mut(&id) {
        *question = updated_question.0;
        (StatusCode::OK, "Question updated")
    } else {
        (StatusCode::NOT_FOUND, "Question not found")
    }
}

async fn delete_question(questions: Questions, id: String) -> impl IntoResponse {
    let id = Uuid::parse_str(&id).unwrap();
    let mut questions = questions.lock().await;
    if questions.remove(&id).is_some() {
        (StatusCode::OK, "Question deleted")
    } else {
        (StatusCode::NOT_FOUND, "Question not found")
    }
}

async fn add_answer(questions: Questions, id: String, answer: Json<String>) -> impl IntoResponse {
    let id = Uuid::parse_str(&id).unwrap();
    let mut questions = questions.lock().await;
    if let Some(question) = questions.get_mut(&id) {
        question.answer = Some(answer.0);
        (StatusCode::OK, "Answer added")
    } else {
        (StatusCode::NOT_FOUND, "Question not found")
    }
}

#[tokio::main]
async fn main() {
    let questions = Questions::default();

     let app = Router::new()
        .route("/question/:id", get(move |id: String| get_question(questions.clone(), id)))
        .route("/question", post(move |new_question: axum::Json<Question>| add_question(questions.clone(), new_question)))
        .route("/question/:id", put(move |id: String, updated_question: Json<Question>| update_question(questions.clone(), id, updated_question)))
        .route("/question/:id", delete(move |id: String| delete_question(questions.clone(), id)))
        .route("/question/:id/answer", post(move |id: String, new_answer: axum::Json<Answer>| add_answer(questions.clone(), id, new_answer)));

    axum::Server::bind(&"0.0.0.0:3030".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();  
}
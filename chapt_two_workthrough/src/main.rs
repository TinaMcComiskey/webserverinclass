use axum::{
    routing::{get, post, delete, put}, 
    Router, serve, Json, 
    http::{StatusCode, Response},
    extract::{State, Path},
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

async fn get_question(State(questions): State<Questions>) -> Response<Body> {
    let id = Uuid::new_v4();
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

async fn add_question(State(questions): State<Questions>, Path(id): Path<Uuid>, new_question: Json<Question>) -> (StatusCode, Json<String>) {
    let mut questions = questions.lock().await;
    let id = Path(id);
    let question = Question {
        id: id.clone(),
        text: new_question.text.clone(),
        answer: None,
    };
    questions.insert(*id, question);
    (StatusCode::OK, Json("Inserted successfully".to_string()))
}

async fn update_question(State(questions): State<Questions>, updated_question: Json<Question>) -> (StatusCode, Json<String>) {
    let mut questions = questions.lock().await;
    let id = updated_question.id.clone();
    if let Some(question) = questions.get_mut(&id) {
        *question = Question {
            id: id.clone(),
            text: updated_question.text.clone(),
            answer: updated_question.answer.clone(),
        };
        (StatusCode::OK, Json("Question updated".to_string()))
    } else {
        (StatusCode::NOT_FOUND, Json("Question not found".to_string()))
    }
}

async fn delete_question(State(questions): State<Questions>, Path(id): Path<Uuid>) -> (StatusCode, Json<String>, ) {
    let id = Path(id);
    let mut questions = questions.lock().await;
    if questions.remove(&id).is_some() {
        (StatusCode::OK, Json("Question deleted".to_string()))
    } else {
        (StatusCode::NOT_FOUND, Json("Question not found".to_string()))
    }
}

async fn add_answer(State(questions): State<Questions>, Path(id): Path<Uuid>, answer: Json<String>) -> (StatusCode, Json<String>) {
    let id = Path(id);
    let mut questions = questions.lock().await;
    if let Some(question) = questions.get_mut(&id) {
        question.answer = Some(answer.to_string());
        (StatusCode::OK, Json("Answer added".to_string()))
    } else {
        (StatusCode::NOT_FOUND, Json("Question not found".to_string()))
    }
}

#[tokio::main]
async fn main() {
    let questions = Questions::default();

    let app = Router::new()
       .route("/question/:id", get(get_question))
       .route("/question", post(add_question))
       .route("/question/:id", put(update_question))
       .route("/question/:id", delete(delete_question))
       .route("/question/:id/answer", post(add_answer));
    //let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3030));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
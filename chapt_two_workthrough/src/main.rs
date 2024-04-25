use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::str::FromStr;
use warp::{
    filters::cors::CorsForbidden, http::Method, http::StatusCode, reject::Reject, Filter,
    Rejection, Reply,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
struct QuestionId(String);

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    if let Some(n) = params.get("start") {
        println!("{:?}", n.parse::<usize>());
    }
    if let Some(n) = params.get("start") {
        println!("{}", n);
    }
    match params.get("start") {
        Some(start) => println!("{}", start),
        None => println!("No start value"),
    }
    println!("{:?}", params);
    let res: Vec<Question> = store.questions.values().cloned().collect();
    Ok(warp::reply::json(&res))
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    println!("{:?}", r);
    if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

struct Store {
    questions: HashMap<QuestionId, Question>,
}

#[derive(Clone)]
impl Store {
    fn new() -> Self {
        Store {
            questions: Self::init(),
        }
    }
    /*fn init(self) -> Self {
        let question = Question::new(
            QuestionId::from_str("1").expect("Id not set"),
            "How?".to_string(),
            "Please help!".to_string(),
            Some(vec!["general".to_string()]),
        );
        self.add_question(question)
    }*/
    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("not-in-the-request")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);
    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter)
        .and_then(get_questions)
        .recover(return_error);
    let routes = get_items.with(cors);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Error as IOError, ErrorKind};
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
    let mut start = 0;
    if let Some(n) = params.get("start") {
        start = n.parse::<usize>().expect("Could not parse start");
    }
    println!("{}", start);
    match params.get("start") {
        Some(start) => println!("{}", start),
        None => println!("No start value"),
    }
    println!("{:?}", params);
    let res: Vec<Question> = store.questions.values().cloned().collect();
    Ok(warp::reply::json(&res))
}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        return Ok(Pagination {
            start: params
                .get("start")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
        });
    }
    Err(Error::MissingParameters)
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

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
}

#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameters => write!(f, "Missing parameter"),
        }
    }
}

impl Reject for Error {}

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

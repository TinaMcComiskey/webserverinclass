use std::io::{Error, ErrorKind};
use std::str::FromStr; 

use warp::Filter;
/*
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

impl FromStr for QuestionId {
    type Err = std::io::Error;
    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(QuestionId(id.to_string())),
            true => Err(
                Error::new(ErrorKind::InvalidInput, "No id provided")
            ),
        }
    }
}

impl Question {
    fn new(
        id: QuestionId, 
        title: String, 
        content: String, 
        tags: Option<Vec<String>>
      ) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}
*/
 #[tokio::main]
async fn main() {
    let hello = warp::get()
        .map(|| format!("Hello, World!"));
    warp::serve(hello)
    .run(([127, 0, 0, 1], 3030))
    .await;
}

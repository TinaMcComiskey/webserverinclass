use crate::*;

#[derive(Properties, Clone, PartialEq, serde::Deserialize)]
pub struct questionStruct {
    pub id: String,
    pub text: String,
    pub answer: String 
    pub source: Option<String>,
}

impl questionStruct {
    pub async fn get_question(key: Option<String>) -> Msg {
        let request = match &key {
            None => "http://localhost:3000/api/v1/question".to_string(),
            Some(ref key) => format!("http://localhost:3000/api/v1/question/{}", key,),
        };
        let response = http::Request::get(&request).send().await;
        match response {
            Err(e) => Msg::Gotquestion(Err(e)),
            Ok(data) => Msg::Gotquestion(data.json().await),
        }
    }
}

#[derive(Properties, Clone, PartialEq, serde::Deserialize)]
pub struct questionProps {
    pub question: questionStruct,
}

#[function_component(question)]
pub fn question(question: &questionProps) -> Html {
    let question = &question.question;
    html! { <>
        <div class="question">
            {&question.text}<br/>
            {"Answer:"}<br/>
            {&question.answer}<br/>
        </div>
        <span class="annotation">
            {format!("[id: {}]", &question.id)}
            if let Some(ref source) = question.source {
            {format!("; source: {}", source)}
            }
        </span>
    </> }
}
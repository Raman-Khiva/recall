use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[allow(dead_code)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SnippetMeta {
    title: String,
    summary: String,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct SnippetTitle {
    title: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
pub enum ResponseSchema {
    SnippetMeta,
    SnippetTitle,
}

pub async fn agent(
    messages: &Vec<Message>,
    response_schema: ResponseSchema,
) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .bearer_auth("token")
        .json(&serde_json::json!({
            "agent":"openai/gpt-oss-120b",
            "messages" : &messages,
            "response_format" : {
                "type":"json_schema",
                "json_schema": {
                    "name" : "model reponse",
                    "properties" : response_schema,
                    "strict":true
                }
            }
        }))
        .send()
        .await?;

    Ok(())
}

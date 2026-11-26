use anyhow::Result;
use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[allow(dead_code)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SnippetMeta {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct SnippetTitle {
    pub title: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
pub enum ResponseSchema {
    SnippetMeta,
    SnippetTitle,
}

pub async fn agent(
    // agent props
    messages: &[Message],
    response_schema: serde_json::Value,
    required_props: &[String],
) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let api_key = std::env::var("GROQ_API_KEY")?;
    println!("{}", api_key);
    let response: serde_json::Value = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model":"openai/gpt-oss-120b",
            "messages" : &messages,
            "response_format" : {
                "type":"json_schema",
                "json_schema": {
                    "name" : "model_reponse",
                    "strict":true,
                    "schema":{
                        "type":"object",
                        "properties" : response_schema,
                        "required" : required_props,
                        "additionalProperties":false,
                    },
                }
            }
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let content: String = response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Missing or invalid response content")?
        .to_string();

    Ok(content)
}

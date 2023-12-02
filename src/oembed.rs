use std::env;

use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

#[derive(Deserialize)]
pub struct EmbedRequest {
    #[serde(rename = "n")]
    pub author_name: String,
    #[serde(rename = "i")]
    pub author_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmbedResponse {
    version: &'static str,
    #[serde(rename = "type")]
    embed_type: &'static str,
    author_name: String,
    author_url: String,
    provider_name: String,
    provider_url: String,
}

impl EmbedResponse {
    fn new(author_name: String, author_url: String) -> Self {
        Self {
            version: "1.0",
            embed_type: "rich",
            author_name,
            author_url,
            provider_name: env::var("PROVIDER_NAME").unwrap_or_else(|_| String::from("phixiv")),
            provider_url: env::var("PROVIDER_URL").unwrap_or_else(|_| String::from("https://github.com/HazelTheWitch/phixiv")),
        }
    }
}

pub async fn oembed_handler(
    Query(EmbedRequest {
        author_name,
        author_id,
    }): Query<EmbedRequest>,
) -> Json<EmbedResponse> {
    if let Some(author_id) = author_id {
        Json(EmbedResponse::new(
            author_name,
            format!("https://www.pixiv.net/users/{}", encode(&author_id)),
        ))
    } else {
        Json(EmbedResponse::new(
            author_name,
            String::from("https://www.pixiv.net/"),
        ))
    }
}

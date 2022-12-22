use std::collections::HashMap;

use lambda_http::{run, service_fn, Error, Request, RequestExt};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use thiserror::Error;
use urlencoding::encode;

#[derive(Debug, Error)]
enum EmbedError {
    #[error("url was not provided")]
    InvalidAuthorParameters,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(embed_handler)).await
}

#[derive(Debug, Serialize)]
struct EmbedResponse {
    version: String,
    #[serde(rename = "type")]
    embed_type: String,
    author_name: String,
    author_url: String,
    provider_name: String,
    provider_url: String,
}

impl From<PixivEmbedResponse> for EmbedResponse {
    fn from(por: PixivEmbedResponse) -> Self {
        Self {
            version: "1.0".to_string(),
            embed_type: "rich".to_string(),
            author_name: por.author_name,
            author_url: por.author_url,
            provider_name: "phixiv".into(),
            provider_url: "https://github.com/HazelTheWitch/phixiv".into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PixivEmbedResponse {
    author_name: String,
    author_url: String,
}

async fn embed_handler(request: Request) -> Result<(StatusCode, serde_json::Value), Error> {
    // TODO: This could be optimized to not require so many allocations but I don't care right now.
    let query_string: HashMap<String, String> = request
        .query_string_parameters()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let (Some(author_name), Some(author_id)) = (query_string.get("n"), query_string.get("i")) else {
        return Err(EmbedError::InvalidAuthorParameters)?;
    };

    let pixiv_embed_response = PixivEmbedResponse {
        author_name: author_name.to_string(),
        author_url: format!("https://www.pixiv.net/users/{}", encode(&author_id)),
    };

    Ok((
        StatusCode::OK,
        to_value::<EmbedResponse>(pixiv_embed_response.into())?,
    ))
}

use lambda_http::{run, service_fn, Error, Request, RequestExt};
use reqwest::StatusCode;
use serde::Serialize;
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
struct EmbedResponse<'s> {
    version: &'static str,
    #[serde(rename = "type")]
    embed_type: &'static str,
    author_name: &'s str,
    author_url: String,
    provider_name: &'static str,
    provider_url: &'static str,
}

impl<'s> EmbedResponse<'s> {
    fn new(author_name: &'s str, author_url: String) -> Self {
        Self {
            version: "1.0",
            embed_type: "rich",
            author_name: author_name,
            author_url: author_url,
            provider_name: "phixiv",
            provider_url: "https://github.com/HazelTheWitch/phixiv",
        }
    }
}

async fn embed_handler(request: Request) -> Result<(StatusCode, serde_json::Value), Error> {
    let query_string = request.query_string_parameters();

    let (Some(author_name), Some(author_id)) = (query_string.first("n"), query_string.first("i")) else {
        return Err(EmbedError::InvalidAuthorParameters)?;
    };

    let pixiv_embed_response = EmbedResponse::new(
        author_name,
        format!("https://www.pixiv.net/users/{}", encode(author_id)),
    );

    Ok((
        StatusCode::OK,
        to_value::<EmbedResponse>(pixiv_embed_response)?,
    ))
}

use lambda_http::{run, service_fn, Body, Error, Request, Response};
use reqwest::StatusCode;
use serde::Serialize;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(oembed_handler)).await
}

#[derive(Debug, Serialize)]
struct OembedResponse {
    value: String,
}

async fn oembed_handler(request: Request) -> Result<(StatusCode, serde_json::Value), Error> {
    Ok((StatusCode::OK, serde_json::value::to_value(OembedResponse {value: "good".into()})?))
}
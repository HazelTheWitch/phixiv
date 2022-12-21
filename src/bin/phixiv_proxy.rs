use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use thiserror::Error;

#[derive(Debug, Error)]
enum ProxyError {
    #[error("invalid host provided")]
    InvalidHost,
    #[error("invalid query string parameters provided")]
    InvalidParameters,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(proxy_handler)).await
}

async fn pass_response(response: reqwest::Response) -> Result<Response<Body>, Error> {
    let content_type = response.headers().get("Content-Type");

    Ok({
        let mut builder = Response::builder().status(response.status());

        if let Some(content_type) = content_type {
            builder = builder.header("Content-Type", content_type);
        }

        builder
            .body({
                let bytes: Vec<u8> = response.bytes().await?.into_iter().collect();
                bytes.into()
            })
            .map_err(Box::new)?
    })
}

async fn proxy_handler(request: Request) -> Result<Response<Body>, Error> {
    let url = request
        .query_string_parameters()
        .iter()
        .find_map(|(name, value)| {
            if name == "url" {
                Some(value.to_string())
            } else {
                None
            }
        })
        .ok_or(ProxyError::InvalidParameters)?;

    let url_object = url::Url::parse(&url)?;

    if url_object.host_str() != Some("i.pximg.net") {
        return Err(ProxyError::InvalidHost)?;
    }

    let client = reqwest::Client::new();
    let image_response = client
        .get(&url)
        .header("Referer", "https://www.pixiv.net/")
        .send()
        .await?;

    pass_response(image_response).await
}

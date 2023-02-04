use std::env;

use http::{HeaderValue, HeaderMap};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use phixiv::pixiv::auth::PixivAuth;
use thiserror::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(proxy_handler)).await
}

#[derive(Error, Debug)]
enum ProxyError {
    #[error("could not find query string params")]
    QueryError,
}

async fn pass_response(response: reqwest::Response) -> Result<Response<Body>, Error> {
    Ok({
        let mut builder = Response::builder().status(response.status());

        if let Some(content_type) = response.headers().get("Content-Type") {
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

async fn handle_request(path: &str, base: &str) -> Result<Response<Body>, Error> {
    let client = reqwest::Client::new();

    let auth = PixivAuth::login(&client, &env::var("PIXIV_REFRESH_TOKEN").unwrap()).await?;

    let pximg_url = format!("https://{}.pximg.net{}", &base, &path);

    let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(10);

    headers.append("app-os", "ios".parse().unwrap());
    headers.append("app-os-version", "14.6".parse().unwrap());
    headers.append("user-agent", "PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)".parse().unwrap());
    headers.append("Referer", "https://www.pixiv.net/".parse().unwrap());
    headers.append("Authorization", format!("Bearer {}", auth.access_token).parse().unwrap());

    let image_response = client
        .get(&pximg_url)
        .headers(headers)
        .send()
        .await?;

    println!("{:?}", image_response.status());

    pass_response(image_response).await
}

async fn proxy_handler(request: Request) -> Result<Response<Body>, Error> {
    let query = request.query_string_parameters();

    let path = query.first("p").ok_or(ProxyError::QueryError)?;
    let base = query.first("b").ok_or(ProxyError::QueryError)?;

    if base.len() != 1 {
        return Err(Box::new(ProxyError::QueryError));
    }

    handle_request(path, base).await
}

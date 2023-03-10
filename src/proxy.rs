use std::sync::Arc;

use axum::{
    extract::{Path, State},
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use http::{HeaderMap, HeaderValue, StatusCode};
use moka::future::Cache;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::{auth_middleware, handle_error, ImageBody, PhixivState};

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("could not fetch image")]
    Fetch(#[from] reqwest::Error),
    #[error("could not get the content type from the response")]
    NoContentType,
}

#[instrument(skip(cache, access_token))]
pub async fn fetch_image(
    path: String,
    access_token: String,
    cache: Cache<String, ImageBody>,
) -> Result<ImageBody, ProxyError> {
    let pximg_url = format!("https://i.pximg.net/{path}");

    if let Some(image_body) = cache.get(&path) {
        tracing::info!("Retrieving cached image");

        return Ok(image_body);
    }

    let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(5);

    headers.append("app-os", "ios".parse().unwrap());
    headers.append("app-os-version", "14.6".parse().unwrap());
    headers.append(
        "user-agent",
        "PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)".parse().unwrap(),
    );
    headers.append("Referer", "https://www.pixiv.net/".parse().unwrap());
    headers.append(
        "Authorization",
        format!("Bearer {}", access_token)
            .parse::<HeaderValue>()
            .unwrap(),
    );

    let client = reqwest::Client::new();

    let image_response = client.get(&pximg_url).headers(headers).send().await?;

    match image_response.headers().get("Content-Type") {
        Some(content_type) => {
            let content_type = content_type.to_str().unwrap().to_string();
            let bytes = image_response.bytes().await?;

            let image_body = ImageBody {
                content_type,
                data: bytes,
            };

            tracing::info!("Caching image");

            cache.insert(path, image_body.clone()).await;

            Ok(image_body)
        }
        None => Err(ProxyError::NoContentType),
    }
}

#[instrument(skip(state))]
pub async fn proxy_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let state = state.read().await;

    let cache = state.image_cache.clone();

    Ok(fetch_image(path, state.auth.access_token.clone(), cache)
        .await
        .map_err(|e| handle_error(e.into()))?
        .into_response())
}

pub fn proxy_router(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .route("/*path", get(proxy_handler))
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

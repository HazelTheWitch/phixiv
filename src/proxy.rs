use std::sync::Arc;

use axum::{
    extract::{OriginalUri, State},
    middleware,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use http::{HeaderMap, HeaderValue, StatusCode};
use tokio::sync::RwLock;

use crate::{auth_middleware, handle_error, PhixivState};

pub async fn proxy_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    OriginalUri(uri): OriginalUri,
) -> Result<Response, (StatusCode, String)> {
    let suffix = uri
        .path_and_query()
        .map(|path_and_query| path_and_query.as_str())
        .unwrap_or_default();

    let pximg_url = format!("https://i.pximg.net{suffix}");

    let state = state.read().await;

    let cache = state.image_cache.clone();

    if let Some(image) = cache.get(suffix) {
        tracing::info!("Using cached image for : {suffix}");

        let response = image.into_response();

        let content_type = response.headers().get("Content-Type");

        tracing::info!("{:?}", content_type);
        return Ok(response);
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
        format!("Bearer {}", state.auth.access_token)
            .parse::<HeaderValue>()
            .map_err(|e| handle_error(e.into()))?,
    );

    let client = reqwest::Client::new();

    let image_response = client
        .get(&pximg_url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| handle_error(e.into()))?;

    let image = image_response.bytes().await.map_err(|e| handle_error(e.into()))?;

    cache.insert(suffix.to_owned(), image.clone());

    let response = image.into_response();

    let content_type = response.headers().get("Content-Type");

    tracing::info!("{:?}", content_type);

    Ok(response)
}

pub fn proxy_router(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .route("/*path", get(proxy_handler))
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

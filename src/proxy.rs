use std::sync::Arc;

use axum::{
    body::StreamBody,
    extract::{OriginalUri, State},
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use http::{HeaderMap, HeaderValue, StatusCode};
use tokio::sync::RwLock;

use crate::{auth_middleware, PhixivState, handle_error};

pub async fn proxy_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let suffix = uri
        .path_and_query()
        .map(|path_and_query| path_and_query.as_str())
        .unwrap_or_default();

    let pximg_url = format!("https://i.pximg.net{}", suffix);

    let state = state.read().await;

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

    let stream = image_response.bytes_stream();

    let body = StreamBody::new(stream);

    Ok(body)
}

pub fn proxy_router(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .route("/*path", get(proxy_handler))
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

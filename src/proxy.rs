use std::sync::Arc;

use axum::{
    body::StreamBody,
    extract::{State, Path},
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use http::{HeaderMap, HeaderValue, StatusCode};
use tokio::sync::RwLock;

use crate::{auth_middleware, handle_error, PhixivState, ImageBody, CACHE_SIZE};

pub async fn proxy_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pximg_url = format!("https://i.pximg.net/{path}");

    let state = state.read().await;

    let cache = state.image_cache.clone();

    tracing::info!("Cache Size: {} / {}", cache.weighted_size(), CACHE_SIZE);

    if let Some(image_body) = cache.get(&path) {
        tracing::info!("Using cached image for: {path}");

        return Ok(([("Content-Type", image_body.content_type)], image_body.data).into_response())
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

    match image_response.headers().get("Content-Type") {
        Some(content_type) => {
            let content_type = content_type.to_str().map_err(|e| handle_error(e.into()))?.to_string();
            let bytes = image_response.bytes().await.map_err(|e| handle_error(e.into()))?;

            let image_body = ImageBody {
                content_type,
                data: bytes,
            };

            cache.insert(path, image_body.clone()).await;

            Ok(([("Content-Type", image_body.content_type)], image_body.data).into_response())
        },
        None => {
            Ok(StreamBody::new(image_response.bytes_stream()).into_response())
        },
    }
}

pub fn proxy_router(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .route("/*path", get(proxy_handler))
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

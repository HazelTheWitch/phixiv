use std::{sync::Arc, time::Duration};

use axum::{
    extract::{Path, State},
    headers::CacheControl,
    middleware,
    response::{IntoResponse, Redirect},
    routing::get,
    Router, TypedHeader,
};
use http::{HeaderMap, HeaderValue, StatusCode};
use reqwest::Client;
use thiserror::Error;
use tokio::sync::{RwLock};
use tracing::instrument;

use crate::{
    auth_middleware, handle_error,
    pixiv::artwork::{Artwork, ImageUrl},
    ImageBody, ImageKey, PhixivState,
};

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("could not fetch image")]
    Fetch(#[from] reqwest::Error),
    #[error("could not get the content type from the response")]
    NoContentType,
}

#[instrument(skip_all)]
pub async fn fetch_image(path: &String, access_token: &String) -> Result<ImageBody, ProxyError> {
    let pximg_url = format!("https://i.pximg.net/{path}");

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

    tracing::info!("Fetching {path}");

    Ok((
        TypedHeader(CacheControl::new().with_max_age(Duration::from_secs(60 * 60 * 24))),
        fetch_image(&path, &state.auth.access_token)
            .await
            .map_err(|e| handle_error(e.into()))?
            .into_response(),
    ))
}

pub async fn direct_image_handler(
    Path(image_key): Path<ImageKey>,
    State(state): State<Arc<RwLock<PhixivState>>>,
) -> Result<Redirect, (StatusCode, String)> {
    let state = state.read().await;

    let ImageUrl {
        image_proxy_path: _,
        image_proxy_url,
    } = Artwork::get_image_url(&Client::new(), &image_key.into(), &state.auth.access_token)
        .await
        .map_err(|e| handle_error(e.into()))?;

    tracing::info!("Redirecting to {image_proxy_url}");

    Ok(Redirect::permanent(&image_proxy_url))
}

pub fn direct_router(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .route("/:id", get(direct_image_handler))
        .route("/:id/:image_index", get(direct_image_handler))
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn proxy_router(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .route("/*path", get(proxy_handler))
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

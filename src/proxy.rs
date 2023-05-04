use std::{sync::Arc, time::Duration};

use axum::{
    body::StreamBody,
    extract::{Path, State},
    headers::CacheControl,
    middleware,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router, TypedHeader,
};
use http::{HeaderMap, HeaderValue, StatusCode};
use reqwest::Client;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::{
    auth_middleware,
    pixiv::{
        artwork::{Artwork, ImageUrl},
        PixivError,
    },
    ImageKey, PhixivState,
};

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("could not fetch image")]
    Fetch(#[from] reqwest::Error),
    #[error("underlying pixiv error")]
    Pixiv(#[from] PixivError),
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")).into_response()
    }
}

#[instrument(skip_all)]
pub async fn fetch_image(
    path: &String,
    access_token: &String,
) -> Result<impl IntoResponse, ProxyError> {
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

    Ok(StreamBody::new(image_response.bytes_stream()))
}

#[instrument(skip(state))]
pub async fn proxy_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, ProxyError> {
    let state = state.read().await;

    tracing::info!("Fetching {path}");

    Ok((
        TypedHeader(
            CacheControl::new()
                .with_max_age(Duration::from_secs(60 * 60 * 24))
                .with_public(),
        ),
        fetch_image(&path, &state.auth.access_token)
            .await?
            .into_response(),
    ))
}

pub async fn direct_image_handler(
    Path(image_key): Path<ImageKey>,
    State(state): State<Arc<RwLock<PhixivState>>>,
) -> Result<Response, ProxyError> {
    let state = state.read().await;

    let ImageUrl {
        image_proxy_path: _,
        image_proxy_url,
    } = Artwork::get_image_url(&Client::new(), &image_key.into(), &state.auth.access_token).await?;

    tracing::info!("Redirecting to {image_proxy_url}");

    Ok((
        TypedHeader(
            CacheControl::new()
                .with_max_age(Duration::from_secs(60 * 60 * 24))
                .with_public(),
        ),
        Redirect::permanent(&image_proxy_url),
    )
        .into_response())
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

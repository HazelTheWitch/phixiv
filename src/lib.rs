#![feature(iter_intersperse)]

pub mod embed;
pub mod proxy;
use std::{
    env,
    error::Error,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::{OriginalUri, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use bytes::Bytes;
use http::{Request, StatusCode, Uri};
use moka::future::Cache;
use pixiv::{
    artwork::ArtworkPath,
    auth::{AuthError, PixivAuth},
};
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};
use tracing::instrument;

pub mod phixiv;
pub mod pixiv;

const TOKEN_DURATION: u64 = 3500;
pub const CACHE_SIZE: u64 = 256 * 1024 * 1024;

#[instrument]
pub async fn pixiv_redirect(OriginalUri(uri): OriginalUri) -> impl IntoResponse {
    tracing::info!("Unknown uri: {} redirecting to pixiv.", uri);

    let Some(path_and_query) = uri.path_and_query() else {
        tracing::warn!("Could not find path and query, redirecting to the homepage.");
        return Redirect::temporary("https://www.pixiv.net/");
    };

    let redirect_uri = Uri::builder()
        .scheme("https")
        .authority("www.pixiv.net")
        .path_and_query(path_and_query.as_str())
        .build()
        .unwrap();

    Redirect::temporary(&redirect_uri.to_string())
}

#[derive(Clone)]
pub struct ImageBody {
    pub content_type: String,
    pub data: Bytes,
}

impl ImageBody {
    pub fn into_response(&self) -> Response {
        (
            [("Content-Type", self.content_type.clone())],
            self.data.clone(),
        )
            .into_response()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct ImageKey {
    pub id: String,
    pub image_index: Option<usize>,
}

impl From<ArtworkPath> for ImageKey {
    fn from(path: ArtworkPath) -> Self {
        Self {
            id: path.id,
            image_index: path.image_index,
        }
    }
}

impl From<ImageKey> for ArtworkPath {
    fn from(key: ImageKey) -> Self {
        Self {
            language: None,
            id: key.id,
            image_index: key.image_index,
        }
    }
}

#[derive(Clone)]
pub struct PhixivState {
    pub auth: PixivAuth,
    pub expires_after: Instant,
    pub image_cache: Cache<String, Arc<ImageBody>>,
    pub immediate_cache: Cache<String, Arc<Mutex<Option<ImageBody>>>>,
    pub proxy_url_cache: Cache<ImageKey, String>,
    client: Client,
}

impl PhixivState {
    pub async fn new(max_bytes: u64) -> Result<Self, AuthError> {
        let client = Client::new();
        Ok(Self {
            auth: PixivAuth::login(&client, &env::var("PIXIV_REFRESH_TOKEN").unwrap()).await?,
            expires_after: Instant::now() + Duration::from_secs(TOKEN_DURATION),
            image_cache: Cache::builder()
                .max_capacity(max_bytes)
                .weigher(|_: &String, image: &Arc<ImageBody>| image.data.len() as u32)
                .time_to_live(Duration::from_secs(30 * 60))
                .build(),
            immediate_cache: Cache::builder()
                .max_capacity(256)
                .time_to_live(Duration::from_secs(10))
                .build(),
            proxy_url_cache: Cache::new(4096),
            client,
        })
    }

    #[instrument(skip(self))]
    pub async fn refresh(&mut self) -> Result<(), AuthError> {
        self.auth =
            PixivAuth::login(&self.client, &env::var("PIXIV_REFRESH_TOKEN").unwrap()).await?;
        self.expires_after = Instant::now() + Duration::from_secs(TOKEN_DURATION);

        Ok(())
    }
}

#[instrument(skip(state, request, next))]
pub async fn auth_middleware<B>(
    State(state): State<Arc<RwLock<PhixivState>>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let requires_refresh = {
        let state = state.read().await;
        Instant::now() > state.expires_after
    };

    if requires_refresh {
        tracing::info!("Re-authorizing pixiv token.");
        let mut state = state.write().await;

        state
            .refresh()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    }

    Ok(next.run(request).await)
}

pub fn handle_error(err: Box<dyn Error>) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}"))
}

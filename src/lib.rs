#![feature(iter_intersperse)]

pub mod embed;
pub mod proxy;
use std::{
    env,
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
use moka::sync::Cache;
use pixiv::auth::{AuthError, PixivAuth};
use reqwest::Client;
use tokio::sync::RwLock;

pub mod phixiv;
pub mod pixiv;

const TOKEN_DURATION: u64 = 3500;

pub async fn pixiv_redirect(OriginalUri(uri): OriginalUri) -> impl IntoResponse {
    tracing::info!("Unknown uri: {} redirecting to pixiv.", uri);

    let Some(path_and_query) = uri.path_and_query() else {
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
pub struct PhixivState {
    pub auth: PixivAuth,
    pub expires_after: Instant,
    pub image_cache: Cache<String, Bytes>,
    client: Client,
}

impl PhixivState {
    pub async fn new(max_capacity: u64) -> Result<Self, AuthError> {
        let client = Client::new();
        Ok(Self {
            auth: PixivAuth::login(&client, &env::var("PIXIV_REFRESH_TOKEN").unwrap()).await?,
            expires_after: Instant::now() + Duration::from_secs(TOKEN_DURATION),
            image_cache: Cache::builder().max_capacity(max_capacity).weigher(|_, bytes: &Bytes| bytes.len() as u32).build(),
            client,
        })
    }

    pub async fn refresh(&mut self) -> Result<(), AuthError> {
        self.auth =
            PixivAuth::login(&self.client, &env::var("PIXIV_REFRESH_TOKEN").unwrap()).await?;
        self.expires_after = Instant::now() + Duration::from_secs(TOKEN_DURATION);

        Ok(())
    }
}

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
        let mut state = state.write().await;

        state
            .refresh()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    }

    Ok(next.run(request).await)
}

pub fn handle_error(err: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}"))
}

pub mod embed;
pub mod proxy;
use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{extract::State, middleware::Next, response::Response};
use http::{Request, StatusCode};
use pixiv::auth::{AuthError, PixivAuth};
use reqwest::Client;
use tokio::sync::RwLock;

pub mod phixiv;
pub mod pixiv;

const TOKEN_DURATION: u64 = 3500;

#[derive(Clone)]
pub struct PhixivState {
    pub auth: PixivAuth,
    pub expires_after: Instant,
    client: Client,
}

impl PhixivState {
    pub async fn new() -> Result<Self, AuthError> {
        let client = Client::new();
        Ok(Self {
            auth: PixivAuth::login(&client, &env::var("PIXIV_REFRESH_TOKEN").unwrap()).await?,
            expires_after: Instant::now() + Duration::from_secs(TOKEN_DURATION),
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
    {
        let mut state = state.write().await;

        tracing::info!("Obtained State Lock");

        if Instant::now() > state.expires_after {
            state
                .refresh()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    }

    Ok(next.run(request).await)
}

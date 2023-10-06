use std::sync::Arc;

use axum::{extract::State, middleware::Next, response::Response};
use http::Request;
use reqwest::Client;
use tokio::sync::RwLock;

use crate::{auth::PixivAuth, helper::PhixivError};

#[derive(Clone)]
pub struct PhixivState {
    pub auth: PixivAuth,
    pub client: Client,
}

impl PhixivState {
    pub async fn login(refresh_token: String) -> anyhow::Result<Self> {
        let client = Client::new();

        let auth = PixivAuth::login(&client, refresh_token).await?;

        Ok(Self { auth, client })
    }

    async fn refresh(&mut self) -> anyhow::Result<()> {
        self.auth.refresh(&self.client).await
    }
}

pub async fn authorized_middleware<B>(
    State(state): State<Arc<RwLock<PhixivState>>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, PhixivError> {
    if state.read().await.auth.expired() {
        let mut state = state.write().await;
        state.refresh().await?;
    }

    Ok(next.run(request).await)
}

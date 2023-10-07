use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use http::StatusCode;
use reqwest::Client;
use serde::Deserialize;

use crate::helper;

/// Token lifetime, actually 3600 seconds, but using 3500 to be safe
const TOKEN_DURATION: u64 = 3500;

const CLIENT_ID: &str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
const CLIENT_SECRET: &str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";

#[derive(Debug, Deserialize)]
struct AuthPayload {
    pub response: AuthResponse,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

/// Pixiv authorization state manager, holds and manages refreshing access tokens for authorization.
#[derive(Clone)]
pub struct PixivAuth {
    pub access_token: String,
    refresh_token: String,
    expires_after: Instant,
}

impl PixivAuth {
    async fn authorize(client: &Client, refresh_token: &String) -> anyhow::Result<AuthResponse> {
        let form_data = HashMap::from([
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("get_secure_url", "1"),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ]);

        let auth_response = client
            .post("https://oauth.secure.pixiv.net/auth/token")
            .headers(helper::headers())
            .form(&form_data)
            .send()
            .await?;

        match auth_response.status() {
            StatusCode::OK | StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND => {}
            s => {
                anyhow::bail!("invalid credentials, status code {s}")
            }
        }

        Ok(auth_response.json::<AuthPayload>().await?.response)
    }

    pub fn expired(&self) -> bool {
        Instant::now() > self.expires_after
    }

    pub async fn login(client: &Client, refresh_token: String) -> anyhow::Result<Self> {
        let response = Self::authorize(client, &refresh_token).await?;

        Ok(Self {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_after: Instant::now() + Duration::from_secs(TOKEN_DURATION),
        })
    }

    pub async fn refresh(&mut self, client: &Client) -> anyhow::Result<()> {
        let response = Self::authorize(client, &self.refresh_token).await?;

        self.access_token = response.access_token;
        self.refresh_token = response.refresh_token;
        self.expires_after = Instant::now() + Duration::from_secs(TOKEN_DURATION);

        Ok(())
    }
}

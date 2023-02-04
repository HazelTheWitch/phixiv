use std::collections::HashMap;

use http::{HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use thiserror::Error;


// Taken from https://github.com/fairingrey/rustpixiv/blob/master/src/client.rs#L11
const CLIENT_ID: &str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
const CLIENT_SECRET: &str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("could not log in")]
    Login(#[from] reqwest::Error),
    #[error("invalid refresh_token")]
    InvalidCredentials(StatusCode),
}

#[derive(Deserialize)]
struct AuthPayload {
    pub response: AuthResponse,
}

#[derive(Deserialize)]
struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug)]
pub struct PixivAuth {
    pub access_token: String,
    pub refresh_token: String,
}

impl PixivAuth {
    pub async fn login(client: &Client, refresh_token: &str) -> Result<Self, AuthError> {
        let data = HashMap::from([
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("get_secure_url", "1"),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ]);

        let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(10);

        headers.append("app-os", "ios".parse().unwrap());
        headers.append("app-os-version", "14.6".parse().unwrap());
        headers.append("user-agent", "PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)".parse().unwrap());
        headers.append("host", "oauth.secure.pixiv.net".parse().unwrap());

        let auth_response = client
            .post("https://oauth.secure.pixiv.net/auth/token")
            .headers(headers)
            .form(&data)
            .send()
            .await?;

        match auth_response.status() {
            StatusCode::OK | StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND => { },
            s => {
                return Err(AuthError::InvalidCredentials(s));
            },
        }

        let response_payload: AuthPayload = auth_response.json().await?;

        Ok(Self { access_token: response_payload.response.access_token, refresh_token: response_payload.response.refresh_token })
    }
}
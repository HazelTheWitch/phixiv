mod app_payload;
pub mod artwork;
pub mod auth;
mod response_payload;

use std::{collections::HashMap, env};

use crate::pixiv::{auth::PixivAuth, response_payload::PixivResponse};
use http::{HeaderMap, HeaderValue};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use thiserror::Error;

use self::{
    app_payload::AppReponse,
    artwork::{Artwork, ArtworkError},
    auth::AuthError,
};

lazy_static! {
    static ref ARTWORK_RE: Regex = Regex::new(r#"^(/.+)?/artworks/(\d+)/?$"#).unwrap();
}

const ILLUST_URL: &str = "https://app-api.pixiv.net/v1/illust/detail";

#[derive(Debug, Error)]
pub enum PixivError {
    #[error("not an artwork path")]
    NotArtworkPath,
    #[error("no artwork id, should never happen")]
    NoArtworkID,
    #[error("failed to resolve PixivPath")]
    Resolution(#[from] reqwest::Error),
    #[error("failed to parse the pixiv data to an artwork")]
    Artwork(#[from] ArtworkError),
    #[error("failed to authenticate with pixiv")]
    Auth(#[from] AuthError),
}

pub struct PixivPath {
    language: Option<String>,
    artwork_id: String,
}

impl PixivPath {
    pub fn parse(path: &str) -> Result<Self, PixivError> {
        let capture = ARTWORK_RE
            .captures(path)
            .ok_or(PixivError::NotArtworkPath)?;

        let language = capture.get(1).map(|m| m.as_str());
        let artwork_id = capture
            .get(2)
            .map(|m| m.as_str())
            .ok_or(PixivError::NoArtworkID)?;

        Ok(Self {
            language: language.map(ToString::to_string),
            artwork_id: artwork_id.to_string(),
        })
    }
}

impl Artwork {
    async fn app_request(
        client: &Client,
        path: &PixivPath,
        refresh_token: &str,
    ) -> Result<AppReponse, PixivError> {
        let params = HashMap::from([("illust_id", &path.artwork_id)]);

        let auth = PixivAuth::login(&client, &refresh_token).await?;

        let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(10);

        headers.append("app-os", "ios".parse().unwrap());
        headers.append("app-os-version", "14.6".parse().unwrap());
        headers.append(
            "user-agent",
            "PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)".parse().unwrap(),
        );
        headers.append("host", "app-api.pixiv.net".parse().unwrap());
        headers.append(
            "Authorization",
            format!("Bearer {}", auth.access_token).parse().unwrap(),
        );

        Ok(client
            .get(ILLUST_URL)
            .headers(headers)
            .query(&params)
            .send()
            .await?
            .json()
            .await?)
    }

    async fn ajax_request(client: &Client, path: &PixivPath) -> Result<PixivResponse, PixivError> {
        let ajax_url = format!(
            "https://www.pixiv.net/ajax/illust/{}?lang={}",
            &path.artwork_id,
            &path.language.clone().unwrap_or_else(|| "jp".to_owned())
        );

        Ok(client.get(ajax_url).send().await?.json().await?)
    }

    pub async fn from_path(path: PixivPath, refresh_token: &str) -> Result<Self, PixivError> {
        let client = Client::new();

        let (app, ajax) = tokio::join!(
            Artwork::app_request(&client, &path, &refresh_token),
            Artwork::ajax_request(&client, &path),
        );

        let app_response = app?;
        let ajax_response = ajax?;

        let body = ajax_response.body;

        let description = if body.description.is_empty() {
            body.alt.clone()
        } else {
            body.description
        };

        Ok(Self {
            #[cfg(feature = "small_images")]
            image_proxy_url: Artwork::format_image_proxy_url(&app_response.illust.image_urls.medium)?,
            #[cfg(not(feature = "small_images"))]
            image_proxy_url: Artwork::format_image_proxy_url(&app_response.illust.image_urls.large)?,
            title: body.title,
            description,
            url: body.extra_data.meta.canonical,
            alt_text: body.alt,
            author_name: body.author_name,
            author_id: body.author_id,
            embed_url: env::var("EMBED_URL").unwrap(),
        })
    }
}

use serde::Deserialize;
use thiserror::Error;
use lazy_static::lazy_static;
use regex::Regex;

use crate::artwork::Artwork;

#[derive(Debug, Error)]
pub enum PixivError {
    #[error("not an artwork path")]
    NotArtworkPath,
    #[error("no artwork id, should never happen")]
    NoArtworkID,
    #[error("failed to resolve PixivPath")]
    Resolution(#[from] ResolutionError)
}

#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("failed to retrieve url")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize)]
pub struct PixivResponse {
    pub body: PixivBody,
}

#[derive(Debug, Deserialize)]
pub struct PixivBody {
    pub title: String,
    pub description: String,
    pub alt: String,
    pub urls: PixivUrls,
    #[serde(rename = "userId")]
    pub author_id: String,
    #[serde(rename = "userName")]
    pub author_name: String,
    #[serde(rename = "extraData")]
    pub extra_data: PixivExtraData,
}

#[derive(Debug, Deserialize)]
pub struct PixivUrls {
    pub small: String,
}

#[derive(Debug, Deserialize)]
pub struct PixivExtraData {
    pub meta: PixivMeta,
}

#[derive(Debug, Deserialize)]
pub struct PixivMeta {
    pub canonical: String,
}

#[derive(Debug)]
pub struct PixivPath {
    language: Option<String>,
    artwork_id: String,
}

impl PixivPath {
    /// Parses a `PixivUrl` from a URL path
    pub fn parse(path: &str) -> Result<Self, PixivError> {
        lazy_static! {
            static ref ARTWORK_RE: Regex = Regex::new(r#"^(/.+)?/artworks/(\d+)/?$"#).unwrap();
        }

        let capture = ARTWORK_RE.captures(path).ok_or(PixivError::NotArtworkPath)?;

        let language = capture.get(1).map(|m| m.as_str());
        let artwork_id = capture.get(2).map(|m| m.as_str()).ok_or(PixivError::NoArtworkID)?;

        Ok(Self {
            language: language.map(|s| s.to_string()),
            artwork_id: artwork_id.to_string()
        })
    }

    pub async fn resolve(self) -> Result<Artwork, PixivError> {
        let url = format!("https://www.pixiv.net/ajax/illust/{}?lang={}", self.artwork_id, self.language.unwrap_or("jp".to_string()));

        let pixiv_response = reqwest::get(url)
            .await.map_err(ResolutionError::Reqwest)?
            .json::<PixivResponse>()
            .await.map_err(ResolutionError::Reqwest)?;

        Ok(pixiv_response.into())
    }
}

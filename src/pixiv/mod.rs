pub mod artwork;
mod types;

use std::env;

use crate::{
    pixiv::types::PixivResponse,
};
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

use self::artwork::{ArtworkError, Artwork};

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
}

impl TryFrom<PixivResponse> for Artwork {
    type Error = ArtworkError;

    fn try_from(response: PixivResponse) -> Result<Self, Self::Error> {
        let body = response.body;

        let description = if body.description.is_empty() {
            body.alt.clone()
        } else {
            body.description
        };

        Ok(Self {
            #[cfg(feature = "small_images")]
            image_proxy_url: Artwork::format_image_proxy_url(&body.urls.small)?,
            #[cfg(not(feature = "small_images"))]
            image_proxy_url: Artwork::format_image_proxy_url(&body.urls.regular)?,
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

impl Artwork {
    pub async fn from_path(path: &str) -> Result<Self, PixivError> {
        lazy_static! {
            static ref ARTWORK_RE: Regex = Regex::new(r#"^(/.+)?/artworks/(\d+)/?$"#).unwrap();
        }

        let capture = ARTWORK_RE
            .captures(path)
            .ok_or(PixivError::NotArtworkPath)?;

        let language = capture.get(1).map(|m| m.as_str());
        let artwork_id = capture
            .get(2)
            .map(|m| m.as_str())
            .ok_or(PixivError::NoArtworkID)?;

        let language = language.map(ToString::to_string);
        let artwork_id = artwork_id.to_string();

        let url = format!(
            "https://www.pixiv.net/ajax/illust/{}?lang={}",
            artwork_id,
            language.unwrap_or_else(|| "jp".to_owned())
        );

        let pixiv_response = reqwest::get(url).await?.json::<PixivResponse>().await?;

        Ok(pixiv_response.try_into()?)
    }
}

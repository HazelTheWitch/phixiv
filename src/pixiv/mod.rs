pub mod artwork;
pub mod auth;
mod payloads;

use thiserror::Error;

use self::{artwork::ArtworkError, auth::AuthError};

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

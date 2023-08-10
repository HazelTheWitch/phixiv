use std::sync::Arc;

use axum::{
    extract::{Path, State, Host},
    headers::{CacheControl, UserAgent},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router, TypedHeader,
};

use tokio::sync::RwLock;
use tracing::{info, instrument};

use crate::{
    pixiv::artwork::{Artwork, RawArtworkPath},
    pixiv_redirect, PhixivState,
};

#[instrument(skip(state))]
pub async fn artwork_handler(
    Path(path): Path<RawArtworkPath>,
    State(state): State<Arc<RwLock<PhixivState>>>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Host(host): Host,
) -> Result<Response, Response> {
    let path = path.parse();

    let redirect = (
        TypedHeader(CacheControl::new().with_no_cache()),
        Redirect::temporary(&format!("http://www.pixiv.net{}", path.format_path())),
    );

    #[cfg(feature = "bot_filtering")]
    {
        let bots = isbot::Bots::default();

        if !bots.is_bot(user_agent.as_str()) {
            tracing::info!("Non-bot request, redirecting to pixiv.");

            return Ok(redirect.into_response());
        }
    }

    let state = state.read().await;

    let artwork = match Artwork::from_path(&path, &state.auth.access_token, host).await {
        Ok(artwork) => artwork,
        Err(e) => {
            tracing::error!("{e}");
            return Err(redirect.into_response());
        }
    };

    info!("Parsed artwork: {}", path.id);

    Ok((
        TypedHeader(CacheControl::new().with_no_cache()),
        Html(match artwork.render_minified() {
            Ok(html) => html,
            Err(e) => {
                tracing::error!("{e}");
                return Err(redirect.into_response());
            }
        }),
    )
        .into_response())
}

pub fn phixiv_router(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .route("/:language/artworks/:id", get(artwork_handler))
        .route("/:language/artworks/:id/:image_index", get(artwork_handler))
        .route("/artworks/:id", get(artwork_handler))
        .route("/artworks/:id/:image_index", get(artwork_handler))
        .fallback(pixiv_redirect)
        .with_state(state.clone())
}

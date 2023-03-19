use std::sync::Arc;

use axum::{
    extract::{State, Path},
    response::{Html, Response, IntoResponse},
    routing::get,
    Router, headers::UserAgent, TypedHeader,
};

#[cfg(feature = "bot_filtering")]
use axum::response::Redirect;

use http::StatusCode;
use tokio::sync::RwLock;
use tracing::{info, instrument};

use crate::{
    handle_error,
    pixiv::artwork::{Artwork, RawArtworkPath},
    pixiv_redirect, PhixivState, proxy::fetch_image,
};

#[instrument(skip(state))]
pub async fn artwork_handler(
    Path(path): Path<RawArtworkPath>,
    State(state): State<Arc<RwLock<PhixivState>>>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
) -> Result<Response, (StatusCode, String)> {
    #[cfg(feature = "bot_filtering")]
    {
        let bots = isbot::Bots::default();

        if !bots.is_bot(user_agent.as_str()) {
            tracing::info!("Non-bot request, redirecting to pixiv.");

            return Ok(Redirect::temporary(&format!("http://www.pixiv.net{}", path.format_path())).into_response());
        }
    }

    let path = path.parse();

    let state = state.read().await;

    let artwork = Artwork::from_path(path, &state.auth.access_token)
        .await
        .map_err(|e| handle_error(e.into()))?;

    info!("Parsed artwork");

    let _ = fetch_image(artwork.image_proxy_path.clone(), state.auth.access_token.clone(), state.image_cache.clone());

    Ok(Html(
        artwork
            .render_minified()
            .map_err(|e| handle_error(e.into()))?,
    ).into_response())
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

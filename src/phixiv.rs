use std::sync::Arc;

use axum::{
    extract::{Path, State},
    middleware,
    response::Html,
    routing::get,
    Router,
};
use http::StatusCode;
use tokio::sync::RwLock;

use crate::{
    auth_middleware,
    pixiv::artwork::{Artwork, ArtworkPath},
    PhixivState, handle_error, pixiv_redirect,
};

pub async fn artwork_handler(
    Path(path): Path<ArtworkPath>,
    State(state): State<Arc<RwLock<PhixivState>>>,
) -> Result<Html<String>, (StatusCode, String)> {
    let state = state.read().await;

    tracing::info!("Access Token: {}", &state.auth.access_token);

    let artwork = Artwork::from_path(path, &state.auth.access_token)
        .await
        .map_err(|e| handle_error(e.into()))?;

    Ok(Html(
        artwork
            .render_minified()
            .map_err(|e| handle_error(e.into()))?,
    ))
}

pub fn phixiv_router(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .route("/:language/artworks/:id", get(artwork_handler))
        .route("/artworks/:id", get(artwork_handler))
        .fallback(pixiv_redirect)
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

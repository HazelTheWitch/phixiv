use std::sync::Arc;

use axum::{
    extract::{Path, State},
    middleware::{self, Next},
    response::{Html, Response},
    routing::get,
    Router,
};

#[cfg(feature = "bot_filtering")]
use axum::response::{IntoResponse, Redirect};

use http::{Request, StatusCode};
use tokio::sync::RwLock;
use tracing::{info, instrument};

use crate::{
    auth_middleware, handle_error,
    pixiv::artwork::{Artwork, ArtworkPath},
    pixiv_redirect, PhixivState, proxy::fetch_image,
};

#[instrument(skip(state))]
pub async fn artwork_handler(
    Path(path): Path<ArtworkPath>,
    State(state): State<Arc<RwLock<PhixivState>>>,
) -> Result<Html<String>, (StatusCode, String)> {
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
    ))
}

#[instrument(skip(request, next))]
pub async fn redirect_middleware<B>(request: Request<B>, next: Next<B>) -> Response {
    #[cfg(feature = "bot_filtering")]
    {
        let bots = isbot::Bots::default();

        if let Some(Ok(user_agent)) = request.headers().get("User-Agent").map(|h| h.to_str()) {
            if !bots.is_bot(user_agent) {
                tracing::info!("Non-bot request, redirecting to pixiv.");

                let uri = request.uri();

                let path_and_query = match uri.path_and_query() {
                    Some(path_and_query) => path_and_query.as_str(),
                    None => "",
                };

                return Redirect::temporary(&format!("http://www.pixiv.net{path_and_query}"))
                    .into_response();
            }
        }
    }

    next.run(request).await
}

pub fn phixiv_router(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .route("/:language/artworks/:id", get(artwork_handler))
        .route("/artworks/:id", get(artwork_handler))
        .fallback(pixiv_redirect)
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state, auth_middleware))
        .layer(middleware::from_fn(redirect_middleware))
}

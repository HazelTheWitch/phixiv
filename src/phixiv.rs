use std::sync::Arc;

use axum::{
    extract::{Path, State},
    headers::UserAgent,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router, TypedHeader,
};

use tokio::sync::{Mutex, RwLock};
use tracing::{info, instrument};

use crate::{
    pixiv::artwork::{Artwork, RawArtworkPath},
    pixiv_redirect,
    proxy::fetch_image,
    PhixivState,
};

#[instrument(skip(state))]
pub async fn artwork_handler(
    Path(path): Path<RawArtworkPath>,
    State(state): State<Arc<RwLock<PhixivState>>>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
) -> Result<Response, Response> {
    let path = path.parse();

    let redirect = Redirect::temporary(&format!("http://www.pixiv.net{}", path.format_path()));

    #[cfg(feature = "bot_filtering")]
    {
        let bots = isbot::Bots::default();

        if !bots.is_bot(user_agent.as_str()) {
            tracing::info!("Non-bot request, redirecting to pixiv.");

            return Ok(redirect.into_response());
        }
    }

    let state = state.read().await;

    let artwork = match Artwork::from_path(&path, &state.auth.access_token).await {
        Ok(artwork) => artwork,
        Err(e) => {
            tracing::error!("{e}");
            return Err(redirect.into_response());
        }
    };

    info!("Parsed artwork: {}", path.id);

    {
        let proxy_path = artwork.image_proxy_path.clone();

        state
            .proxy_url_cache
            .insert(path.into(), artwork.image_proxy_url.clone())
            .await;

        if !state.image_cache.contains_key(&proxy_path) {
            let immediate = state.immediate_cache.clone();
            let access_token = state.auth.access_token.clone();

            let image = Arc::new(Mutex::new(None));
            if !immediate.contains_key(&proxy_path) {
                immediate.insert(proxy_path.clone(), image.clone()).await;
                tracing::info!("Inserted dummy image");

                tokio::spawn(async move {
                    let mut image = image.lock().await;

                    let Ok(image_body) = fetch_image(&proxy_path, &access_token).await else {
                        immediate.invalidate(&proxy_path).await;
                        return;
                    };

                    *image = Some(image_body);

                    tracing::info!("Inserted real image into immediate cache");
                });
            }
        }
    }

    Ok(Html(match artwork.render_minified() {
        Ok(html) => html,
        Err(e) => {
            tracing::error!("{e}");
            return Err(redirect.into_response());
        }
    })
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

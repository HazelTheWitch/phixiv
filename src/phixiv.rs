use std::sync::Arc;

use axum::{
    extract::{OriginalUri, Path, State},
    middleware,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Router,
};
use http::{StatusCode, Uri};
use tokio::sync::RwLock;

use crate::{
    auth_middleware,
    pixiv::artwork::{Artwork, ArtworkPath},
    PhixivState,
};

async fn pixiv_redirect(OriginalUri(uri): OriginalUri) -> impl IntoResponse {
    tracing::info!("Unknown uri: {} redirecting to pixiv.", uri);

    let Some(path_and_query) = uri.path_and_query() else {
        return Redirect::temporary("https://www.pixiv.net/");
    };

    let redirect_uri = Uri::builder()
        .scheme("https")
        .authority("www.pixiv.net")
        .path_and_query(path_and_query.as_str())
        .build()
        .unwrap();

    Redirect::temporary(&redirect_uri.to_string())
}

pub async fn artwork_handler(
    Path(path): Path<ArtworkPath>,
    State(state): State<Arc<RwLock<PhixivState>>>,
) -> Result<Html<String>, StatusCode> {
    let state = state.read().await;

    tracing::info!("Access Token: {}", &state.auth.access_token);

    let artwork = Artwork::from_path(path, &state.auth.access_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(
        artwork
            .render_minified()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
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

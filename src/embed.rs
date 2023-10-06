use std::{env, sync::Arc};

use askama::Template;
use axum::{
    extract::{Host, OriginalUri, Path, Query, State},
    headers::{CacheControl, UserAgent},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router, TypedHeader,
};
use http::{Request, Uri};
use serde::Deserialize;
use tokio::sync::RwLock;
use tower::ServiceBuilder;

use crate::{
    helper::PhixivError,
    pixiv::{ArtworkListing, ArtworkPath, RawArtworkPath},
    state::{authorized_middleware, PhixivState},
};

async fn artwork_response(
    raw_path: RawArtworkPath,
    state: Arc<RwLock<PhixivState>>,
    host: String,
) -> anyhow::Result<Response> {
    let path: ArtworkPath = raw_path.try_into()?;

    let state = state.read().await;

    let listing = ArtworkListing::get_listing(
        path.language,
        path.id,
        &state.auth.access_token,
        &host,
        &state.client,
    )
    .await?;

    let artwork = listing.to_template(path.image_index, host);

    Ok((
        TypedHeader(CacheControl::new().with_no_cache()),
        Html(artwork.render()?),
    )
        .into_response())
}

async fn artwork_handler(
    Path(path): Path<RawArtworkPath>,
    State(state): State<Arc<RwLock<PhixivState>>>,
    Host(host): Host,
) -> Result<Response, PhixivError> {
    Ok(artwork_response(path, state, host).await?)
}

#[derive(Deserialize)]
struct MemberIllustParams {
    pub illust_id: String,
}

impl From<MemberIllustParams> for RawArtworkPath {
    fn from(params: MemberIllustParams) -> Self {
        Self {
            language: None,
            id: params.illust_id,
            image_index: None,
        }
    }
}

async fn member_illust_handler(
    Query(params): Query<MemberIllustParams>,
    State(state): State<Arc<RwLock<PhixivState>>>,
    Host(host): Host,
) -> Result<Response, PhixivError> {
    Ok(artwork_response(params.into(), state, host).await?)
}

fn redirect_uri(uri: Uri) -> String {
    let Some(path_and_query) = uri.path_and_query() else {
        return String::from("https://www.pixiv.net/");
    };

    Uri::builder()
        .scheme("https")
        .authority("www.pixiv.net")
        .path_and_query(path_and_query.as_str())
        .build()
        .unwrap()
        .to_string()
}

async fn redirect_middleware<B>(
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    OriginalUri(uri): OriginalUri,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, PhixivError> {
    if env::var("BOT_FILTERING")
        .unwrap_or_else(|_| String::from("false"))
        .parse::<bool>()?
    {
        let bots = isbot::Bots::default();

        if !bots.is_bot(user_agent.as_str()) {
            return Ok(Redirect::temporary(&redirect_uri(uri)).into_response());
        }
    }

    Ok(next.run(request).await)
}

async fn redirect_fallback(OriginalUri(uri): OriginalUri) -> Redirect {
    Redirect::temporary(&redirect_uri(uri))
}

pub fn router(
    state: Arc<RwLock<PhixivState>>,
) -> Router<Arc<RwLock<PhixivState>>, axum::body::Body> {
    Router::new()
        .route("/:language/artworks/:id", get(artwork_handler))
        .route("/:language/artworks/:id/:image_index", get(artwork_handler))
        .route("/artworks/:id", get(artwork_handler))
        .route("/artworks/:id/:image_index", get(artwork_handler))
        .route("/member_illust.php", get(member_illust_handler))
        .fallback(redirect_fallback)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(redirect_middleware))
                .layer(middleware::from_fn_with_state(state, authorized_middleware)),
        )
}

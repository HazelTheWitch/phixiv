use std::{env, sync::Arc};

use axum::{routing::get, Router};
use phixiv::{
    embed::embed_handler,
    phixiv::phixiv_router,
    pixiv_redirect,
    proxy::{direct_router, proxy_router},
    PhixivState,
};
use tokio::sync::RwLock;

use tower_http::normalize_path::NormalizePathLayer;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    #[cfg(feature = "sentry")]
    let _guard = sentry::init((env::var("SENTRY_URL").unwrap().as_str(), sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    }));

    let registry = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env());

    #[cfg(feature = "sentry")]
    registry.with(sentry_tracing::layer());

    registry.init();

    let state = Arc::new(RwLock::new(PhixivState::new().await.unwrap()));

    let phixiv = phixiv_router(state.clone());
    let proxy = proxy_router(state.clone());
    let direct = direct_router(state.clone());

    let app = Router::new()
        .merge(phixiv)
        .route("/e", get(embed_handler))
        .nest("/i", proxy)
        .nest("/d", direct)
        .fallback(pixiv_redirect)
        .layer(NormalizePathLayer::trim_trailing_slash());

    let addr = format!("[::]:{}", env::var("PORT").unwrap_or("3000".to_owned()))
        .parse()
        .unwrap();

    tracing::info!("Listening on: {}", addr);
    tracing::info!("Hosted to: {}", env::var("RAILWAY_STATIC_URL").unwrap());

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

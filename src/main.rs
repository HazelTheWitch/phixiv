use std::{env, sync::Arc};

use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;
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

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::fmt().with_file(true).init();

    let state = Arc::new(RwLock::new(PhixivState::new().await.unwrap()));

    let phixiv = phixiv_router(state.clone());
    let proxy = proxy_router(state.clone());
    let direct = direct_router(state.clone());

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app = Router::new()
        .merge(phixiv)
        .route("/e", get(embed_handler))
        .nest("/i", proxy)
        .nest("/d", direct)
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .fallback(pixiv_redirect)
        .layer(prometheus_layer)
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

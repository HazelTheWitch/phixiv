use std::{env, sync::Arc};

use axum::Router;
use phixiv::{
    embed::embed_router, phixiv::phixiv_router, pixiv_redirect, proxy::proxy_router, PhixivState,
};
use tokio::sync::RwLock;

use tower_http::{normalize_path::NormalizePathLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::fmt()
        .with_file(true)
        .init();

    let state = Arc::new(RwLock::new(PhixivState::new().await.unwrap()));

    let phixiv = phixiv_router(state.clone());
    let embed = embed_router();
    let proxy = proxy_router(state.clone());

    let app = Router::new()
        .nest("/", phixiv)
        .nest("/e", embed)
        .nest("/i", proxy)
        .fallback(pixiv_redirect)
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(TraceLayer::new_for_http());

    let addr = format!("[::]:{}", env::var("PORT").unwrap_or("3000".to_owned()))
        .parse()
        .unwrap();

    tracing::info!("Listening on: {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

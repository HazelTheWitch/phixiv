use std::sync::Arc;

use axum::{body::Body, extract::Host, routing::get, Router};
use http::Request;
use phixiv::{embed::embed_router, phixiv::phixiv_router, proxy::proxy_router, PhixivState};
use tokio::sync::RwLock;
use tower::ServiceExt;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::fmt().with_file(true).init();

    let state = Arc::new(RwLock::new(PhixivState::new().await.unwrap()));

    let phixiv = phixiv_router(state.clone());

    let embed = embed_router();

    let proxy = proxy_router(state.clone());

    let app = Router::new()
        .route(
            "/*path",
            get(|Host(hostname): Host, request: Request<Body>| async move {
                match hostname.split_once(".") {
                    Some(("i", _)) => proxy.oneshot(request).await,
                    Some(("e", _)) => embed.oneshot(request).await,
                    _ => phixiv.oneshot(request).await,
                }
            }),
        )
        .with_state(state);

    let addr = "[::]:3000".parse().unwrap();

    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

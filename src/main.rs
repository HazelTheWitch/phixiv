pub mod state;
pub mod auth;
pub mod helper;
pub mod embed;
pub mod pixiv;
pub mod oembed;
pub mod proxy;
pub mod api;

use std::{env, net::SocketAddr, sync::Arc};

use api::api_router;
use axum::{response::IntoResponse, Json, Router, routing::get};
use oembed::oembed_handler;
use proxy::proxy_router;
use serde_json::json;
use state::PhixivState;
use tokio::sync::RwLock;
use tower_http::{trace::{TraceLayer, DefaultOnRequest, DefaultMakeSpan}, normalize_path::NormalizePathLayer};
use tracing::Level;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let addr: SocketAddr = format!("[::]:{}", env::var("PORT").unwrap_or_else(|_| String::from("3000"))).parse()?;

    let tracing_registry = tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env());

    if let Ok(loki_url) = env::var("LOKI_URL") {
        let (layer, task) = tracing_loki::builder()
            .label("environment", &env::var("ENVIRONMENT").unwrap_or_else(|_| String::from("development")))?
            .build_url(url::Url::parse(&loki_url).unwrap())?;

        tokio::spawn(task);

        tracing_registry.with(layer).init();
    } else {
        tracing_registry.init();
    }

    tracing::info!("Listening on: {addr}");

    let state = Arc::new(RwLock::new(PhixivState::login(env::var("PIXIV_REFRESH_TOKEN")?).await?));

    axum::Server::bind(&addr)
        .serve(app(state).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn app(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .merge(embed::router(state.clone()))
        .route("/health", get(health))
        .route("/e", get(oembed_handler))
        .nest("/i", proxy_router(state.clone()))
        .nest("/api", api_router(state.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
        )
        .layer(NormalizePathLayer::trim_trailing_slash())
        .with_state(state)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn health() -> impl IntoResponse {
    Json(json!({ "health": "UP" }))
}
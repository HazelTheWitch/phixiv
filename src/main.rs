use std::{net::SocketAddr, env};

use axum::{response::Html, Router, routing::get};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_file(true)
        .init();

    let app = Router::new()
        .route("/", get(handler));

    let addr = SocketAddr::from((
        [127, 0, 0, 1],
        env::var("PORT")
            .unwrap_or_else(|_| "3000".to_owned())
            .parse::<u16>()
            .unwrap()
    ));

    tracing::info!("Listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello World!</h1>")
}
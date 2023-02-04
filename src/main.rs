use axum::{response::Html, Router, routing::get};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_file(true)
        .init();

    let app = Router::new()
        .route("/", get(handler));

    let addr = "[::]:3000".parse().unwrap();

    tracing::info!("Listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello World!</h1>")
}
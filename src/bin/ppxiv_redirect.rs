use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};

static REDIRECT_PATH: &str = "https://www.phixiv.net";

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(redirect_handler)).await
}

async fn redirect_handler(request: Request) -> Result<Response<Body>, Error> {
    let redirect_url = format!("{}{}", REDIRECT_PATH, request.raw_http_path());

    Ok(
        Response::builder()
            .status(302)
            .header("Location", &redirect_url)
            .body(Body::Empty)
            .map_err(Box::new)?
    )
}
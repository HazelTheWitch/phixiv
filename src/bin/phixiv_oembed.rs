use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(redirect_handler)).await
}

async fn redirect_handler(request: Request) -> Result<Response<Body>, Error> {
    Ok(
        Response::builder()
            .status(200)
            .body("oembed up".into())
            .map_err(Box::new)?
    )
}
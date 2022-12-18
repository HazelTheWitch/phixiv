use lambda_http::{run, service_fn, Body, Error, Request, Response, RequestExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(phixiv_handler)).await
}

async fn phixiv_handler(event: Request) -> Result<Response<Body>, Error> {
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(event.raw_http_path().into())
        .map_err(Box::new)?;
    
    Ok(resp)
}
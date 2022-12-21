use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use phixiv::pixiv_url::PixivPath;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(phixiv_handler)).await
}

#[inline(always)]
fn error_redirect(pixiv_url: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(302)
        .header("Location", pixiv_url)
        .body(Body::Empty)
        .map_err(Box::new)?)
}

async fn generate_html(path: String) -> Result<Response<Body>, Error> {
    let pixiv_path = PixivPath::parse(&path)?;

    let artwork = pixiv_path.resolve().await?;

    let html = artwork.to_html()?;

    Ok(
        Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(html.into())
            .map_err(Box::new)?
    )
}

async fn phixiv_handler(event: Request) -> Result<Response<Body>, Error> {
    let pixiv_path = event.raw_http_path();
    let pixiv_url = format!("https://pixiv.net{}", &pixiv_path);

    match generate_html(pixiv_path).await {
        Ok(response) => Ok(response),
        Err(_) => error_redirect(&pixiv_url),
    }
}

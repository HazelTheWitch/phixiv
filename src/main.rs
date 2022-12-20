use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use phixiv::artwork::Artwork;

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

async fn phixiv_handler(event: Request) -> Result<Response<Body>, Error> {
    let pixiv_url = format!("https://pixiv.net{}", event.raw_http_path());

    let body = reqwest::get(&pixiv_url).await?.text().await?;

    let Some(artwork) = Artwork::parse(body) else {
        return error_redirect(&pixiv_url);
    };

    let Some(html) = artwork.to_html() else {
        return error_redirect(&pixiv_url);
    };

    let resp = Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .header("Referer", "http://www.pixiv.net/")
        .body(html.into())
        .map_err(Box::new)?;

    Ok(resp)
}

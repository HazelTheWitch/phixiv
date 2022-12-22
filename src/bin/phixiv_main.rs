use isbot::Bots;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use phixiv::pixiv::artwork::Artwork;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(phixiv_handler)).await
}

#[inline(always)]
fn redirect(pixiv_url: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(302)
        .header("Location", pixiv_url)
        .body(Body::Empty)
        .map_err(Box::new)?)
}

async fn generate_html(path: String) -> Result<Response<Body>, Error> {
    let artwork = Artwork::from_path(&path).await?;

    let html = artwork.render_minified()?;

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(html.into())
        .map_err(Box::new)?)
}

async fn phixiv_handler(request: Request) -> Result<Response<Body>, Error> {
    let pixiv_path = request.raw_http_path();
    let pixiv_url = format!("https://pixiv.net{}", &pixiv_path);

    let bots = Bots::default();

    if let Some(Ok(user_agent)) = request.headers().get("User-Agent").map(|ua| ua.to_str()) {
        if !bots.is_bot(user_agent) {
            return redirect(&pixiv_url);
        }
    }

    match generate_html(pixiv_path).await {
        Ok(response) => Ok(response),
        Err(err) => {
            tracing::error!("{}", err);
            redirect(&pixiv_url)
        },
    }
}

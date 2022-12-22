#![feature(iter_intersperse)]

use isbot::Bots;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use phixiv::pixiv::{artwork::Artwork, PixivPath};

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

async fn generate_html(path: PixivPath) -> Result<Response<Body>, Error> {
    let artwork = Artwork::from_path(path).await?;

    let html = artwork.render_minified()?;

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(html.into())
        .map_err(Box::new)?)
}

fn get_path(request: &Request) -> String {
    let base_path = request.raw_http_path();
    let mut query_string: String = request
        .query_string_parameters()
        .iter()
        .map(|(k, v)| {
            format!("{}={}", urlencoding::encode(k), urlencoding::encode(v))
        })
        .intersperse(String::from("&"))
        .collect();
    
    if !query_string.is_empty() {
        query_string = ["?", &query_string].concat();
    }
    
    [base_path, query_string].concat()
}

async fn phixiv_handler(request: Request) -> Result<Response<Body>, Error> {
    let raw_path = get_path(&request);

    let pixiv_url = format!("https://www.pixiv.net{}", raw_path);

    let Ok(pixiv_path) = PixivPath::parse(&raw_path) else {
        return redirect(&pixiv_url);
    };

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

use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use scraper::{Html, Selector};
use serde::Serialize;
use tera::{Context, Tera};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(phixiv_handler)).await
}

#[derive(Debug, Serialize)]
struct Artwork {
    pub image_url: String,
    pub title: String,
    pub description: String,
    pub url: String,
}

macro_rules! selector {
    ($document: expr, $selector: expr, $attr: expr) => {
        { $document.select(&Selector::parse($selector).unwrap()).map(|e| e.value().attr($attr)) }
    };
}

impl Artwork {
    pub fn parse(body: String) -> Option<Self> {
        let document = Html::parse_document(&body);

        Some(Self {
            image_url: selector!(document, r#"link[rel=preload][as=image]"#, "href").next()??.to_string(),
            title: selector!(document, r#"meta[property="og:title"]"#, "content").next()??.to_string(),
            description: selector!(document, r#"meta[property="twitter:description"]"#, "content").next()??.to_string(),
            url: selector!(document, r#"meta[property="twitter:url"]"#, "content").next()??.to_string(),
        })
    }

    pub fn to_html(&self) -> Option<String> {
        let mut tera = Tera::default();
        tera.add_raw_template("artwork.html", include_str!("../templates/artwork.html"))
            .unwrap();
        tera.autoescape_on(vec![]);

        tera.render("artwork.html", &Context::from_serialize(self).ok()?)
            .ok()
    }
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
        .header("content-type", "text/html")
        .header("Referer", "http://www.pixiv.net/")
        .body(html.into())
        .map_err(Box::new)?;

    Ok(resp)
}

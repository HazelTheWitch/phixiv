use lambda_http::{run, service_fn, Error, Request, RequestExt};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(oembed_handler)).await
}

#[derive(Debug, Serialize)]
struct OembedResponse {
    version: String,
    #[serde(rename = "type")]
    oembed_type: String,
    author_name: String,
    author_url: String,
    provider_name: String,
    provider_url: String,
}

impl From<PixivOembedResponse> for OembedResponse {
    fn from(por: PixivOembedResponse) -> Self {
        Self {
            version: "1.0".to_string(),
            oembed_type: "rich".to_string(),
            author_name: por.author_name,
            author_url: por.author_url,
            provider_name: "phixiv".into(),
            provider_url: "https://github.com/HazelTheWitch/phixiv".into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PixivOembedResponse {
    author_name: String,
    author_url: String,
}

async fn oembed_handler(request: Request) -> Result<(StatusCode, serde_json::Value), Error> {
    let Some(url) = request.query_string_parameters().iter().find_map(|(name, value)| if name == "url" { Some(value.to_string()) } else { None }) else {
        return Ok((StatusCode::NOT_FOUND, json!({})));
    };

    let Ok(url_object) = url::Url::parse(&url) else {
        return Ok((StatusCode::NOT_FOUND, json!({})));
    };

    if url_object.host_str() != Some("www.pixiv.net") {
        return Ok((StatusCode::NOT_FOUND, json!({})));
    }

    let por = reqwest::get(format!("https://embed.pixiv.net/oembed.php?url={}", urlencoding::encode(&url))).await?.json::<PixivOembedResponse>().await?;

    Ok((StatusCode::OK, serde_json::value::to_value::<OembedResponse>(por.into())?))
}

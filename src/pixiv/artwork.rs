use std::{collections::HashMap, env, string::FromUtf8Error};

use askama::Template;
use http::{HeaderMap, HeaderValue};
use minify_html::{minify, Cfg};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    payloads::{AjaxResponse, AppReponse},
    PixivError,
};

const ILLUST_URL: &str = "https://app-api.pixiv.net/v1/illust/detail";

#[derive(Debug, Error)]
pub enum ArtworkError {
    #[error("minifying error")]
    Minify(#[from] FromUtf8Error),
    #[error("templating error")]
    Templating(#[from] askama::Error),
    #[error("image url parsing error")]
    Parsing(#[from] url::ParseError),
    #[error("invalid image url")]
    ImageURL,
}

#[derive(Deserialize)]
pub struct ArtworkPath {
    pub language: Option<String>,
    pub id: String,
}

#[derive(Debug, Serialize, Template)]
#[template(path = "artwork.html")]
pub struct Artwork {
    pub image_proxy_url: String,
    pub title: String,
    pub description: String,
    pub author_name: String,
    pub author_id: String,
    pub url: String,
    pub alt_text: String,
    pub embed_url: String,
}

impl Artwork {
    pub fn render_minified(&self) -> Result<String, ArtworkError> {
        let html = self.render()?;

        let mut cfg = Cfg::new();
        cfg.do_not_minify_doctype = true;
        cfg.ensure_spec_compliant_unquoted_attribute_values = true;
        cfg.keep_spaces_between_attributes = true;

        let minified = minify(html.as_bytes(), &cfg);

        Ok(String::from_utf8(minified)?)
    }

    pub fn image_proxy_url(url: &str) -> Result<String, ArtworkError> {
        let url = url::Url::parse(url)?;

        Ok(format!("{}{}", env::var("PROXY_URL").unwrap(), url.path()))
    }

    async fn app_request(
        client: &Client,
        path: &ArtworkPath,
        access_token: &str,
    ) -> Result<AppReponse, PixivError> {
        let params = HashMap::from([("illust_id", &path.id)]);

        let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(10);

        headers.append("app-os", "ios".parse().unwrap());
        headers.append("app-os-version", "14.6".parse().unwrap());
        headers.append(
            "user-agent",
            "PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)".parse().unwrap(),
        );
        headers.append("host", "app-api.pixiv.net".parse().unwrap());
        headers.append(
            "Authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );

        Ok(client
            .get(ILLUST_URL)
            .headers(headers)
            .query(&params)
            .send()
            .await?
            .json()
            .await?)
    }

    async fn ajax_request(client: &Client, path: &ArtworkPath) -> Result<AjaxResponse, PixivError> {
        let ajax_url = format!(
            "https://www.pixiv.net/ajax/illust/{}?lang={}",
            &path.id,
            &path.language.clone().unwrap_or_else(|| "jp".to_owned())
        );

        Ok(client.get(ajax_url).send().await?.json().await?)
    }

    pub async fn from_path(path: ArtworkPath, access_token: &str) -> Result<Self, PixivError> {
        let client = Client::new();

        let (app, ajax) = tokio::join!(
            Artwork::app_request(&client, &path, &access_token),
            Artwork::ajax_request(&client, &path),
        );

        let app_response = app?;
        let ajax_response = ajax?;

        let body = ajax_response.body;

        let description = if body.description.is_empty() {
            body.alt.clone()
        } else {
            body.description
        };

        Ok(Self {
            #[cfg(feature = "small_images")]
            image_proxy_url: Artwork::image_proxy_url(&app_response.illust.image_urls.medium)?,
            #[cfg(not(feature = "small_images"))]
            image_proxy_url: Artwork::image_proxy_url(&app_response.illust.image_urls.large)?,
            title: body.title,
            description,
            url: body.extra_data.meta.canonical,
            alt_text: body.alt,
            author_name: body.author_name,
            author_id: body.author_id,
            embed_url: env::var("EMBED_URL").unwrap(),
        })
    }
}

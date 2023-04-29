use std::{collections::HashMap, env, string::FromUtf8Error};

use askama::Template;
use http::{HeaderMap, HeaderValue};
use minify_html::{minify, Cfg};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::instrument;

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

#[derive(Deserialize, Debug)]
pub struct RawArtworkPath {
    pub language: Option<String>,
    pub id: String,
    pub image_index: Option<String>,
}

impl RawArtworkPath {
    pub fn parse(self) -> ArtworkPath {
        let Some(image_index) = &self.image_index else {
            return ArtworkPath { language: self.language, id: self.id, image_index: None };
        };

        match image_index.parse::<usize>() {
            Ok(image_index) => ArtworkPath {
                language: self.language,
                id: self.id,
                image_index: Some(image_index),
            },
            Err(_) => ArtworkPath {
                language: self.language,
                id: self.id,
                image_index: None,
            },
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ArtworkPath {
    pub language: Option<String>,
    pub id: String,
    pub image_index: Option<usize>,
}

impl ArtworkPath {
    pub fn format_path(&self) -> String {
        match &self.language {
            Some(lang) => format!("/{lang}/artworks/{}", self.id),
            None => format!("/artworks/{}", self.id),
        }
    }
}

pub struct ImageUrl {
    pub image_proxy_url: String,
    pub image_proxy_path: String,
}

#[derive(Debug, Serialize, Template)]
#[template(path = "artwork.html")]
pub struct Artwork {
    pub image_proxy_url: String,
    pub image_proxy_path: String,
    pub title: String,
    pub description: String,
    pub author_name: String,
    pub author_id: String,
    pub url: String,
    pub alt_text: String,
    pub host: String,
}

impl Artwork {
    #[instrument(skip(self))]
    pub fn render_minified(&self) -> Result<String, ArtworkError> {
        let html = self.render()?;

        let mut cfg = Cfg::new();
        cfg.do_not_minify_doctype = true;
        cfg.ensure_spec_compliant_unquoted_attribute_values = true;
        cfg.keep_spaces_between_attributes = true;

        let minified = minify(html.as_bytes(), &cfg);

        Ok(String::from_utf8(minified)?)
    }

    pub fn image_proxy_url(url: &str) -> Result<(String, String), ArtworkError> {
        let url = url::Url::parse(url)?;

        Ok((
            format!("{}/i{}", env::var("RAILWAY_STATIC_URL").unwrap(), url.path()),
            url.path().split_at(1).1.to_owned(),
        ))
    }

    #[instrument(skip(client, access_token))]
    async fn app_request(
        client: &Client,
        path: &ArtworkPath,
        access_token: &str,
    ) -> Result<AppReponse, PixivError> {
        let params = HashMap::from([("illust_id", &path.id)]);

        let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(5);

        headers.append("app-os", "ios".parse().unwrap());
        headers.append("app-os-version", "14.6".parse().unwrap());
        headers.append(
            "user-agent",
            "PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)".parse().unwrap(),
        );
        headers.append("host", "app-api.pixiv.net".parse().unwrap());
        headers.append(
            "Authorization",
            format!("Bearer {access_token}").parse().unwrap(),
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

    #[instrument(skip(client))]
    async fn ajax_request(client: &Client, path: &ArtworkPath) -> Result<AjaxResponse, PixivError> {
        let ajax_url = format!(
            "https://www.pixiv.net/ajax/illust/{}?lang={}",
            &path.id,
            &path.language.clone().unwrap_or_else(|| "jp".to_owned())
        );

        Ok(client.get(ajax_url).send().await?.json().await?)
    }

    #[instrument(skip(access_token))]
    pub async fn get_image_url(
        client: &Client,
        path: &ArtworkPath,
        access_token: &str,
    ) -> Result<ImageUrl, PixivError> {
        let app_response = Artwork::app_request(client, path, access_token).await?;

        #[cfg(feature = "small_images")]
        let (image_proxy_url, image_proxy_path) =
            Artwork::image_proxy_url(&app_response.illust.image_urls.large)?;
        #[cfg(not(feature = "small_images"))]
        let (image_proxy_url, image_proxy_path) = Artwork::image_proxy_url(&{
            match app_response.illust.meta_single_page.original_image_url {
                Some(url) => url,
                None => {
                    let pages = app_response.illust.meta_pages;
                    match pages.get(
                        path.image_index
                            .unwrap_or(1)
                            .min(pages.len())
                            .saturating_sub(1),
                    ) {
                        Some(meta_page) => meta_page.image_urls.original.clone(),
                        None => app_response.illust.image_urls.large.clone(),
                    }
                }
            }
        })?;

        Ok(ImageUrl {
            image_proxy_url,
            image_proxy_path,
        })
    }

    #[instrument(skip(access_token))]
    pub async fn from_path(path: &ArtworkPath, access_token: &str) -> Result<Self, PixivError> {
        let client = Client::new();

        let (image_url, ajax) = tokio::join!(
            Artwork::get_image_url(&client, &path, access_token),
            Artwork::ajax_request(&client, &path),
        );

        let ImageUrl {
            image_proxy_url,
            image_proxy_path,
        } = image_url?;
        let ajax_response = ajax?;

        let body = ajax_response.body;

        let tag_string = body
            .tags
            .tags
            .into_iter()
            .map(|tag| {
                format!("#{}", if let Some(language) = &path.language {
                    if let Some(translation) = tag.translation {
                        translation.get(language).unwrap_or(&tag.tag).to_string()
                    } else {
                        tag.tag
                    }
                } else {
                    tag.tag
                })
            })
            .intersperse_with(|| String::from(", "))
            .collect::<String>();

        let description = if body.description.is_empty() {
            tag_string.clone()
        } else {
            body.description
        };

        Ok(Self {
            image_proxy_url,
            image_proxy_path,
            title: body.title,
            description,
            url: body.extra_data.meta.canonical,
            alt_text: tag_string,
            author_name: body.author_name,
            author_id: body.author_id,
            host: env::var("RAILWAY_STATIC_URL").unwrap(),
        })
    }
}

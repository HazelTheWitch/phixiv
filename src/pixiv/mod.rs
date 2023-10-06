use std::collections::HashMap;

use askama::Template;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use itertools::Itertools;

use crate::helper;

use self::model::{AppReponse, AjaxResponse};

mod model;

const ILLUST_URL: &str = "https://app-api.pixiv.net/v1/illust/detail";

#[derive(Deserialize)]
pub struct RawArtworkPath {
    pub language: Option<String>,
    pub id: String,
    pub image_index: Option<String>,
}

pub struct ArtworkPath {
    pub language: Option<String>,
    pub id: String,
    pub image_index: Option<usize>,
}

impl TryFrom<RawArtworkPath> for ArtworkPath {
    type Error = anyhow::Error;

    fn try_from(value: RawArtworkPath) -> Result<Self, Self::Error> {
        let image_index = match value.image_index {
            Some(index) => Some(index.parse()?),
            None => None,
        };

        Ok(Self { language: value.language, id: value.id, image_index })
    }
}

#[derive(Debug, Serialize, Template)]
#[template(path = "artwork.html")]
pub struct ArtworkTemplate {
    pub image_proxy_url: String,
    pub title: String,
    pub description: String,
    pub author_name: String,
    pub author_id: String,
    pub url: String,
    pub alt_text: String,
    pub host: String,
}

#[derive(Serialize)]
/// Representing a listing of artworks, uniquely determined by language and illust_id
pub struct ArtworkListing {
    pub image_proxy_urls: Vec<String>,
    pub title: String,
    pub ai_generated: bool,
    pub description: String,
    pub tags: Vec<String>,
    pub url: String,
    pub author_name: String,
    pub author_id: String,
}

async fn app_request(illust_id: &String, access_token: &str, client: &Client) -> anyhow::Result<AppReponse> {
    let app_params = HashMap::from([("illust_id", illust_id)]);
    let mut app_headers = helper::headers();
    app_headers.append("Host", "app-api.pixiv.net".parse()?);
    app_headers.append("Authorization", format!("Bearer {access_token}").parse()?);

    Ok(client
        .get(ILLUST_URL)
        .headers(app_headers)
        .query(&app_params)
        .send()
        .await?
        .json()
        .await?)
}

async fn ajax_request(illust_id: &String, language: &Option<String>, client: &Client) -> anyhow::Result<AjaxResponse> {
    Ok(client
        .get(format!("https://www.pixiv.net/ajax/illust/{}?lang={}", &illust_id, &language.clone().unwrap_or_else(|| String::from("jp"))))
        .send()
        .await?
        .json()
        .await?)
}

impl ArtworkListing {
    pub async fn get_listing(
        language: Option<String>,
        illust_id: String,
        access_token: &str,
        host: &str,
        client: &Client,
    ) -> anyhow::Result<Self> {
        let (app_response, ajax_response) = tokio::join!(
            app_request(&illust_id, &access_token, &client),
            ajax_request(&illust_id, &language, &client),
        );

        let illust_response = app_response?.illust;
        let ajax_body = ajax_response?.body;

        let ai_generated = illust_response.illust_ai_type == 2;

        let tags: Vec<_> = ajax_body.tags.tags
            .into_iter()
            .map(|tag| {
                format!(
                    "#{}",
                    if let Some(language) = &language {
                        if let Some(translation) = tag.translation {
                            translation.get(language).unwrap_or(&tag.tag).to_string()
                        } else {
                            tag.tag
                        }
                    } else {
                        tag.tag
                    }
                )
            })
            .collect();

        let image_proxy_urls = if illust_response.meta_pages.len() == 0 {
            let url = url::Url::parse(&illust_response.image_urls.large)?;

            vec![format!("https://{}/i{}", host, url.path())]
        } else {
            illust_response.meta_pages
                .into_iter()
                .map(|mp| {
                    let url = url::Url::parse(&mp.image_urls.large)?;

                    Ok(format!("https://{}/i{}", host, url.path()))
                })
                .collect::<anyhow::Result<Vec<String>>>()?
        };

        Ok(Self {
            image_proxy_urls,
            title: ajax_body.title,
            ai_generated,
            description: ajax_body.description,
            tags,
            url: ajax_body.extra_data.meta.canonical,
            author_name: ajax_body.author_name,
            author_id: ajax_body.author_id,
        })
    }

    pub fn to_template(self, image_index: Option<usize>, host: String) -> ArtworkTemplate {
        let index = image_index
            .unwrap_or(1)
            .min(self.image_proxy_urls.len())
            .saturating_sub(1);

        let image_proxy_url = self.image_proxy_urls[index].clone();

        let tag_string = Itertools::intersperse_with(self.tags.into_iter().map(|t| format!("#{t}")), || String::from(", ")).collect::<String>();

        let description = Itertools::intersperse_with([
                String::from(if self.ai_generated { "AI Generated\n" } else { "" }),
                self.description,
                tag_string.clone(),
            ]
            .into_iter()
            .filter(|s| !s.is_empty()), || String::from("\n"))
            .collect::<String>();

        ArtworkTemplate {
            image_proxy_url,
            title: self.title,
            description,
            author_name: self.author_name,
            author_id: self.author_id,
            url: self.url, 
            alt_text: tag_string,
            host
        }
    }
}

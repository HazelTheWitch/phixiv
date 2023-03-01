use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppReponse {
    pub illust: IllustrationResponse,
}

#[derive(Debug, Deserialize)]
pub struct IllustrationResponse {
    pub image_urls: ImageUrls,
    pub meta_single_page: MetaSinglePage,
    pub meta_pages: Vec<MetaPage>,
    pub illust_ai_type: u8,
}

#[derive(Debug, Deserialize)]
pub struct MetaSinglePage {
    pub original_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaPage {
    pub image_urls: MetaPageImageUrls,
}

#[derive(Debug, Deserialize)]
pub struct MetaPageImageUrls {
    pub original: String,
}

#[derive(Debug, Deserialize)]
pub struct ImageUrls {
    pub large: String,
    pub medium: String,
}

#[derive(Debug, Deserialize)]
pub struct AjaxResponse {
    pub body: AjaxBody,
}

#[derive(Debug, Deserialize)]
pub struct AjaxBody {
    pub title: String,
    pub description: String,
    pub tags: Tags,
    #[serde(rename = "userId")]
    pub author_id: String,
    #[serde(rename = "userName")]
    pub author_name: String,
    #[serde(rename = "extraData")]
    pub extra_data: AjaxExtraData,
}

#[derive(Debug, Deserialize)]
pub struct Tags {
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub tag: String,
    pub translation: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct AjaxExtraData {
    pub meta: AjaxMeta,
}

#[derive(Debug, Deserialize)]
pub struct AjaxMeta {
    pub canonical: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub response: AuthResponse,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

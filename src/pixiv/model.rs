use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct AppReponse {
    pub illust: IllustrationResponse,
}

#[derive(Debug, Deserialize)]
pub(super) struct IllustrationResponse {
    pub image_urls: ImageUrls,
    pub meta_pages: Vec<MetaPage>,
    pub illust_ai_type: u8,
}

#[derive(Debug, Deserialize)]
pub(super) struct MetaPage {
    pub image_urls: MetaPageImageUrls,
}

#[derive(Debug, Deserialize)]
pub(super) struct MetaPageImageUrls {
    pub large: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ImageUrls {
    pub large: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct AjaxResponse {
    pub body: AjaxBody,
}

#[derive(Debug, Deserialize)]
pub(super) struct AjaxBody {
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
pub(super) struct Tags {
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Tag {
    pub tag: String,
    pub translation: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub(super) struct AjaxExtraData {
    pub meta: AjaxMeta,
}

#[derive(Debug, Deserialize)]
pub(super) struct AjaxMeta {
    pub canonical: String,
}

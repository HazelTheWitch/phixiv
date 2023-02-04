use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PixivResponse {
    pub body: PixivBody,
}

#[derive(Debug, Deserialize)]
pub struct PixivBody {
    pub title: String,
    pub description: String,
    pub alt: String,
    #[serde(rename = "userId")]
    pub author_id: String,
    #[serde(rename = "userName")]
    pub author_name: String,
    #[serde(rename = "extraData")]
    pub extra_data: PixivExtraData,
}

#[derive(Debug, Deserialize)]
pub struct PixivExtraData {
    pub meta: PixivMeta,
}

#[derive(Debug, Deserialize)]
pub struct PixivMeta {
    pub canonical: String,
}

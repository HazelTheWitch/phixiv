use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppReponse {
    pub illust: IllustrationResponse,
}

#[derive(Deserialize)]
pub struct IllustrationResponse {
    pub image_urls: ImageUrls,
}

#[derive(Deserialize)]
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
    pub alt: String,
    #[serde(rename = "userId")]
    pub author_id: String,
    #[serde(rename = "userName")]
    pub author_name: String,
    #[serde(rename = "extraData")]
    pub extra_data: AjaxExtraData,
}

#[derive(Debug, Deserialize)]
pub struct AjaxExtraData {
    pub meta: AjaxMeta,
}

#[derive(Debug, Deserialize)]
pub struct AjaxMeta {
    pub canonical: String,
}

#[derive(Deserialize)]
pub struct AuthPayload {
    pub response: AuthResponse,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

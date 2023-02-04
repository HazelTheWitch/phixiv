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
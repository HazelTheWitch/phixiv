use serde::Serialize;
use tera::{Context, Tera};

use crate::pixiv_url::PixivResponse;

#[derive(Debug, Serialize)]
pub struct Artwork {
    pub image_url: String,
    pub title: String,
    pub description: String,
    pub author_name: String,
    pub url: String,
    pub alt_text: String,
}

impl Artwork {
    pub fn to_html(&self) -> Result<String, tera::Error> {
        let mut tera = Tera::default();
        tera.add_raw_template("artwork.html", include_str!("../templates/artwork.html"))
            .unwrap();
        tera.autoescape_on(vec![]);

        tera.render("artwork.html", &Context::from_serialize(self)?)
    }
}

impl From<PixivResponse> for Artwork {
    fn from(response: PixivResponse) -> Self {
        let body = response.body;

        let description = if body.description.len() > 0 {
            body.description
        } else {
            body.alt.to_owned()
        };

        Self { image_url: body.urls.small, title: body.title, description: description, url: body.extra_data.meta.canonical, alt_text: body.alt, author_name: body.author_name }
    }
}

#[cfg(test)]
mod tests {
    use crate::pixiv_url::PixivPath;

    #[tokio::test]
    async fn test_formatting() {
        let path = "/en/artworks/101595682";

        let pixiv_path = PixivPath::parse(&path).unwrap();

        let artwork = pixiv_path.resolve().await.unwrap();

        let html = artwork.to_html().unwrap();

        assert!(html.len() > 0);
    }
}
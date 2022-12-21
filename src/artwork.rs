use std::{string::FromUtf8Error, env};

use askama::Template;
use minify_html::{Cfg, minify};
use serde::Serialize;
use thiserror::Error;

use crate::pixiv_url::PixivResponse;

#[derive(Debug, Error)]
pub enum ArtworkError {
    #[error("minifying error")]
    Minify(#[from] FromUtf8Error),
    #[error("templating error")]
    Templating(#[from] askama::Error),
}

#[derive(Debug, Serialize, Template)]
#[template(path = "artwork.html")]
pub struct Artwork {
    pub image_url: String,
    pub title: String,
    pub description: String,
    pub author_name: String,
    pub url: String,
    pub alt_text: String,
    embed_url: String,
    proxy_url: String,
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
}

impl From<PixivResponse> for Artwork {
    fn from(response: PixivResponse) -> Self {
        let body = response.body;

        let description = if !body.description.is_empty() {
            body.description
        } else {
            body.alt.to_string()
        };

        Self {
            #[cfg(feature = "small_images")]
            image_url: body.urls.small,
            #[cfg(not(feature = "small_images"))]
            image_url: body.urls.regular,
            title: body.title,
            description,
            url: body.extra_data.meta.canonical,
            alt_text: body.alt,
            author_name: body.author_name,
            embed_url: env::var("EMBED_URL").unwrap(),
            proxy_url: env::var("PROXY_URL").unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use askama::Template;

    use crate::pixiv_url::PixivPath;

    #[tokio::test]
    async fn test_formatting() {
        env::set_var("EMBED_URL", "EMBED_URL");
        env::set_var("PROXY_URL", "PROXY_URL");

        let path = "/en/artworks/101595682";

        let pixiv_path = PixivPath::parse(&path).unwrap();

        let artwork = pixiv_path.resolve().await.unwrap();

        let html = artwork.render().unwrap();

        println!("{}", html);

        assert!(html.len() > 0);
    }
}

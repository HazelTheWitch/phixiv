use std::{env, string::FromUtf8Error};

use askama::Template;
use minify_html::{minify, Cfg};
use serde::Serialize;
use thiserror::Error;

use crate::pixiv_url::PixivResponse;

#[derive(Debug, Error)]
pub enum ArtworkError {
    #[error("minifying error")]
    Minify(#[from] FromUtf8Error),
    #[error("templating error")]
    Templating(#[from] askama::Error),
    #[error("image url parsing error")]
    Parsing(#[from] url::ParseError),
}

#[derive(Debug, Serialize, Template)]
#[template(path = "artwork.html")]
pub struct Artwork {
    image_proxy_url: String,
    title: String,
    description: String,
    author_name: String,
    author_id: String,
    url: String,
    alt_text: String,
    embed_url: String,
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

    pub fn format_image_proxy_url(url: String) -> Result<String, ArtworkError> {
        let url = url::Url::parse(&url)?;

        let proxy_url = url::Url::parse(&env::var("PROXY_URL").unwrap())?;

        Ok(proxy_url.join(url.path())?.to_string())
    }
}

impl TryFrom<PixivResponse> for Artwork {
    type Error = ArtworkError;

    fn try_from(response: PixivResponse) -> Result<Self, Self::Error> {
        let body = response.body;

        let description = if !body.description.is_empty() {
            body.description
        } else {
            body.alt.to_string()
        };

        Ok(Self {
            #[cfg(feature = "small_images")]
            image_proxy_url: Artwork::format_image_proxy_url(body.urls.small)?,
            #[cfg(not(feature = "small_images"))]
            image_proxy_url: Artwork::format_image_proxy_url(body.urls.regular)?,
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

#[cfg(test)]
mod tests {
    use std::env;

    use askama::Template;

    use crate::pixiv_url::PixivPath;

    #[tokio::test]
    async fn test_formatting() {
        env::set_var("EMBED_URL", "https://e.phixiv.net/");
        env::set_var("PROXY_URL", "https://i.phixiv.net/");

        let path = "/en/artworks/101595682";

        let pixiv_path = PixivPath::parse(&path).unwrap();

        let artwork = pixiv_path.resolve().await.unwrap();

        let html = artwork.render().unwrap();

        println!("{}", html);

        assert!(html.len() > 0);
    }
}

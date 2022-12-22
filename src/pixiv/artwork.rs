use std::{env, string::FromUtf8Error};

use askama::Template;
use minify_html::{minify, Cfg};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArtworkError {
    #[error("minifying error")]
    Minify(#[from] FromUtf8Error),
    #[error("templating error")]
    Templating(#[from] askama::Error),
    #[error("image url parsing error")]
    Parsing(#[from] url::ParseError),
    #[error("missing environment variable in lambda")]
    EnvironmentVariable(&'static str),
}

#[derive(Debug, Serialize, Template)]
#[template(path = "artwork.html")]
pub struct Artwork {
    pub image_proxy_url: String,
    pub title: String,
    pub description: String,
    pub author_name: String,
    pub author_id: String,
    pub url: String,
    pub alt_text: String,
    pub embed_url: String,
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

    pub fn format_image_proxy_url(url: &str) -> Result<String, ArtworkError> {
        let url = url::Url::parse(url)?;

        let proxy_url = url::Url::parse(
            &env::var("PROXY_URL").or(Err(ArtworkError::EnvironmentVariable("PROXY_URL")))?,
        )?;

        Ok(proxy_url.join(url.path())?.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use askama::Template;

    use crate::pixiv::{artwork::Artwork, PixivPath};

    #[tokio::test]
    async fn test_formatting() {
        env::set_var("EMBED_URL", "https://e.phixiv.net/");
        env::set_var("PROXY_URL", "https://i.phixiv.net/");

        let path = "/en/artworks/101595682";

        let artwork = Artwork::from_path(PixivPath::parse(path).unwrap()).await.unwrap();

        let html = artwork.render().unwrap();

        println!("{}", html);

        assert!(html.len() > 0);
    }
}

use std::{env, string::FromUtf8Error};

use askama::Template;
use minify_html::{minify, Cfg};
use regex::Regex;
use serde::Serialize;
use thiserror::Error;
use lazy_static::lazy_static;

lazy_static! {
    static ref IMAGE_RE: Regex = Regex::new(r#"^(.)\.pximg\.net$"#).unwrap();
}

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
    #[error("invalid image url")]
    ImageURL,
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

        let proxy_url = env::var("PROXY_URL").or(Err(ArtworkError::EnvironmentVariable("PROXY_URL")))?;

        let base = match url.host_str() {
            Some(s) => {
                let captures = IMAGE_RE.captures(s).ok_or(ArtworkError::ImageURL)?;

                Ok(captures.get(1).map(|m| m.as_str()).ok_or(ArtworkError::ImageURL)?)
            }
            None => Err(ArtworkError::ImageURL),
        }?;

        Ok(format!("{}?p={}&b={}", proxy_url, urlencoding::encode(url.path()), urlencoding::encode(base)))
    }
}

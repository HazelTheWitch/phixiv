use scraper::{Html, Selector};
use serde::Serialize;
use tera::{Context, Tera};

#[derive(Debug, Serialize)]
pub struct Artwork {
    pub image_url: String,
    pub title: String,
    pub description: String,
    pub url: String,
}

macro_rules! selector {
    ($document: expr, $selector: expr, $attr: expr) => {
        { $document.select(&Selector::parse($selector).unwrap()).map(|e| e.value().attr($attr)) }
    };
}

impl Artwork {
    pub fn parse(body: String) -> Option<Self> {
        let document = Html::parse_document(&body);

        Some(Self {
            image_url: selector!(document, r#"link[rel=preload][as=image]"#, "href").next()??.to_string(),
            title: selector!(document, r#"meta[property="twitter:title"]"#, "content").next()??.to_string(),
            description: selector!(document, r#"meta[property="twitter:description"]"#, "content").next()??.to_string(),
            url: selector!(document, r#"meta[property="twitter:url"]"#, "content").next()??.to_string(),
        })
    }

    pub fn to_html(&self) -> Option<String> {
        let mut tera = Tera::default();
        tera.add_raw_template("artwork.html", include_str!("../templates/artwork.html"))
            .unwrap();
        tera.autoescape_on(vec![]);

        tera.render("artwork.html", &Context::from_serialize(self).ok()?)
            .ok()
    }
}
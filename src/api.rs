use std::sync::Arc;

use axum::{Router, middleware, Json, extract::{State, Query, Host}, routing::get, response::{Response, IntoResponse}};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

use crate::{PhixivState, auth_middleware, pixiv::artwork::Artwork};

#[derive(Serialize, Debug)]
pub struct ArtworkInfo {
    pub urls: Vec<String>,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub author: AuthorInfo,
}

#[derive(Serialize, Debug)]
pub struct AuthorInfo {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct ArtworkInfoPath {
    pub language: Option<String>,
    pub id: String,
}

pub async fn artwork_info_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    Query(path): Query<ArtworkInfoPath>,
    Host(host): Host,
) -> Result<Json<ArtworkInfo>, Response> {
    let state = state.read().await;

    let client = Client::new();

    let (ajax_response, app_response) = tokio::join!(
        Artwork::ajax_request(&client, path.id.clone(), path.language.clone()),
        Artwork::app_request(&client, path.id.clone(), &state.auth.access_token),
    );

    let ajax_response = ajax_response.map_err(|e| e.into_response())?;
    let app_response = app_response.map_err(|e| e.into_response())?;

    let urls = if app_response.illust.meta_pages.len() == 0 {
        vec![Artwork::image_proxy_url(&app_response.illust.image_urls.large, &host).unwrap().0]
    } else {
        app_response.illust.meta_pages
            .into_iter()
            .map(|meta_page|
                Artwork::image_proxy_url(&meta_page.image_urls.large, &host).unwrap().0
            )
            .collect()
    };
    
    Ok(Json(ArtworkInfo {
        urls,
        title: ajax_response.body.title,
        description: ajax_response.body.description,
        tags: ajax_response.body.tags.tags.into_iter().map(|tag| tag.tag).collect(),
        author: AuthorInfo {
            id: ajax_response.body.author_id,
            name: ajax_response.body.author_name
        },
    }))
}

pub fn api_router(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .route("/info", get(artwork_info_handler))
        .with_state(state.clone())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}
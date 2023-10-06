use std::sync::Arc;

use axum::{extract::{State, Query, Host}, Json};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{state::PhixivState, helper::PhixivError, pixiv::ArtworkListing};

#[derive(Deserialize)]
pub struct ArtworkInfoPath {
    pub language: Option<String>,
    pub id: String,
}

pub(super) async fn artwork_info_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    Query(path): Query<ArtworkInfoPath>,
    Host(host): Host,
) -> Result<Json<ArtworkListing>, PhixivError> {
    let state = state.read().await;

    Ok(Json(ArtworkListing::get_listing(path.language, path.id, &state.auth.access_token, &host, &state.client).await?))
}
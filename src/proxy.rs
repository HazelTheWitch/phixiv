use std::{sync::Arc, time::Duration};

use axum::{extract::{Path, State}, Router, middleware, routing::get, response::IntoResponse, TypedHeader, headers::CacheControl, body::StreamBody};
use tokio::sync::RwLock;

use crate::{state::{PhixivState, authorized_middleware}, helper::{PhixivError, self}};

async fn proxy_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, PhixivError> {
    let state = state.read().await;

    let url = format!("https://i.pximg.net/{path}");

    let mut headers = helper::headers();
    headers.append("Referer", "https://www.pixiv.net/".parse()?);
    headers.append("Authorization", format!("Bearer {}", state.auth.access_token).parse()?);

    let response = state.client.get(&url).headers(headers).send().await?;

    Ok((
        TypedHeader(
            CacheControl::new()
                .with_max_age(Duration::from_secs(60 * 60 * 24))
                .with_public()
        ),
        StreamBody::new(response.bytes_stream())
    ))
}

pub fn proxy_router(state: Arc<RwLock<PhixivState>>) -> Router<Arc<RwLock<PhixivState>>> {
    Router::new()
        .route("/*path", get(proxy_handler))
        .layer(middleware::from_fn_with_state(state, authorized_middleware))
}
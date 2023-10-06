use std::{sync::Arc, time::Duration};

use axum::{
    body::StreamBody,
    extract::{Path, State},
    headers::CacheControl,
    middleware,
    response::IntoResponse,
    routing::get,
    Router, TypedHeader,
};
use tokio::sync::RwLock;

use crate::{
    helper::{self, PhixivError},
    state::{authorized_middleware, PhixivState},
};

async fn proxy_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, PhixivError> {
    let state = state.read().await;

    let url = format!("https://i.pximg.net/{path}");

    let mut headers = helper::headers();
    headers.append("Referer", "https://www.pixiv.net/".parse()?);
    headers.append(
        "Authorization",
        format!("Bearer {}", state.auth.access_token).parse()?,
    );

    let response = state.client.get(&url).headers(headers).send().await?;

    Ok((
        TypedHeader(
            CacheControl::new()
                .with_max_age(Duration::from_secs(60 * 60 * 24))
                .with_public(),
        ),
        StreamBody::new(response.bytes_stream()),
    ))
}

pub fn proxy_router(state: Arc<RwLock<PhixivState>>) -> Router<Arc<RwLock<PhixivState>>> {
    Router::new()
        .route("/*path", get(proxy_handler))
        .layer(middleware::from_fn_with_state(state, authorized_middleware))
}

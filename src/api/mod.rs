mod info;

use std::sync::Arc;

use axum::{Router, routing::get, middleware};
use tokio::sync::RwLock;

use crate::state::{PhixivState, authorized_middleware};

use self::info::artwork_info_handler;

pub fn api_router(state: Arc<RwLock<PhixivState>>) -> Router<Arc<RwLock<PhixivState>>> {
    Router::new()
        .route("/info", get(artwork_info_handler))
        .layer(middleware::from_fn_with_state(state.clone(), authorized_middleware))
}
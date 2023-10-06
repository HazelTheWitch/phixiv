use std::time::Instant;

use reqwest::Client;

use crate::auth::PixivAuth;

#[derive(Clone)]
pub struct PhixivState {
    auth: PixivAuth,
    expires_after: Instant,
    client: Client,
}
use std::sync::Arc;

const CLIENT_ID: &str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
const CLIENT_SECRET: &str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";

#[derive(Clone)]
pub struct PixivAuth {
    pub access_token: Arc<String>,
    pub refresh_token: Arc<String>,
}
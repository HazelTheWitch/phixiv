use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderValue, StatusCode};

pub fn headers() -> HeaderMap<HeaderValue> {
    let mut headers = HeaderMap::with_capacity(5);

    headers.append("App-Os", "iOS".parse().unwrap());
    headers.append("App-Os-Version", "14.6".parse().unwrap());
    headers.append("User-Agent", "PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)".parse().unwrap());

    headers
}

pub struct PhixivError(anyhow::Error);

impl IntoResponse for PhixivError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

impl<E> From<E> for PhixivError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

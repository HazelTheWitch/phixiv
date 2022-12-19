use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    run(service_fn(proxy_handler)).await
}

async fn pass_response(response: reqwest::Response) -> Result<Response<Body>, Error> {
    let content_type = response.headers().get("Content-Type");

    Ok({
        let mut builder = Response::builder().status(response.status());

        if let Some(content_type) = content_type {
            builder = builder.header("Content-Type", content_type);
        }

        builder
            .body({
                let bytes: Vec<u8> = response.bytes().await?.into_iter().collect();
                bytes.into()
            })
            .map_err(Box::new)?
    })
}

fn error_response(error: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(error.into())
        .map_err(Box::new)?)
}

async fn proxy_handler(request: Request) -> Result<Response<Body>, Error> {
    let Some(url) = request.query_string_parameters().iter().find_map(|(name, value)| {
        if name == "url" {
            Some(value.to_owned())
        } else {
            None
        }
    }) else {
        return error_response("please provide a url");
    };

    let Ok(url_object) = url::Url::parse(&url) else {
        return error_response(&format!("could not parse url: {}", url));
    };

    if url_object.host_str() != Some("i.pximg.net") {
        return error_response(&format!(
            "can not proxy to host: {:?}",
            url_object.host_str()
        ));
    }

    let client = reqwest::Client::new();
    let Ok(image_response) = client
        .get(&url)
        .header("Referer", "https://www.pixiv.net/")
        .send()
        .await else {
            return error_response("could not access url");
    };

    pass_response(image_response).await
}

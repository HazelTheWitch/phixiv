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
    Ok({
        let mut builder = Response::builder().status(response.status());

        if let Some(content_type) = response.headers().get("Content-Type") {
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

async fn proxy_handler(request: Request) -> Result<Response<Body>, Error> {
    let pixiv_path = request.raw_http_path();
    let pximg_url = format!("https://i.pximg.net{}", &pixiv_path);

    let client = reqwest::Client::new();
    let image_response = client
        .get(&pximg_url)
        .header("Referer", "https://www.pixiv.net/")
        .send()
        .await?;

    pass_response(image_response).await
}

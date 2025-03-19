use std::error::Error;
use std::path::PathBuf;
use warp::{
    self, Filter, path::FullPath, http::{Method, header::{HeaderMap, HeaderValue}}, hyper::body::Bytes
};
use warp_reverse_proxy::{reverse_proxy_filter, extract_request_data_filter, proxy_to_and_forward_response};

#[derive(Clone)]
struct Config {
    cert_path: PathBuf,
    key_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = load_config()?;

    let base_proxy = warp::path::end()
        .and(reverse_proxy_filter("".to_string(), "http://localhost:9091/".to_string()))
        .boxed();

    let transmission_proxy = warp::path("transmission")
        .and(reverse_proxy_filter(
            "".to_string(),
            "http://localhost:9091/".to_string(),
        ))
        .boxed();
    let other_paths_proxy = extract_request_data_filter()
    .and(warp::header::headers_cloned())
    .and(warp::any().map(|| "http://localhost:80/".to_string()))
    .and(warp::any().map(|| "".to_string()))
    .and_then(
        |uri: FullPath,
         params: Option<String>,
         method: Method,
         mut headers: HeaderMap,
         body: Bytes,
         original_headers: HeaderMap,
         proxy_address: String,
         base_path: String| async move {
            if let Some(host) = original_headers.get("Host") {
                headers.insert("Host", host.clone());
            } else {
                headers.insert("Host", HeaderValue::from_static("example.com"));
            }

            // Forward the request with modified headers
            proxy_to_and_forward_response(proxy_address, base_path, uri, params, method, headers, body).await
        },
    )
    .boxed();

    // Combine the routes with priority
    let routes = base_proxy
        .or(transmission_proxy)
        .or(other_paths_proxy);

    let server = warp::serve(routes)
        .tls()
        .cert_path(&config.cert_path)
        .key_path(&config.key_path);

    server.run(([0, 0, 0, 0], 443)).await;

    Ok(())
}
fn load_config() -> Result<Config, Box<dyn Error>> {
    let cert_path = PathBuf::from("cert.pem");
    let key_path = PathBuf::from("key.pem");
    if !cert_path.exists() {
        return Err(format!("Certificate file not found: {}", cert_path.display()).into());
    }
    if !key_path.exists() {
        return Err(format!("Key file not found: {}", key_path.display()).into());
    }

    Ok(Config {
        cert_path,
        key_path,
    })
}

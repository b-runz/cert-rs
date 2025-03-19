use std::error::Error;
use std::path::PathBuf;
use warp::{self, Filter, Rejection, Reply};
use warp_reverse_proxy::reverse_proxy_filter;

#[derive(Clone)]
struct Config {
    cert_path: PathBuf,
    key_path: PathBuf,
}

// Root path filter (/)

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load TLS configuration
    let config = load_config()?;
    // Filter for base path (localhost:9091)
    let base_proxy = warp::path::end()
    .and(reverse_proxy_filter("".to_string(), "http://localhost:9091/".to_string()))
    .boxed(); // Box the filter to unify types

    // Filter for all other paths (localhost:80)
    let other_paths_proxy = reverse_proxy_filter("".to_string(), "http://localhost:80/".to_string())
        .boxed(); // Box the filter to unify types

    // Combine the routes
    let routes = base_proxy.or(other_paths_proxy);
    // Configure and run TLS server
    let server = warp::serve(routes)
        .tls()
        .cert_path(&config.cert_path)
        .key_path(&config.key_path);

    server.run(([0, 0, 0, 0], 443)).await;

    Ok(())
}

// Function to load config at startup
fn load_config() -> Result<Config, Box<dyn Error>> {
    let cert_path = PathBuf::from("cert.pem");
    let key_path = PathBuf::from("key.pem");

    // Verify files exist
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

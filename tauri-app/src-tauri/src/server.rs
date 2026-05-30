use axum::{
    routing::{get, post},
    Router,
    response::IntoResponse,
    http::{StatusCode, Uri, HeaderMap, header, Method},
    Extension,
};
use tower_http::cors::{CorsLayer, Any};
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use tauri::AppHandle;
use axum_server::tls_rustls::RustlsConfig;

use crate::api::{health_handler, get_settings_handler, save_settings_handler, prompt_handler, trust_cert_handler};
use crate::tls::get_or_create_tls_certs;
use crate::settings::load_settings;

#[derive(RustEmbed)]
#[folder = "../dist/"]
struct Assets;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }
    
    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, mime.as_ref().parse().unwrap());
            // Add security headers
            headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
            (StatusCode::OK, headers, content.data.into_owned()).into_response()
        }
        None => {
            // Fallback for SPA or subfolders
            if !path.contains('.') {
                if let Some(content) = Assets::get("index.html") {
                    let mut headers = HeaderMap::new();
                    headers.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());
                    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
                    return (StatusCode::OK, headers, content.data.into_owned()).into_response();
                }
            }
            (StatusCode::NOT_FOUND, "Not Found").into_response()
        }
    }
}

pub async fn start_server(app_handle: AppHandle) -> Result<(), String> {
    // 1. Get or create self-signed certs
    let tls_config = get_or_create_tls_certs(&app_handle)?;
    
    // 2. Load certs into Axum rustls config
    let cert_path = tls_config.cert_path;
    let key_path = tls_config.key_path;
    
    let rustls_config = RustlsConfig::from_pem_file(&cert_path, &key_path)
        .await
        .map_err(|e| format!("Failed to load TLS certificates: {}", e))?;

    // 3. Get server port from settings
    let settings = load_settings(&app_handle);
    let addr = SocketAddr::from(([127, 0, 0, 1], settings.port));

    // 4. Configure CORS to allow the Office Add-In to communicate with the local API securely
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // 5. Setup router
    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/settings", get(get_settings_handler).post(save_settings_handler))
        .route("/api/prompt", post(prompt_handler))
        .route("/api/trust-cert", post(trust_cert_handler))
        .fallback(static_handler)
        .layer(cors)
        .layer(Extension(app_handle));

    println!("Starting HTTPS server at https://{}", addr);

    // 6. Run server
    tokio::spawn(async move {
        if let Err(e) = axum_server::bind_rustls(addr, rustls_config)
            .serve(app.into_make_service())
            .await
        {
            eprintln!("HTTP server error: {}", e);
        }
    });

    Ok(())
}

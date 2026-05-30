use axum::{
    Extension,
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use crate::settings::{AppSettings, load_settings, save_settings};
use crate::ai_client::{PromptRequest, execute_prompt};
use crate::tls::trust_cert_in_os;
use tauri::AppHandle;

pub async fn health_handler() -> impl IntoResponse {
    Json(json!({ "status": "ok", "message": "DOCX AI Helper local server is running" }))
}

pub async fn get_settings_handler(
    Extension(app_handle): Extension<AppHandle>,
) -> impl IntoResponse {
    let settings = load_settings(&app_handle);
    // Don't return the full keys to client, mask them for security
    let mut safe_settings = settings.clone();
    if !safe_settings.openai_api_key.is_empty() {
        safe_settings.openai_api_key = "********".to_string();
    }
    if !safe_settings.gemini_api_key.is_empty() {
        safe_settings.gemini_api_key = "********".to_string();
    }
    Json(safe_settings)
}

pub async fn save_settings_handler(
    Extension(app_handle): Extension<AppHandle>,
    Json(new_settings): Json<AppSettings>,
) -> impl IntoResponse {
    // Preserve existing keys if they are masked
    let current_settings = load_settings(&app_handle);
    
    let mut updated_settings = new_settings.clone();
    if updated_settings.openai_api_key == "********" {
        updated_settings.openai_api_key = current_settings.openai_api_key;
    }
    if updated_settings.gemini_api_key == "********" {
        updated_settings.gemini_api_key = current_settings.gemini_api_key;
    }

    match save_settings(&app_handle, &updated_settings) {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "success", "message": "Settings saved successfully" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e }))).into_response(),
    }
}

pub async fn prompt_handler(
    Extension(app_handle): Extension<AppHandle>,
    Json(req): Json<PromptRequest>,
) -> impl IntoResponse {
    let settings = load_settings(&app_handle);
    match execute_prompt(&settings, &req).await {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e }))).into_response(),
    }
}

pub async fn trust_cert_handler(
    Extension(app_handle): Extension<AppHandle>,
) -> impl IntoResponse {
    match trust_cert_in_os(&app_handle) {
        Ok(msg) => (StatusCode::OK, Json(json!({ "status": "success", "message": msg }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "status": "error", "message": e }))).into_response(),
    }
}

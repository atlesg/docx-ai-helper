mod settings;
mod tls;
mod ai_client;
mod api;
mod server;

use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::TrayIconBuilder,
    Manager,
    WebviewUrl,
    WebviewWindowBuilder,
};
use settings::{AppSettings, load_settings, save_settings};
use tls::trust_cert_in_os;
use server::start_server;

// Tauri Command: Load App Settings
#[tauri::command]
fn get_app_settings(app_handle: tauri::AppHandle) -> Result<AppSettings, String> {
    Ok(load_settings(&app_handle))
}

// Tauri Command: Save App Settings
#[tauri::command]
fn save_app_settings(app_handle: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    save_settings(&app_handle, &settings)
}

// Tauri Command: Trust certificate
#[tauri::command]
async fn run_trust_cert(app_handle: tauri::AppHandle) -> Result<String, String> {
    trust_cert_in_os(&app_handle)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            get_app_settings,
            save_app_settings,
            run_trust_cert
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            // 1. Generate TLS certs and start the embedded HTTPS server in background
            if let Err(e) = tauri::async_runtime::block_on(start_server(app_handle.clone())) {
                eprintln!("Failed to start HTTPS server: {}", e);
            }

            // 2. Create Tray Menu Items
            let show_i = MenuItem::with_id(app, "show", "Open Settings Panel", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            // 3. Build Tray Menu
            let menu = MenuBuilder::new(app)
                .item(&show_i)
                .separator()
                .item(&quit_i)
                .build()?;

            // 4. Build Tray Icon
            let icon = app.default_window_icon().cloned().ok_or("No default window icon found")?;
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            } else {
                                // Recreate window if closed
                                let _ = WebviewWindowBuilder::new(
                                    app,
                                    "main",
                                    WebviewUrl::default(),
                                )
                                .title("DOCX AI Helper - Settings")
                                .inner_size(800.0, 600.0)
                                .resizable(true)
                                .build();
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // 5. Hide main window on start by default
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

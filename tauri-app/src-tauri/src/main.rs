// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Explicitly install default crypto provider for rustls 0.23+
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    docx_ai_helper_lib::run()
}

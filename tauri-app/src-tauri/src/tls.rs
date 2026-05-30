use rcgen::{generate_simple_self_signed, CertifiedKey};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

pub struct TlsConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

fn get_tls_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    
    path.push("certs");
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

pub fn get_or_create_tls_certs(app_handle: &tauri::AppHandle) -> Result<TlsConfig, String> {
    let tls_dir = get_tls_dir(app_handle)?;
    let cert_path = tls_dir.join("cert.pem");
    let key_path = tls_dir.join("key.pem");

    if cert_path.exists() && key_path.exists() {
        return Ok(TlsConfig { cert_path, key_path });
    }

    println!("Generating new self-signed TLS certificates for localhost...");
    
    let subject_alt_names = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
    ];

    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names)
        .map_err(|e| format!("Failed to generate simple self signed cert: {}", e))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    // Write to disk
    fs::write(&cert_path, cert_pem).map_err(|e| e.to_string())?;
    fs::write(&key_path, key_pem).map_err(|e| e.to_string())?;

    Ok(TlsConfig { cert_path, key_path })
}

/// Trusts the self-signed certificate on Windows or Mac.
/// Performs OS-level commands to insert it into the system trusted store.
pub fn trust_cert_in_os(app_handle: &tauri::AppHandle) -> Result<String, String> {
    let tls_config = get_or_create_tls_certs(app_handle)?;
    let cert_path_str = tls_config.cert_path.to_str().ok_or("Invalid path string")?;

    #[cfg(target_os = "windows")]
    {
        // PowerShell command to trust certificate:
        // Import-Certificate -FilePath "path\to\cert.pem" -CertStoreLocation Cert:\LocalMachine\Root
        // Or using certutil (faster, no admin prompt if run with right privileges, but certutil normally requires admin prompt)
        // Let's use certutil.exe
        let output = Command::new("certutil")
            .args(&["-addstore", "-f", "Root", cert_path_str])
            .output()
            .map_err(|e| format!("Failed to execute certutil: {}", e))?;

        if output.status.success() {
            Ok("Successfully trusted the certificate on Windows.".to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("certutil failed: {}. Please run the app as Administrator or trust the certificate manually.", error_msg))
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS trust command:
        // sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain cert.pem
        // Since it requires sudo, we notify the user.
        let output = Command::new("security")
            .args(&[
                "add-trusted-cert",
                "-d",
                "-r",
                "trustRoot",
                "-k",
                "/Library/Keychains/System.keychain",
                cert_path_str,
            ])
            .output()
            .map_err(|e| format!("Failed to execute security command: {}", e))?;

        if output.status.success() {
            Ok("Successfully trusted the certificate on macOS.".to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("security command failed: {}. Please trust the certificate manually in Keychain Access.", error_msg))
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err("Unsupported OS for auto-trusting. Please trust cert.pem manually.".to_string())
    }
}

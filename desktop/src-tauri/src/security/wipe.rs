use tauri::Manager;

pub fn clear_browsing_artifacts(app: &tauri::AppHandle) {
    for (_label, wv) in app.webviews() {
        let _ = wv.clear_all_browsing_data();
    }

    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let cache_path = std::path::Path::new(&local_app_data).join("EBWebView");
        if cache_path.exists() {
            let _ = std::fs::remove_dir_all(&cache_path);
            log::info!("[TUNNEL] EBWebView HTTP cache removed: {:?}", cache_path);
        }
        let chrome_path = std::path::Path::new(&local_app_data).join("com.google.chrome");
        if chrome_path.exists() {
            let _ = std::fs::remove_dir_all(&chrome_path);
            log::info!("[TUNNEL] com.google.chrome profile removed: {:?}", chrome_path);
        }
    }
    let shell_cache = std::env::temp_dir().join("ShellCache");
    if shell_cache.exists() {
        let _ = std::fs::remove_dir_all(&shell_cache);
        log::info!("[TUNNEL] ShellCache data dir removed: {:?}", shell_cache);
    }
    log::info!("[TUNNEL] All browsing data cleared before switching to Tunnel mode");
}

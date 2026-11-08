use tauri::Manager;

pub fn clean_directory(path: &std::path::Path) {
    if !path.exists() {
        return;
    }
    if let Err(e) = std::fs::remove_dir_all(path) {
        log::warn!("[CLEANUP] Could not remove directory {:?}: {}, attempting entry removal", path, e);
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let _ = std::fs::remove_dir_all(&p);
                } else {
                    let _ = std::fs::remove_file(&p);
                }
            }
        }
    } else {
        log::info!("[CLEANUP] Directory removed: {:?}", path);
    }
}

pub fn clear_browsing_artifacts(app: &tauri::AppHandle) {
    for (_label, wv) in app.webviews() {
        let _ = wv.clear_all_browsing_data();
    }

    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let cache_path = std::path::Path::new(&local_app_data).join("EBWebView");
        clean_directory(&cache_path);
    }

    let shell_cache = std::env::temp_dir().join("ShellCache");
    clean_directory(&shell_cache);

    log::info!("[TUNNEL] All browsing data cleared before switching to Tunnel mode");
}

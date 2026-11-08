use crate::browser::TabManager;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};

#[cfg(windows)]
extern "system" {
    fn SetWindowPos(
        hWnd: isize,
        hWndInsertAfter: isize,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;
}

static WAS_MAXIMIZED: AtomicBool = AtomicBool::new(false);
pub static IS_FULLSCREEN: AtomicBool = AtomicBool::new(false);

pub fn minimize(app: &tauri::AppHandle) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        let _ = win.minimize();
    }
}

pub fn maximize(app: &tauri::AppHandle) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        if let Ok(is_maximized) = win.is_maximized() {
            if is_maximized {
                let _ = win.unmaximize();
            } else {
                let _ = win.maximize();
            }
        }
    }
}

pub fn close(app: &tauri::AppHandle) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        let _ = win.close();
    }
}

pub fn set_title(app: &tauri::AppHandle, title: &str) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        let _ = win.set_title(title);
    }
}

pub fn start_dragging(app: &tauri::AppHandle) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        let _ = win.start_dragging();
    }
}

pub fn toggle_fullscreen(app: &tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        let is_fs = win.is_fullscreen().map_err(|e| e.to_string())?;
        let next_state = !is_fs;
        IS_FULLSCREEN.store(next_state, Ordering::SeqCst);

        if next_state {
            let is_max = win.is_maximized().unwrap_or(false);
            WAS_MAXIMIZED.store(is_max, Ordering::SeqCst);
            if is_max {
                let _ = win.unmaximize();
            }
            win.set_fullscreen(true).map_err(|e| e.to_string())?;
            #[cfg(windows)]
            {
                if let Ok(hwnd) = win.hwnd() {
                    if let Ok(Some(mon)) = win.current_monitor() {
                        let pos = mon.position();
                        let size = mon.size();
                        unsafe {
                            SetWindowPos(
                                hwnd.0 as isize,
                                0,
                                pos.x,
                                pos.y,
                                size.width as i32,
                                size.height as i32,
                                0x0004 | 0x0020 | 0x0040,
                            );
                        }
                    }
                }
            }
        } else {
            win.set_fullscreen(false).map_err(|e| e.to_string())?;
            if WAS_MAXIMIZED.load(Ordering::SeqCst) {
                let _ = win.maximize();
            }
        }

        let _ = win.emit("fullscreen-changed", next_state);

        let tab_mgr = app.state::<TabManager>();
        if next_state {
            tab_mgr.set_top_bar_height(0.0);
        } else {
            tab_mgr.set_top_bar_height(84.0);
        }
        tab_mgr.sync_main_window_geometry(&win);

        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            if let Some(w) = app_clone.get_window("main") {
                let tm = app_clone.state::<TabManager>();
                tm.sync_main_window_geometry(&w);
            }
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            if let Some(w) = app_clone.get_window("main") {
                let tm = app_clone.state::<TabManager>();
                tm.sync_main_window_geometry(&w);
            }
        });

        Ok(next_state)
    } else {
        Err("Main window not found".to_string())
    }
}

pub fn set_fullscreen(app: &tauri::AppHandle, fullscreen: bool) -> Result<bool, String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        let is_fs = win.is_fullscreen().map_err(|e| e.to_string())?;
        IS_FULLSCREEN.store(fullscreen, Ordering::SeqCst);
        if is_fs != fullscreen {
            if fullscreen {
                let is_max = win.is_maximized().unwrap_or(false);
                WAS_MAXIMIZED.store(is_max, Ordering::SeqCst);
                if is_max {
                    let _ = win.unmaximize();
                }
                win.set_fullscreen(true).map_err(|e| e.to_string())?;
                #[cfg(windows)]
                {
                    if let Ok(hwnd) = win.hwnd() {
                        if let Ok(Some(mon)) = win.current_monitor() {
                            let pos = mon.position();
                            let size = mon.size();
                            unsafe {
                                SetWindowPos(
                                    hwnd.0 as isize,
                                    0,
                                    pos.x,
                                    pos.y,
                                    size.width as i32,
                                    size.height as i32,
                                    0x0004 | 0x0020 | 0x0040,
                                );
                            }
                        }
                    }
                }
            } else {
                win.set_fullscreen(false).map_err(|e| e.to_string())?;
                if WAS_MAXIMIZED.load(Ordering::SeqCst) {
                    let _ = win.maximize();
                }
            }

            let _ = win.emit("fullscreen-changed", fullscreen);

            let tab_mgr = app.state::<TabManager>();
            if fullscreen {
                tab_mgr.set_top_bar_height(0.0);
            } else {
                tab_mgr.set_top_bar_height(84.0);
            }
            tab_mgr.sync_main_window_geometry(&win);

            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                if let Some(w) = app_clone.get_window("main") {
                    let tm = app_clone.state::<TabManager>();
                    tm.sync_main_window_geometry(&w);
                }
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                if let Some(w) = app_clone.get_window("main") {
                    let tm = app_clone.state::<TabManager>();
                    tm.sync_main_window_geometry(&w);
                }
            });
        }
        Ok(fullscreen)
    } else {
        Err("Main window not found".to_string())
    }
}

pub fn is_fullscreen(app: &tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        win.is_fullscreen().map_err(|e| e.to_string())
    } else {
        Err("Main window not found".to_string())
    }
}

pub fn show_fullscreen_header(app: &tauri::AppHandle, show: bool) -> Result<(), String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        if let Ok(true) = win.is_fullscreen() {
            let _ = win.emit("show-fullscreen-header", show);
        }
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

pub fn exit_fullscreen_if_active(app: &tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        if let Ok(true) = win.is_fullscreen() {
            IS_FULLSCREEN.store(false, Ordering::SeqCst);
            win.set_fullscreen(false).map_err(|e| e.to_string())?;
            if WAS_MAXIMIZED.load(Ordering::SeqCst) {
                let _ = win.maximize();
            }
            let _ = win.emit("fullscreen-changed", false);

            let tab_mgr = app.state::<TabManager>();
            tab_mgr.set_top_bar_height(84.0);
            tab_mgr.sync_main_window_geometry(&win);

            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                if let Some(w) = app_clone.get_window("main") {
                    let tm = app_clone.state::<TabManager>();
                    tm.sync_main_window_geometry(&w);
                }
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                if let Some(w) = app_clone.get_window("main") {
                    let tm = app_clone.state::<TabManager>();
                    tm.sync_main_window_geometry(&w);
                }
            });

            return Ok(true);
        }
        Ok(false)
    } else {
        Err("Main window not found".to_string())
    }
}

pub fn open_path(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Ok(())
    }
}

pub fn show_in_folder(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Ok(())
    }
}

pub fn get_downloads_dir() -> String {
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let downloads = std::path::Path::new(&user_profile).join("Downloads");
        if downloads.exists() {
            return downloads.to_string_lossy().to_string();
        }
    }
    ".".to_string()
}


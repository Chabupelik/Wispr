// frontend/src-tauri/src/commands.rs
use crate::browser::TabManager;
use crate::security::device::get_hardware_fingerprint;
use crate::tunnel::{ProxyConnectionInfo, TunnelManager};
use tauri::{Emitter, Manager, State};

/// Возвращает уникальный аппаратный отпечаток устройства
#[tauri::command]
pub fn get_device_id() -> String {
    get_hardware_fingerprint()
}

/// Запускает Wisp-туннель к VPS и поднимает локальный прокси.
/// Перед переключением в Tunnel-режим чистит весь кэш WebView2,
/// чтобы исключить утечку данных из FakeChromeView (кэшированные страницы с реальным IP).
#[tauri::command]
pub async fn start_tunnel(
    app: tauri::AppHandle,
    wisp_url: String,
    token: String,
    manager: State<'_, TunnelManager>,
) -> Result<ProxyConnectionInfo, String> {
    // Чистим ВСЕ данные браузера (cookies, cache, localStorage) ПЕРЕД подключением к WISP.
    // Это критически важно: без этого WebView2 может отдать кэшированную страницу
    // с реальным IP из сессии FakeChromeView.
    for (_label, wv) in app.webviews() {
        let _ = wv.clear_all_browsing_data();
    }

    // Принудительно удаляем HTTP disk-cache WebView2 на диске.
    // clear_all_browsing_data() работает только для in-memory кэша активных webview,
    // но HTTP-кэш на диске может пережить смену сессии.
    // Без этого 2ip.ru и подобные сервисы отдают закэшированный ответ с реальным IP.
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        // Старые пути от прежних версий (до перенаправления папки данных в %TEMP%)
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
    // Текущая папка данных WebView2 — %TEMP%\ShellCache.
    // Пока WebView2 запущен, часть файлов заблокирована, поэтому best-effort:
    // незаблокированные файлы кэша удалятся, остальные чистит clear_all_browsing_data.
    let shell_cache = std::env::temp_dir().join("ShellCache");
    if shell_cache.exists() {
        let _ = std::fs::remove_dir_all(&shell_cache);
        log::info!("[TUNNEL] ShellCache data dir removed: {:?}", shell_cache);
    }
    log::info!("[TUNNEL] All browsing data cleared before switching to Tunnel mode");

    manager.start(&wisp_url, &token).await
}

/// Останавливает активный прокси-туннель
#[tauri::command]
pub fn stop_tunnel(manager: State<'_, TunnelManager>) -> Result<(), String> {
    manager.stop();
    Ok(())
}






// ----------------------------------------------------
// НАЦИОНАЛЬНЫЕ КОМАНДЫ ДЛЯ CHROMIUM-ВКЛАДОК (MULTI-WEBVIEW)
// ----------------------------------------------------

#[tauri::command]
pub async fn native_navigate_tab(
    app: tauri::AppHandle,
    tab_id: String,
    url: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    let tm = tab_manager.inner().clone();
    let app_clone = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let win = app_clone.get_window("main")
            .or_else(|| app_clone.webview_windows().into_values().next().map(|w| w.as_ref().window().clone()));
        let res = match win {
            Some(w) => tm.create_or_navigate_tab(&w, &tab_id, &url),
            None => Err("Main window not found".to_string()),
        };
        let _ = tx.send(res);
    }).map_err(|e| e.to_string())?;

    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn native_switch_tab(
    app: tauri::AppHandle,
    tab_id: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    let tm = tab_manager.inner().clone();
    let app_clone = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let win = app_clone.get_window("main")
            .or_else(|| app_clone.webview_windows().into_values().next().map(|w| w.as_ref().window().clone()));
        let res = match win {
            Some(w) => tm.switch_to_tab(&w, &tab_id),
            None => Err("Main window not found".to_string()),
        };
        let _ = tx.send(res);
    }).map_err(|e| e.to_string())?;

    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn native_close_tab(
    app: tauri::AppHandle,
    tab_id: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    let tm = tab_manager.inner().clone();
    let app_clone = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let win = app_clone.get_window("main")
            .or_else(|| app_clone.webview_windows().into_values().next().map(|w| w.as_ref().window().clone()));
        let res = match win {
            Some(w) => tm.close_tab(&w, &tab_id),
            None => Err("Main window not found".to_string()),
        };
        let _ = tx.send(res);
    }).map_err(|e| e.to_string())?;

    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn native_go_back(
    app: tauri::AppHandle,
    tab_id: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    let tm = tab_manager.inner().clone();
    let app_clone = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let win = app_clone.get_window("main")
            .or_else(|| app_clone.webview_windows().into_values().next().map(|w| w.as_ref().window().clone()));
        let res = match win {
            Some(w) => tm.go_back(&w, &tab_id),
            None => Err("Main window not found".to_string()),
        };
        let _ = tx.send(res);
    }).map_err(|e| e.to_string())?;

    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn native_go_forward(
    app: tauri::AppHandle,
    tab_id: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    let tm = tab_manager.inner().clone();
    let app_clone = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let win = app_clone.get_window("main")
            .or_else(|| app_clone.webview_windows().into_values().next().map(|w| w.as_ref().window().clone()));
        let res = match win {
            Some(w) => tm.go_forward(&w, &tab_id),
            None => Err("Main window not found".to_string()),
        };
        let _ = tx.send(res);
    }).map_err(|e| e.to_string())?;

    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn native_reload(
    app: tauri::AppHandle,
    tab_id: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    let tm = tab_manager.inner().clone();
    let app_clone = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let win = app_clone.get_window("main")
            .or_else(|| app_clone.webview_windows().into_values().next().map(|w| w.as_ref().window().clone()));
        let res = match win {
            Some(w) => tm.reload(&w, &tab_id),
            None => Err("Main window not found".to_string()),
        };
        let _ = tx.send(res);
    }).map_err(|e| e.to_string())?;

    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn native_close_all_tabs(
    app: tauri::AppHandle,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    let tm = tab_manager.inner().clone();
    let app_clone = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let win = app_clone.get_window("main")
            .or_else(|| app_clone.webview_windows().into_values().next().map(|w| w.as_ref().window().clone()));
        let res = match win {
            Some(w) => tm.close_all_tabs(&w),
            None => Err("Main window not found".to_string()),
        };
        let _ = tx.send(res);
    }).map_err(|e| e.to_string())?;

    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn native_update_top_bar_height(
    app: tauri::AppHandle,
    height: f64,
    tab_manager: State<'_, TabManager>,
) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        tab_manager.update_top_bar_height(&win, height);
    }
}

#[tauri::command]
pub fn native_minimize_window(app: tauri::AppHandle) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        let _ = win.minimize();
    }
}

#[tauri::command]
pub fn native_maximize_window(app: tauri::AppHandle) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        if let Ok(is_maximized) = win.is_maximized() {
            if is_maximized {
                let _ = win.unmaximize();
            } else {
                let _ = win.maximize();
            }
        }
    }
}

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

static WAS_MAXIMIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub static IS_FULLSCREEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[tauri::command]
pub fn native_toggle_fullscreen(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        let is_fs = win.is_fullscreen().map_err(|e| e.to_string())?;
        let next_state = !is_fs;
        IS_FULLSCREEN.store(next_state, std::sync::atomic::Ordering::SeqCst);

        if next_state {
            let is_max = win.is_maximized().unwrap_or(false);
            WAS_MAXIMIZED.store(is_max, std::sync::atomic::Ordering::SeqCst);
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
                                0, // HWND_TOP
                                pos.x,
                                pos.y,
                                size.width as i32,
                                size.height as i32,
                                0x0004 | 0x0020 | 0x0040, // SWP_NOZORDER | SWP_FRAMECHANGED | SWP_SHOWWINDOW
                            );
                        }
                    }
                }
            }
        } else {
            win.set_fullscreen(false).map_err(|e| e.to_string())?;
            if WAS_MAXIMIZED.load(std::sync::atomic::Ordering::SeqCst) {
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

#[tauri::command]
pub fn native_set_fullscreen(app: tauri::AppHandle, fullscreen: bool) -> Result<bool, String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        let is_fs = win.is_fullscreen().map_err(|e| e.to_string())?;
        IS_FULLSCREEN.store(fullscreen, std::sync::atomic::Ordering::SeqCst);
        if is_fs != fullscreen {
            if fullscreen {
                let is_max = win.is_maximized().unwrap_or(false);
                WAS_MAXIMIZED.store(is_max, std::sync::atomic::Ordering::SeqCst);
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
                if WAS_MAXIMIZED.load(std::sync::atomic::Ordering::SeqCst) {
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

#[tauri::command]
pub fn native_is_fullscreen(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        win.is_fullscreen().map_err(|e| e.to_string())
    } else {
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
pub fn native_show_fullscreen_header(app: tauri::AppHandle, show: bool) -> Result<(), String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        if let Ok(true) = win.is_fullscreen() {
            let _ = win.emit("show-fullscreen-header", show);
        }
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
pub fn native_exit_fullscreen_if_active(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        if let Ok(true) = win.is_fullscreen() {
            IS_FULLSCREEN.store(false, std::sync::atomic::Ordering::SeqCst);
            win.set_fullscreen(false).map_err(|e| e.to_string())?;
            if WAS_MAXIMIZED.load(std::sync::atomic::Ordering::SeqCst) {
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

/// Самоуничтожение приложения: запускает независимый процесс (PowerShell, detached),
/// который после выхода браузера затирает его .exe нулями и удаляет файл,
/// а также зачищает временную папку данных WebView2 в %TEMP%.
#[tauri::command]
pub fn self_destruct() {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW — без консольного окна (DETACHED_PROCESS ломает запуск PowerShell);
        // NEW_PROCESS_GROUP + BREAKAWAY_FROM_JOB — процесс независим от родителя
        const CHILD_FLAGS: u32 = 0x0800_0000 | 0x0000_0200 | 0x0100_0000;

        let pid = std::process::id();
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().replace('\'', "''"))
            .unwrap_or_default();

        if exe.is_empty() {
            println!("ERROR: current_exe() failed");
        } else {
            // Абсолютный путь к powershell — не зависим от PATH
            let sysroot = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
            let ps_exe = std::path::Path::new(&sysroot)
                .join(r"System32\WindowsPowerShell\v1.0\powershell.exe");

            // Скрипт ожидает завершения процесса и перезаписывает файл нулями
            let script = format!(
                r#"$deadline=(Get-Date).AddSeconds(30)
while((Get-Process -Id {pid} -ErrorAction SilentlyContinue) -and ((Get-Date) -lt $deadline)){{ Start-Sleep -Milliseconds 200 }}
$p='{exe}'
try{{
    $fs=[IO.File]::Open($p,'Open','ReadWrite','None')
    $len=$fs.Length
    $buf=New-Object byte[] (1048576)
    while($fs.Position -lt $len){{
        $n=[Math]::Min(1048576,[int]($len-$fs.Position))
        $fs.Write($buf,0,$n)
    }}
    $fs.Flush($true)
    $fs.Close()
}}catch{{}}
try{{
    Remove-Item -LiteralPath $p -Force
}}catch{{}}
Remove-Item -LiteralPath (Join-Path $env:TEMP 'ShellCache') -Recurse -Force -ErrorAction SilentlyContinue
"#
            );

            match std::process::Command::new(&ps_exe)
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-EncodedCommand",
                    &powershell_encoded(&script),
                ])
                .creation_flags(CHILD_FLAGS)
                .spawn()
            {
                Ok(_) => println!("SPAWNED pid={pid} exe={exe}"),
                Err(e) => println!("SPAWN_ERR: {} (ps={:?})", e, ps_exe),
            }
        }
    }

    std::process::exit(0);
}

/// Кодирует скрипт в base64 (UTF-16LE) для powershell -EncodedCommand
#[cfg(windows)]
fn powershell_encoded(script: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bytes = Vec::with_capacity(script.len() * 2);
    for unit in script.encode_utf16() {
        bytes.push(unit as u8);
        bytes.push((unit >> 8) as u8);
    }
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[n as usize & 63] as char } else { '=' });
    }
    out
}

#[tauri::command]
pub fn native_close_window(app: tauri::AppHandle) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        let _ = win.close();
    }
}

#[tauri::command]
pub fn native_set_window_title(app: tauri::AppHandle, title: String) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        let _ = win.set_title(&title);
    }
}

#[tauri::command]
pub fn native_start_dragging(app: tauri::AppHandle) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        let _ = win.start_dragging();
    }
}

#[tauri::command]
pub fn native_title_changed(app: tauri::AppHandle, tab_id: String, title: String) {
    #[derive(serde::Serialize, Clone)]
    struct TitlePayload {
        tab_id: String,
        title: String,
    }
    use tauri::Emitter;
    let _ = app.emit("webview-title-changed", TitlePayload { tab_id, title });
}

#[tauri::command]
pub fn native_favicon_changed(app: tauri::AppHandle, tab_id: String, favicon: String) {
    #[derive(serde::Serialize, Clone)]
    struct FaviconPayload {
        tab_id: String,
        favicon: String,
    }
    use tauri::Emitter;
    let _ = app.emit("webview-favicon-changed", FaviconPayload { tab_id, favicon });
}

#[tauri::command]
pub fn native_context_menu(_app: tauri::AppHandle, window: String, href: String, src: String, selection: String) {
    println!("Context menu: window={}, href={}, src={}, selection={}", window, href, src, selection);
}


#[tauri::command]
pub fn native_url_changed(app: tauri::AppHandle, tab_id: String, url: String) {
    #[derive(serde::Serialize, Clone)]
    #[allow(non_snake_case)]
    struct UrlPayload {
        tabId: String,
        url: String,
    }
    use tauri::Emitter;
    // Emit webview-loaded so frontend updates the URL and stops loading spinner
    let _ = app.emit("webview-loaded", UrlPayload { tabId: tab_id, url });
}

#[tauri::command]
pub fn native_set_zoom(
    app: tauri::AppHandle,
    tab_id: String,
    zoom_factor: f64,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next().map(|w| w.clone())) {
        tab_manager.set_zoom(&win, &tab_id, zoom_factor)?;
    }
    Ok(())
}


#[tauri::command]
pub fn native_open_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
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

#[tauri::command]
pub fn native_show_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
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

#[tauri::command]
pub fn native_get_downloads_dir() -> String {
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let downloads = std::path::Path::new(&user_profile).join("Downloads");
        if downloads.exists() {
            return downloads.to_string_lossy().to_string();
        }
    }
    ".".to_string()
}

/// Открывает URL в новой вкладке браузера (вызывается из JS при window.open / target=_blank)
#[tauri::command]
pub fn native_open_new_tab(app: tauri::AppHandle, url: String) {
    #[derive(serde::Serialize, Clone)]
    struct NewTabPayload {
        url: String,
    }
    use tauri::Emitter;
    let _ = app.emit("open-new-tab", NewTabPayload { url });
}

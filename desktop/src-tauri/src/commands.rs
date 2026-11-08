use crate::browser::TabManager;
use crate::tunnel::{ProxyConnectionInfo, TunnelManager};
use tauri::{Manager, State};

pub use crate::browser::window::IS_FULLSCREEN;

#[tauri::command]
pub fn get_device_id() -> String {
    crate::security::device::get_hardware_fingerprint()
}

#[tauri::command]
pub async fn start_tunnel(
    app: tauri::AppHandle,
    wisp_url: String,
    token: String,
    manager: State<'_, TunnelManager>,
) -> Result<ProxyConnectionInfo, String> {
    crate::security::wipe::clear_browsing_artifacts(&app);
    manager.start(&wisp_url, &token).await
}

#[tauri::command]
pub fn stop_tunnel(manager: State<'_, TunnelManager>) -> Result<(), String> {
    manager.stop();
    Ok(())
}

async fn run_tab_action<F>(
    app: tauri::AppHandle,
    tab_manager: State<'_, TabManager>,
    action: F,
) -> Result<(), String>
where
    F: FnOnce(&TabManager, &tauri::Window) -> Result<(), String> + Send + 'static,
{
    let tm = tab_manager.inner().clone();
    let app_clone = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let win = app_clone.get_window("main")
            .or_else(|| app_clone.webview_windows().into_values().next().map(|w| w.as_ref().window().clone()));
        let res = match win {
            Some(w) => action(&tm, &w),
            None => Err("Main window not found".to_string()),
        };
        let _ = tx.send(res);
    }).map_err(|e| e.to_string())?;

    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn native_navigate_tab(
    app: tauri::AppHandle,
    tab_id: String,
    url: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    run_tab_action(app, tab_manager, move |tm, win| tm.create_or_navigate_tab(win, &tab_id, &url)).await
}

#[tauri::command]
pub async fn native_switch_tab(
    app: tauri::AppHandle,
    tab_id: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    run_tab_action(app, tab_manager, move |tm, win| tm.switch_to_tab(win, &tab_id)).await
}

#[tauri::command]
pub async fn native_close_tab(
    app: tauri::AppHandle,
    tab_id: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    run_tab_action(app, tab_manager, move |tm, win| tm.close_tab(win, &tab_id)).await
}

#[tauri::command]
pub async fn native_go_back(
    app: tauri::AppHandle,
    tab_id: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    run_tab_action(app, tab_manager, move |tm, win| tm.go_back(win, &tab_id)).await
}

#[tauri::command]
pub async fn native_go_forward(
    app: tauri::AppHandle,
    tab_id: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    run_tab_action(app, tab_manager, move |tm, win| tm.go_forward(win, &tab_id)).await
}

#[tauri::command]
pub async fn native_reload(
    app: tauri::AppHandle,
    tab_id: String,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    run_tab_action(app, tab_manager, move |tm, win| tm.reload(win, &tab_id)).await
}

#[tauri::command]
pub async fn native_close_all_tabs(
    app: tauri::AppHandle,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    run_tab_action(app, tab_manager, move |tm, win| tm.close_all_tabs(win)).await
}

#[tauri::command]
pub fn native_update_top_bar_height(
    app: tauri::AppHandle,
    height: f64,
    tab_manager: State<'_, TabManager>,
) {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        tab_manager.update_top_bar_height(&win, height);
    }
}

#[tauri::command]
pub fn native_minimize_window(app: tauri::AppHandle) {
    crate::browser::window::minimize(&app);
}

#[tauri::command]
pub fn native_maximize_window(app: tauri::AppHandle) {
    crate::browser::window::maximize(&app);
}

#[tauri::command]
pub fn native_toggle_fullscreen(app: tauri::AppHandle) -> Result<bool, String> {
    crate::browser::window::toggle_fullscreen(&app)
}

#[tauri::command]
pub fn native_set_fullscreen(app: tauri::AppHandle, fullscreen: bool) -> Result<bool, String> {
    crate::browser::window::set_fullscreen(&app, fullscreen)
}

#[tauri::command]
pub fn native_is_fullscreen(app: tauri::AppHandle) -> Result<bool, String> {
    crate::browser::window::is_fullscreen(&app)
}

#[tauri::command]
pub fn native_show_fullscreen_header(app: tauri::AppHandle, show: bool) -> Result<(), String> {
    crate::browser::window::show_fullscreen_header(&app, show)
}

#[tauri::command]
pub fn native_exit_fullscreen_if_active(app: tauri::AppHandle) -> Result<bool, String> {
    crate::browser::window::exit_fullscreen_if_active(&app)
}

#[tauri::command]
pub fn native_close_window(app: tauri::AppHandle) {
    crate::browser::window::close(&app);
}

#[tauri::command]
pub fn native_set_window_title(app: tauri::AppHandle, title: String) {
    crate::browser::window::set_title(&app, &title);
}

#[tauri::command]
pub fn native_start_dragging(app: tauri::AppHandle) {
    crate::browser::window::start_dragging(&app);
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
    log::debug!("Context menu: window={}, href={}, src={}, selection={}", window, href, src, selection);
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
    let _ = app.emit("webview-loaded", UrlPayload { tabId: tab_id, url });
}

#[tauri::command]
pub fn native_set_zoom(
    app: tauri::AppHandle,
    tab_id: String,
    zoom_factor: f64,
    tab_manager: State<'_, TabManager>,
) -> Result<(), String> {
    if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
        tab_manager.set_zoom(&win, &tab_id, zoom_factor)?;
    }
    Ok(())
}

#[tauri::command]
pub fn native_open_new_tab(app: tauri::AppHandle, url: String) {
    #[derive(serde::Serialize, Clone)]
    struct NewTabPayload {
        url: String,
    }
    use tauri::Emitter;
    let _ = app.emit("open-new-tab", NewTabPayload { url });
}

#[tauri::command]
pub fn native_open_path(path: String) -> Result<(), String> {
    crate::browser::window::open_path(&path)
}

#[tauri::command]
pub fn native_show_in_folder(path: String) -> Result<(), String> {
    crate::browser::window::show_in_folder(&path)
}

#[tauri::command]
pub fn native_get_downloads_dir() -> String {
    crate::browser::window::get_downloads_dir()
}

#[tauri::command]
pub fn self_destruct() {
    crate::security::self_destruct::execute();
}

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, Manager, WebviewBuilder, WebviewUrl};
use url::Url;

#[derive(Clone, Default)]
pub struct TabManager {
    tabs: Arc<Mutex<HashMap<String, String>>>,
    active_tab_id: Arc<Mutex<Option<String>>>,
    top_bar_height: Arc<Mutex<f64>>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(Mutex::new(HashMap::new())),
            active_tab_id: Arc::new(Mutex::new(None)),
            top_bar_height: Arc::new(Mutex::new(84.0)),
        }
    }

    pub fn set_top_bar_height(&self, height: f64) {
        let mut h = self.top_bar_height.lock();
        *h = height;
    }

    pub fn get_target_geometry(&self, window: &tauri::Window) -> (f64, f64, f64) {
        let is_fs = window.is_fullscreen().unwrap_or(false);
        let scale_factor = window.scale_factor().unwrap_or(1.0);

        let (win_w, win_h) = if is_fs {
            if let Ok(Some(monitor)) = window.current_monitor() {
                let mon_logical = monitor.size().to_logical::<f64>(scale_factor);
                (mon_logical.width, mon_logical.height)
            } else if let Ok(window_size) = window.inner_size() {
                let logical = window_size.to_logical::<f64>(scale_factor);
                (logical.width, logical.height)
            } else {
                (800.0, 600.0)
            }
        } else if let Ok(window_size) = window.inner_size() {
            let logical = window_size.to_logical::<f64>(scale_factor);
            (logical.width, logical.height)
        } else {
            (800.0, 600.0)
        };

        let top_bar_height = if is_fs && *self.top_bar_height.lock() == 0.0 {
            0.0
        } else {
            *self.top_bar_height.lock()
        };

        let content_height = (win_h - top_bar_height).max(100.0);
        (win_w, top_bar_height, content_height)
    }

    pub fn update_top_bar_height(&self, window: &tauri::Window, height: f64) {
        self.set_top_bar_height(height);
        self.sync_main_window_geometry(window);
    }

    pub fn create_or_navigate_tab(&self, window: &tauri::Window, tab_id: &str, target_url: &str) -> Result<(), String> {
        let label = format!("tab-wv-{}", tab_id);
        log::debug!("[TAB_MANAGER] create_or_navigate_tab: tab_id={}, target_url={}, label={}", tab_id, target_url, label);
        
        let is_blank = target_url.is_empty() || target_url == "about:blank";
        if is_blank {
            log::debug!("[TAB_MANAGER] Blank URL -> hiding all webviews");
            self.hide_all_webviews(window);
            let mut active = self.active_tab_id.lock();
            *active = Some(tab_id.to_string());
            return Ok(());
        }

        let parsed_url = if let Ok(u) = Url::parse(target_url) {
            WebviewUrl::External(u)
        } else {
            let search_url = format!("https://duckduckgo.com/?q={}", urlencoding::encode(target_url));
            WebviewUrl::External(Url::parse(&search_url).map_err(|e| e.to_string())?)
        };

        let app_handle = window.app_handle();

        if let Some(existing_wv) = app_handle.get_webview(&label) {
            log::debug!("[TAB_MANAGER] Found existing webview for {}. Navigating to {:?}", label, parsed_url);
            if let WebviewUrl::External(u) = &parsed_url {
                if let Err(e) = existing_wv.navigate(u.clone()) {
                    log::error!("[TAB_MANAGER] ERROR navigating webview: {:?}", e);
                }
            }
            self.switch_to_tab(window, tab_id)?;
            return Ok(());
        }

        let window_size = window
            .inner_size()
            .map_err(|e| format!("Failed to get window size: {}", e))?;
        let scale_factor = window
            .scale_factor()
            .map_err(|e| format!("Failed to get scale factor: {}", e))?;
        let logical_size = window_size.to_logical::<f64>(scale_factor);
        let top_bar_height = *self.top_bar_height.lock();
        let content_height = (logical_size.height - top_bar_height).max(100.0);
        let t_id = tab_id.to_string();
        let t_id_ipc = t_id.clone();
        
        let base_script = r#"
                if (window !== window.top) return;

                if (navigator.userAgentData) {
                  Object.defineProperty(navigator, 'userAgentData', {
                    get: () => ({
                      brands: [
                        { brand: 'Google Chrome', version: '131' },
                        { brand: 'Chromium', version: '131' },
                        { brand: 'Not_A Brand', version: '24' }
                      ],
                      mobile: false,
                      platform: 'Windows',
                      getHighEntropyValues: async () => ({
                        architecture: 'x86',
                        bitness: '64',
                        model: '',
                        platform: 'Windows',
                        platformVersion: '10.0.0',
                        uaFullVersion: '131.0.0.0'
                      })
                    })
                  });
                }

                const notifyTitle = () => {
                    if (!document.title) return;
                    const title = document.title;
                    try {
                        if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
                            window.__TAURI__.core.invoke('native_title_changed', {
                                tabId: '{{TAB_ID}}',
                                title: title
                            }).catch(() => {});
                        } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                            window.__TAURI_INTERNALS__.invoke('native_title_changed', {
                                tabId: '{{TAB_ID}}',
                                title: title
                            }).catch(() => {});
                        }
                    } catch (_) {}
                };

                let lastTitle = document.title;
                const checkTitle = () => {
                    if (document.title && document.title !== lastTitle) {
                        lastTitle = document.title;
                        notifyTitle();
                    }
                };

                const notifyFavicon = () => {
                    let icon = '';
                    const links = document.querySelectorAll('link[rel="icon"], link[rel="shortcut icon"], link[rel="apple-touch-icon"]');
                    if (links.length > 0) {
                        for (let i = links.length - 1; i >= 0; i--) {
                            if (links[i].href) {
                                icon = links[i].href;
                                break;
                            }
                        }
                    }
                    if (!icon) {
                        icon = window.location.origin + '/favicon.ico';
                    }
                    try {
                        if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
                            window.__TAURI__.core.invoke('native_favicon_changed', {
                                tabId: '{{TAB_ID}}',
                                favicon: icon
                            }).catch(() => {});
                        } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                            window.__TAURI_INTERNALS__.invoke('native_favicon_changed', {
                                tabId: '{{TAB_ID}}',
                                favicon: icon
                            }).catch(() => {});
                        }
                    } catch (_) {}
                };

                let lastFavicon = '';
                const checkFavicon = () => {
                    const links = document.querySelectorAll('link[rel="icon"], link[rel="shortcut icon"], link[rel="apple-touch-icon"]');
                    let currentFavicon = window.location.origin + '/favicon.ico';
                    if (links.length > 0) {
                        for (let i = links.length - 1; i >= 0; i--) {
                            if (links[i].href) {
                                currentFavicon = links[i].href;
                                break;
                            }
                        }
                    }
                    if (currentFavicon !== lastFavicon) {
                        lastFavicon = currentFavicon;
                        notifyFavicon();
                    }
                };

                setInterval(() => {
                    checkTitle();
                    checkFavicon();
                }, 1000);

                window.addEventListener('DOMContentLoaded', notifyTitle);
                window.addEventListener('load', notifyTitle);

                const notifyUrl = (url) => {
                    try {
                        if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
                            window.__TAURI__.core.invoke('native_url_changed', {
                                tabId: '{{TAB_ID}}',
                                url: url
                            }).catch(() => {});
                        } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                            window.__TAURI_INTERNALS__.invoke('native_url_changed', {
                                tabId: '{{TAB_ID}}',
                                url: url
                            }).catch(() => {});
                        }
                    } catch (_) {}
                    setTimeout(notifyTitle, 200);
                };
                window.addEventListener('popstate', () => notifyUrl(window.location.href));
                const originalPushState = history.pushState;
                history.pushState = function() {
                    originalPushState.apply(this, arguments);
                    notifyUrl(window.location.href);
                };
                const originalReplaceState = history.replaceState;
                history.replaceState = function() {
                    originalReplaceState.apply(this, arguments);
                    notifyUrl(window.location.href);
                };

                const _callTauriAction = (action) => {
                    try {
                        window.open('tauri-action://' + action, '_blank');
                    } catch (_) {
                        try {
                            const a = document.createElement('a');
                            a.href = 'tauri-action://' + action;
                            a.target = '_blank';
                            a.click();
                        } catch (_) {}
                    }
                };

                window.addEventListener('keydown', (e) => {
                    if (e.key === 'F11') {
                        e.preventDefault();
                        e.stopPropagation();
                        _callTauriAction('toggle-fullscreen');
                    } else if (e.key === 'Escape') {
                        if (!document.fullscreenElement) {
                            _callTauriAction('exit-fullscreen');
                        }
                    }
                }, true);

                let _lastTopEdgeCheck = 0;
                window.addEventListener('mousemove', (e) => {
                    if (e.clientY <= 3) {
                        const now = Date.now();
                        if (now - _lastTopEdgeCheck > 200) {
                            _lastTopEdgeCheck = now;
                            _callTauriAction('show-fullscreen-header');
                        }
                    }
                }, { passive: true });

                document.addEventListener('fullscreenchange', () => {
                    const isElemFs = !!document.fullscreenElement;
                    _callTauriAction(isElemFs ? 'enter-fullscreen' : 'exit-fullscreen');
                });
        "#;
        let init_script = base_script.replace("{{TAB_ID}}", &t_id_ipc);

        let t_id_load = t_id.clone();
        let app_handle_new_win = app_handle.clone();
        let app_handle_nav = app_handle.clone();
        let builder = WebviewBuilder::new(&label, parsed_url)
            .incognito(true)
            .initialization_script(&init_script)
            .on_new_window(move |url, _features| {
                log::debug!("[NEW_WINDOW] Requested URL: {}", url);
                if url.scheme() == "tauri-action" {
                    let mut action = url.host_str().unwrap_or("");
                    if action.is_empty() {
                        action = url.path().trim_matches('/');
                    }
                    match action {
                        "toggle-fullscreen" => {
                            let _ = crate::commands::native_toggle_fullscreen(app_handle_new_win.clone());
                        }
                        "exit-fullscreen" => {
                            let _ = crate::commands::native_exit_fullscreen_if_active(app_handle_new_win.clone());
                        }
                        "enter-fullscreen" => {
                            let _ = crate::commands::native_set_fullscreen(app_handle_new_win.clone(), true);
                        }
                        "show-fullscreen-header" => {
                            let _ = crate::commands::native_show_fullscreen_header(app_handle_new_win.clone(), true);
                        }
                        _ => {}
                    }
                    return tauri::webview::NewWindowResponse::Deny;
                }

                let url_str = url.to_string();
                if !url_str.is_empty() && url_str != "about:blank" {
                    #[derive(serde::Serialize, Clone)]
                    struct NewTabPayload {
                        url: String,
                    }
                    use tauri::Emitter;
                    let _ = app_handle_new_win.emit("open-new-tab", NewTabPayload { url: url_str });
                }
                tauri::webview::NewWindowResponse::Deny
            })
            .on_navigation(move |url| {
                if url.scheme() == "tauri-action" {
                    let mut action = url.host_str().unwrap_or("");
                    if action.is_empty() {
                        action = url.path().trim_matches('/');
                    }
                    match action {
                        "toggle-fullscreen" => {
                            let _ = crate::commands::native_toggle_fullscreen(app_handle_nav.clone());
                        }
                        "exit-fullscreen" => {
                            let _ = crate::commands::native_exit_fullscreen_if_active(app_handle_nav.clone());
                        }
                        "enter-fullscreen" => {
                            let _ = crate::commands::native_set_fullscreen(app_handle_nav.clone(), true);
                        }
                        "show-fullscreen-header" => {
                            let _ = crate::commands::native_show_fullscreen_header(app_handle_nav.clone(), true);
                        }
                        _ => {}
                    }
                    return false;
                }
                true
            })
            .on_page_load(move |webview, _payload| {
                let _ = webview.eval(r#"
                    if (!sessionStorage.getItem('first_load_done')) {
                        sessionStorage.setItem('first_load_done', 'true');
                        window.location.reload();
                    }
                "#);

                if let Ok(url) = webview.url() {
                    #[allow(non_snake_case)]
                    #[derive(serde::Serialize, Clone)]
                    struct Payload {
                        tabId: String,
                        url: String,
                    }
                    let _ = webview.emit("webview-loaded", Payload {
                        tabId: t_id.clone(),
                        url: url.to_string(),
                    });

                    let script = format!(r#"
                        (function() {{
                            const t = document.title;
                            if (t) {{
                                if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {{
                                    window.__TAURI__.core.invoke('native_title_changed', {{ tabId: '{}', title: t }});
                                }} else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {{
                                    window.__TAURI_INTERNALS__.invoke('native_title_changed', {{ tabId: '{}', title: t }});
                                }}
                            }}
                        }})();
                    "#, t_id_load, t_id_load);
                    let _ = webview.eval(&script);
                }
            })
            .on_download(move |webview, event| {
                match event {
                    tauri::webview::DownloadEvent::Requested { url, destination } => {
                        log::debug!("[DOWNLOAD] Requested: {:?}", url);

                        let file_name = url.path_segments()
                            .and_then(|mut s| s.next_back())
                            .filter(|s| !s.is_empty())
                            .unwrap_or("download")
                            .to_string();
                        let file_name = file_name.split('?').next().unwrap_or(&file_name).to_string();

                        let dl_dir = if let Ok(profile) = std::env::var("USERPROFILE") {
                            std::path::PathBuf::from(profile).join("Downloads")
                        } else {
                            std::env::temp_dir()
                        };
                        *destination = dl_dir.join(&file_name);

                        log::debug!("[DOWNLOAD] Saving to: {:?}", destination);

                        #[derive(serde::Serialize, Clone)]
                        #[allow(non_snake_case)]
                        struct DlPayload {
                            tabId: String,
                            url: String,
                            filename: String,
                        }
                        use tauri::Emitter;
                        let _ = webview.emit("webview-download-started", DlPayload {
                            tabId: String::new(),
                            url: url.to_string(),
                            filename: file_name,
                        });
                        true
                    }
                    tauri::webview::DownloadEvent::Finished { url: _, path, success } => {
                        log::debug!("[DOWNLOAD] Finished: success={}, path={:?}", success, path);
                        #[derive(serde::Serialize, Clone)]
                        #[allow(non_snake_case)]
                        struct DlFinishedPayload {
                            success: bool,
                            path: String,
                        }
                        use tauri::Emitter;
                        let _ = webview.emit("webview-download-finished", DlFinishedPayload {
                            success,
                            path: path.map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                        });
                        true
                    }
                    _ => true
                }
            });

        let _wv = window
            .add_child(
                builder,
                tauri::LogicalPosition::new(-10000.0, -10000.0),
                tauri::LogicalSize::new(logical_size.width, content_height),
            )
            .map_err(|e| {
                log::error!("[TAB_MANAGER] ERROR adding child Webview: {:?}", e);
                format!("Failed to add child Webview: {}", e)
            })?;

        {
            let mut tabs_map = self.tabs.lock();
            tabs_map.insert(tab_id.to_string(), label.clone());
        }

        self.switch_to_tab(window, tab_id)?;

        Ok(())
    }

    pub fn hide_all_webviews(&self, window: &tauri::Window) {
        let app_handle = window.app_handle();
        let tabs_map = self.tabs.lock();
        for (_, label) in tabs_map.iter() {
            if let Some(wv) = app_handle.get_webview(label) {
                let _ = wv.set_position(tauri::LogicalPosition::new(-10000.0, -10000.0));
            }
        }
    }

    pub fn switch_to_tab(&self, window: &tauri::Window, tab_id: &str) -> Result<(), String> {
        log::debug!("[TAB_MANAGER] switch_to_tab: {}", tab_id);
        let app_handle = window.app_handle();

        let (win_w, top_bar_height, content_height) = self.get_target_geometry(window);
        let tabs_map = self.tabs.lock();
        let mut found_active = false;
        for (id, label) in tabs_map.iter() {
            if id == tab_id {
                if let Some(wv) = app_handle.get_webview(label) {
                    log::debug!("[TAB_MANAGER] Showing webview: {}", label);
                    let _ = wv.set_position(tauri::LogicalPosition::new(0.0, top_bar_height));
                    let _ = wv.set_size(tauri::LogicalSize::new(win_w, content_height));
                    let _ = wv.show();
                    let _ = wv.set_focus();
                    found_active = true;
                }
            }
        }
        
        for (id, label) in tabs_map.iter() {
            if id != tab_id {
                if let Some(wv) = app_handle.get_webview(label) {
                    let _ = wv.set_position(tauri::LogicalPosition::new(-10000.0, -10000.0));
                }
            }
        }

        if !found_active {
            log::debug!("[TAB_MANAGER] No webview found for tab_id: {}. Moving all offscreen.", tab_id);
            for (_, label) in tabs_map.iter() {
                if let Some(wv) = app_handle.get_webview(label) {
                    let _ = wv.set_position(tauri::LogicalPosition::new(-10000.0, -10000.0));
                }
            }
            
            let mut active = self.active_tab_id.lock();
            *active = Some(tab_id.to_string());
            
            return Err("NOT_FOUND".to_string());
        }

        let mut active = self.active_tab_id.lock();
        *active = Some(tab_id.to_string());

        Ok(())
    }

    pub fn close_tab(&self, window: &tauri::Window, tab_id: &str) -> Result<(), String> {
        let app_handle = window.app_handle();
        let label = {
            let mut tabs_map = self.tabs.lock();
            tabs_map.remove(tab_id)
        };

        log::debug!("[TAB_MANAGER] Closing tab_id: {} -> label: {:?}", tab_id, label);

        if let Some(lbl) = label {
            if let Some(wv) = app_handle.get_webview(&lbl) {
                log::debug!("[TAB_MANAGER] Found webview for label: {}. Destroying...", lbl);
                let _ = wv.hide();
                let _ = wv.close().map_err(|e| {
                    log::error!("[TAB_MANAGER] Error closing webview: {}", e);
                    e.to_string()
                });
            } else {
                log::warn!("[TAB_MANAGER] Webview for label {} not found in app_handle!", lbl);
            }
        }

        let mut active = self.active_tab_id.lock();
        if active.as_deref() == Some(tab_id) {
            *active = None;
        }

        Ok(())
    }

    pub fn set_zoom(&self, window: &tauri::Window, tab_id: &str, zoom_factor: f64) -> Result<(), String> {
        let app_handle = window.app_handle();
        let tabs_map = self.tabs.lock();
        if let Some(label) = tabs_map.get(tab_id) {
            if let Some(wv) = app_handle.get_webview(label) {
                let _ = wv.set_zoom(zoom_factor);
            }
        }
        Ok(())
    }

    pub fn close_all_tabs(&self, window: &tauri::Window) -> Result<(), String> {
        let app_handle = window.app_handle();
        let labels: Vec<String> = {
            let mut tabs_map = self.tabs.lock();
            let list = tabs_map.values().cloned().collect();
            tabs_map.clear();
            list
        };

        for lbl in labels {
            if let Some(wv) = app_handle.get_webview(&lbl) {
                let _ = wv.hide();
                let _ = wv.close();
            }
        }

        let mut active = self.active_tab_id.lock();
        *active = None;

        Ok(())
    }

    pub fn sync_main_window_geometry(&self, window: &tauri::Window) {
        let active_id = self.active_tab_id.lock().clone();
        if let Some(id) = active_id {
            let label = {
                let tabs_map = self.tabs.lock();
                tabs_map.get(&id).cloned()
            };
            
            if let Some(lbl) = label {
                if let Some(wv) = window.app_handle().get_webview(&lbl) {
                    let (win_w, top_bar_height, content_height) = self.get_target_geometry(window);
                    let _ = wv.set_position(tauri::LogicalPosition::new(0.0, top_bar_height));
                    let _ = wv.set_size(tauri::LogicalSize::new(win_w, content_height));
                }
            }
        }
    }

    pub fn sync_main_window_focus(&self, window: &tauri::Window, focused: bool) {
        let app_handle = window.app_handle();
        let active_id = self.active_tab_id.lock().clone();

        if let Some(id) = active_id {
            let tabs_map = self.tabs.lock();
            if let Some(label) = tabs_map.get(&id) {
                if let Some(win) = app_handle.get_webview_window(label) {
                    if focused {
                        let _ = win.show();
                    } else {
                        let _ = win.hide();
                    }
                }
            }
        }
    }

    pub fn go_back(&self, window: &tauri::Window, tab_id: &str) -> Result<(), String> {
        let app_handle = window.app_handle();
        let tabs_map = self.tabs.lock();
        if let Some(label) = tabs_map.get(tab_id) {
            if let Some(wv) = app_handle.get_webview(label) {
                let _ = wv.eval("window.history.back();");
            }
        }
        Ok(())
    }

    pub fn go_forward(&self, window: &tauri::Window, tab_id: &str) -> Result<(), String> {
        let app_handle = window.app_handle();
        let tabs_map = self.tabs.lock();
        if let Some(label) = tabs_map.get(tab_id) {
            if let Some(wv) = app_handle.get_webview(label) {
                let _ = wv.eval("window.history.forward();");
            }
        }
        Ok(())
    }

    pub fn reload(&self, window: &tauri::Window, tab_id: &str) -> Result<(), String> {
        let app_handle = window.app_handle();
        let tabs_map = self.tabs.lock();
        if let Some(label) = tabs_map.get(tab_id) {
            if let Some(wv) = app_handle.get_webview(label) {
                let _ = wv.eval("window.location.reload();");
            }
        }
        Ok(())
    }
}

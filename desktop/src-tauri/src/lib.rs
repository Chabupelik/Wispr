pub mod browser;
pub mod commands;
pub mod error;
pub mod keyboard_hook;
pub mod security;
pub mod tunnel;

use browser::TabManager;
use commands::*;
use tauri::Manager;
use tunnel::TunnelManager;

#[tauri::command]
fn force_exit(app: tauri::AppHandle) {
    if let Some(tm) = app.try_state::<TunnelManager>() {
        tm.shutdown();
    }
    keyboard_hook::stop();
    std::process::exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let webview_data_dir = std::env::temp_dir().join("ShellCache");
    crate::security::wipe::clean_directory(&webview_data_dir);
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &webview_data_dir);

    let (listener, proxy_port) = match std::net::TcpListener::bind("127.0.0.1:0") {
        Ok(l) => {
            let port = l.local_addr().map(|a| a.port()).unwrap_or(0);
            (Some(l), port)
        }
        Err(e) => {
            log::error!("Failed to bind local loopback proxy port: {}", e);
            (None, 0)
        }
    };
    let api_domain = option_env!("VITE_API_DOMAIN").unwrap_or("").to_string();
    let proxy_args = format!(
        "--proxy-server=127.0.0.1:{} --proxy-bypass-list=127.0.0.1,localhost,tauri.localhost,ipc.localhost,{} --disk-cache-size=1 --media-cache-size=1 --disable-http-cache --disable-features=AutofillServerCommunication,PasswordManager,UserAgentClientHint --disable-save-password-bubble --disable-single-click-autofill --enforce-webrtc-ip-permission-check --force-webrtc-ip-handling-policy=disable_non_proxied_udp --disable-quic --user-agent=\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36\" --disable-blink-features=AutomationControlled",
        proxy_port, api_domain
    );

    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", proxy_args);

    let tunnel_manager = TunnelManager::new();
    let tab_manager = TabManager::new();

    if let Some(listener) = listener {
        let _ = listener.set_nonblocking(true);
        let tm_clone = tunnel_manager.clone();
        tauri::async_runtime::spawn(async move {
            match tokio::net::TcpListener::from_std(listener) {
                Ok(tokio_listener) => {
                    if let Err(e) = tm_clone.init_global_proxy_listener(tokio_listener).await {
                        log::error!("Не удалось запустить локальный прокси: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Ошибка инициализации tokio listener: {}", e);
                }
            }
        });
    }

    if let Err(e) = tauri::Builder::default()
        .manage(tunnel_manager)
        .manage(tab_manager)
        .setup(move |app| {
            #[cfg(debug_assertions)]
            {
                let desktop_path = if let Ok(user_profile) = std::env::var("USERPROFILE") {
                    std::path::PathBuf::from(user_profile).join("Desktop")
                } else {
                    std::env::temp_dir()
                };

                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Debug)
                        .target(tauri_plugin_log::Target::new(
                            tauri_plugin_log::TargetKind::Folder {
                                path: desktop_path,
                                file_name: Some("wisp_debug".to_string()),
                            },
                        ))
                        .build(),
                )?;

                log::info!("=== WISP DIAGNOSTIC START ===");
                log::info!("[CONFIG] VITE_API_BASE = {}", option_env!("VITE_API_BASE").unwrap_or("<не задан>"));
                log::info!("[CONFIG] VITE_API_DOMAIN = {}", option_env!("VITE_API_DOMAIN").unwrap_or("<не задан>"));
                log::info!("[CONFIG] LOCAL_PROXY_PORT = {}", proxy_port);
                log::info!("[CONFIG] VITE_UPSTREAM_PROXY = {}", option_env!("VITE_UPSTREAM_PROXY").unwrap_or("<не задан — прямое соединение>"));
                log::info!("[CONFIG] WISP_SALT задан = {}", if option_env!("WISP_SALT").is_some() { "да" } else { "НЕТ — ошибка!" });
                log::info!("=== WISP DIAGNOSTIC END ===");
            }

            keyboard_hook::start(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_device_id,
            start_tunnel,
            stop_tunnel,
            native_navigate_tab,
            native_switch_tab,
            native_close_tab,
            native_go_back,
            native_go_forward,
            native_reload,
            native_close_all_tabs,
            native_update_top_bar_height,
            native_minimize_window,
            native_maximize_window,
            native_toggle_fullscreen,
            native_set_fullscreen,
            native_is_fullscreen,
            native_show_fullscreen_header,
            native_exit_fullscreen_if_active,
            native_close_window,
            native_start_dragging,
            native_title_changed,
            native_favicon_changed,
            native_set_window_title,
            native_context_menu,
            native_url_changed,
            native_set_zoom,
            native_open_path,
            native_show_in_folder,
            native_get_downloads_dir,
            native_open_new_tab,
            self_destruct,
            force_exit,
        ])
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Destroyed => {
                    keyboard_hook::stop();
                    if let Some(tm) = window.app_handle().try_state::<TunnelManager>() {
                        tm.shutdown();
                    }
                }
                tauri::WindowEvent::CloseRequested { .. } => {}
                tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                    let app = window.app_handle();
                    let win = app.get_window("main").or_else(|| app.windows().into_values().next());
                    if let Some(main_win) = win {
                        let tab_mgr = app.state::<TabManager>();
                        tab_mgr.sync_main_window_geometry(&main_win);
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
    {
        log::error!("Error while running tauri application: {}", e);
    }
}

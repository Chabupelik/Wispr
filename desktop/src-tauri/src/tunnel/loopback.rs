use crate::tunnel::wisp::WispClient;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use parking_lot::Mutex;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProxyMode {
    Transparent,
    Tunnel,
}

pub struct LoopbackProxyServer {
    pub port: u16,
    pub wisp_client: Arc<Mutex<Option<WispClient>>>,
    pub mode: Arc<Mutex<ProxyMode>>,
    shutdown_tx: watch::Sender<bool>,
}

impl LoopbackProxyServer {
    pub async fn start(proxy_port: u16) -> Result<Self, String> {
        let listener = if proxy_port == 0 {
            TcpListener::bind("127.0.0.1:0").await
        } else {
            match TcpListener::bind(format!("127.0.0.1:{}", proxy_port)).await {
                Ok(l) => Ok(l),
                Err(_) => TcpListener::bind("127.0.0.1:0").await,
            }
        }
        .map_err(|e| format!("Failed to bind proxy listener: {}", e))?;

        Self::from_listener(listener)
    }

    pub fn from_listener(listener: TcpListener) -> Result<Self, String> {
        let port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get local port: {}", e))?
            .port();

        let wisp_client = Arc::new(Mutex::new(None::<WispClient>));
        let mode = Arc::new(Mutex::new(ProxyMode::Transparent));
        let wisp_clone = wisp_client.clone();
        let mode_clone = mode.clone();
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        tokio::spawn(async move {
            log::info!("[LOOPBACK PROXY] Запущен на 127.0.0.1:{}", port);

            loop {
                tokio::select! {
                    accept_res = listener.accept() => {
                        match accept_res {
                            Ok((stream, _)) => {
                                let w = wisp_clone.clone();
                                let m = mode_clone.clone();
                                let mut conn_shutdown = shutdown_rx.clone();
                                tokio::spawn(async move {
                                    tokio::select! {
                                        res = handle_client_connection(stream, w, m, port) => {
                                            if let Err(e) = res {
                                                log::debug!("[LOOPBACK] Ошибка обработки соединения: {:?}", e);
                                            }
                                        }
                                        _ = conn_shutdown.changed() => {
                                            log::debug!("[LOOPBACK] Соединение закрыто по сигналу завершения");
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                log::error!("[LOOPBACK] Ошибка accept: {:?}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        log::info!("[LOOPBACK PROXY] Завершение работы прокси-сервера");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            port,
            wisp_client,
            mode,
            shutdown_tx,
        })
    }

    pub fn set_mode(&self, new_mode: ProxyMode) {
        let mut guard = self.mode.lock();
        log::info!("[LOOPBACK PROXY] Смена режима: {:?} -> {:?}", *guard, new_mode);
        *guard = new_mode;
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

impl Drop for LoopbackProxyServer {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(true);
    }
}

fn get_live_wisp_client(wisp_ref: &Arc<Mutex<Option<WispClient>>>) -> Result<WispClient, String> {
    let mut guard = wisp_ref.lock();
    match guard.as_ref() {
        Some(client) => {
            if client.is_alive() {
                Ok(client.clone())
            } else {
                log::warn!("[LOOPBACK] WISP клиент мёртв (WebSocket разорван), очищаем");
                *guard = None;
                Err("WISP-соединение разорвано (WebSocket dead)".to_string())
            }
        }
        None => Err("WISP-клиент не подключён".to_string()),
    }
}

async fn send_502_error(
    client_stream: &mut TcpStream,
    reason: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let body = format!(
        "<html><head><title>502 Proxy Tunnel Error</title></head>\
         <body style='background:#1a1a2e;color:#e0e0e0;font-family:monospace;display:flex;\
         align-items:center;justify-content:center;height:100vh;margin:0;'>\
         <div style='text-align:center;max-width:600px;'>\
         <h1 style='color:#e74c3c;font-size:48px;margin-bottom:10px;'>⚡ 502</h1>\
         <h2 style='color:#f39c12;'>Защищённый туннель недоступен</h2>\
         <p style='color:#bbb;font-size:14px;line-height:1.6;'>{}</p>\
         <p style='color:#666;font-size:12px;margin-top:20px;'>Соединение через прокси-туннель временно невозможно.<br>\
         Запрос <b>НЕ был отправлен напрямую</b> для защиты вашего IP.</p>\
         </div></body></html>",
        reason
    );
    let response = format!(
        "HTTP/1.1 502 Bad Gateway\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    client_stream.write_all(response.as_bytes()).await?;
    Ok(())
}

async fn handle_client_connection(
    mut client_stream: TcpStream,
    wisp_ref: Arc<Mutex<Option<WispClient>>>,
    mode_ref: Arc<Mutex<ProxyMode>>,
    proxy_port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = [0u8; 8192];
    let n = client_stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let request_str = String::from_utf8_lossy(&buf[..n]);
    let mut lines = request_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return Ok(()),
    };
    log::info!("[LOOPBACK REQ] {}", request_line);

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");

    let current_mode = {
        let guard = mode_ref.lock();
        *guard
    };

    let mut header_end = 0;
    for i in 0..n.saturating_sub(3) {
        if &buf[i..i + 4] == b"\r\n\r\n" {
            header_end = i + 4;
            break;
        }
    }

    if method.eq_ignore_ascii_case("CONNECT") {
        let (host, port) = parse_host_port(target, 443);

        match current_mode {
            ProxyMode::Tunnel => {
                match get_live_wisp_client(&wisp_ref) {
                    Ok(wisp_client) => {
                        match wisp_client.create_stream(&host, port).await {
                            Ok(mut wisp_stream) => {
                                client_stream
                                    .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                                    .await?;
                                if header_end > 0 && n > header_end {
                                    wisp_stream.write_all(&buf[header_end..n]).await?;
                                }
                                let _ = tokio::io::copy_bidirectional(&mut client_stream, &mut wisp_stream).await;
                            }
                            Err(e) => {
                                log::error!("[LOOPBACK TUNNEL] Ошибка создания WISP-стрима для {}:{}: {}", host, port, e);
                                client_stream
                                    .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\n\r\n")
                                    .await?;
                            }
                        }
                    }
                    Err(reason) => {
                        log::warn!("[LOOPBACK TUNNEL] CONNECT {}:{} ЗАБЛОКИРОВАН — WISP недоступен: {}", host, port, reason);
                        client_stream
                            .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\n\r\n")
                            .await?;
                    }
                }
            }
            ProxyMode::Transparent => {
                match connect_upstream(host.as_str(), port).await {
                    Ok(mut direct_stream) => {
                        client_stream
                            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                            .await?;
                        if header_end > 0 && n > header_end {
                            direct_stream.write_all(&buf[header_end..n]).await?;
                        }
                        let _ = tokio::io::copy_bidirectional(&mut client_stream, &mut direct_stream).await;
                    }
                    Err(e) => {
                        log::error!("[LOOPBACK] Upstream connection failed for {}:{}: {}", host, port, e);
                        let _ = client_stream
                            .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\n\r\n")
                            .await;
                    }
                }
            }
        }
    } else {
        if target.ends_with("/proxy.pac") {
            let pac_script = format!(
                "function FindProxyForURL(url, host) {{ if (shExpMatch(host, '127.0.0.1') || shExpMatch(host, 'localhost') || shExpMatch(host, 'tauri.localhost') || shExpMatch(host, 'ipc.localhost')) return 'DIRECT'; return 'PROXY 127.0.0.1:{}'; }}",
                proxy_port
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-ns-proxy-autoconfig\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                pac_script.len(),
                pac_script
            );
            client_stream.write_all(response.as_bytes()).await?;
            return Ok(());
        }

        let (host, port) = parse_http_target(target, &request_str);

        match current_mode {
            ProxyMode::Tunnel => {
                match get_live_wisp_client(&wisp_ref) {
                    Ok(wisp_client) => {
                        match wisp_client.create_stream(&host, port).await {
                            Ok(mut wisp_stream) => {
                                wisp_stream.write_all(&buf[..n]).await?;
                                let _ = tokio::io::copy_bidirectional(&mut client_stream, &mut wisp_stream).await;
                            }
                            Err(e) => {
                                log::error!("[LOOPBACK TUNNEL] Ошибка создания HTTP WISP-стрима для {}:{}: {}", host, port, e);
                                send_502_error(&mut client_stream, &format!(
                                    "Не удалось установить туннельное соединение к <b>{}:{}</b>.<br>Ошибка: {}",
                                    host, port, e
                                )).await?;
                            }
                        }
                    }
                    Err(reason) => {
                        log::warn!("[LOOPBACK TUNNEL] HTTP {}:{} ЗАБЛОКИРОВАН — WISP недоступен: {}", host, port, reason);
                        send_502_error(&mut client_stream, &format!(
                            "Защищённый туннель к VPS не установлен.<br>Причина: {}<br>\
                             Запрос к <b>{}:{}</b> отклонён для защиты вашего реального IP.",
                            reason, host, port
                        )).await?;
                    }
                }
            }
            ProxyMode::Transparent => {
                let mut direct_stream = connect_upstream(host.as_str(), port).await?;
                direct_stream.write_all(&buf[..n]).await?;
                let _ = tokio::io::copy_bidirectional(&mut client_stream, &mut direct_stream).await;
            }
        }
    }

    Ok(())
}

fn parse_host_port(target: &str, default_port: u16) -> (String, u16) {
    if let Some((h, p)) = target.split_once(':') {
        (h.to_string(), p.parse::<u16>().unwrap_or(default_port))
    } else {
        (target.to_string(), default_port)
    }
}

fn parse_http_target(target: &str, request_str: &str) -> (String, u16) {
    if let Ok(url) = url::Url::parse(target) {
        let host = url.host_str().unwrap_or("127.0.0.1").to_string();
        let port = url.port().unwrap_or(80);
        (host, port)
    } else {
        for line in request_str.lines() {
            if line.to_lowercase().starts_with("host:") {
                let host_val = line[5..].trim();
                return parse_host_port(host_val, 80);
            }
        }
        parse_host_port(target, 80)
    }
}

async fn connect_upstream(host: &str, port: u16) -> std::io::Result<TcpStream> {
    let upstream_proxy = option_env!("VITE_UPSTREAM_PROXY").unwrap_or("").to_string();
    log::info!("[UPSTREAM] connect_upstream to {}:{} via proxy='{}'", host, port, upstream_proxy);
    if !upstream_proxy.is_empty() {
        if let Ok(proxy_url) = url::Url::parse(&upstream_proxy) {
            if let Some(proxy_host) = proxy_url.host_str() {
                let proxy_port = proxy_url.port_or_known_default().unwrap_or(3128);
                let mut stream = match TcpStream::connect((proxy_host, proxy_port)).await {
                    Ok(s) => s,
                    Err(_) if proxy_host == "localhost" => {
                        TcpStream::connect(("127.0.0.1", proxy_port)).await?
                    }
                    Err(e) => return Err(e),
                };
                
                let connect_req = format!("CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n", host, port, host, port);
                use tokio::io::AsyncWriteExt;
                stream.write_all(connect_req.as_bytes()).await?;
                
                let mut buf = [0u8; 4096];
                let mut read_len = 0;
                loop {
                    use tokio::io::AsyncReadExt;
                    let n = stream.read(&mut buf[read_len..]).await?;
                    if n == 0 { 
                        return Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "Proxy closed connection"));
                    }
                    read_len += n;
                    if buf[..read_len].windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                let response = String::from_utf8_lossy(&buf[..read_len]);
                if !response.starts_with("HTTP/1.1 200") && !response.starts_with("HTTP/1.0 200") {
                    return Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, format!("Proxy returned error: {}", response)));
                }
                return Ok(stream);
            }
        }
    }
    TcpStream::connect((host, port)).await
}

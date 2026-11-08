pub mod loopback;
pub mod wisp;

use loopback::{LoopbackProxyServer, ProxyMode};
use parking_lot::Mutex;
use std::sync::Arc;
use wisp::WispClient;

#[derive(Clone)]
pub struct TunnelManager {
    pub proxy_server: Arc<Mutex<Option<LoopbackProxyServer>>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ProxyConnectionInfo {
    pub local_port: u16,
    pub session_secret: String,
    pub is_active: bool,
}

impl Default for TunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            proxy_server: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn init_global_proxy_listener(&self, listener: tokio::net::TcpListener) -> Result<(), String> {
        let server = LoopbackProxyServer::from_listener(listener)?;
        let mut guard = self.proxy_server.lock();
        if let Some(old) = guard.take() {
            old.shutdown();
        }
        *guard = Some(server);
        Ok(())
    }

    pub async fn init_global_proxy(&self, proxy_port: u16) -> Result<(), String> {
        let server = LoopbackProxyServer::start(proxy_port).await?;
        let mut guard = self.proxy_server.lock();
        if let Some(old) = guard.take() {
            old.shutdown();
        }
        *guard = Some(server);
        Ok(())
    }

    pub async fn start(&self, wisp_base_url: &str, jwt_token: &str) -> Result<ProxyConnectionInfo, String> {
        let formatted_url = if wisp_base_url.ends_with('/') {
            format!("{}{}", wisp_base_url, urlencoding::encode(jwt_token))
        } else {
            format!("{}/{}", wisp_base_url, urlencoding::encode(jwt_token))
        };

        log::info!("[TUNNEL] Connecting to WISP: {}", wisp_base_url);

        let wisp_client = WispClient::connect(&formatted_url).await.map_err(|e| {
            log::error!("[TUNNEL] WispClient::connect() failed: {}", e);
            e
        })?;
        log::info!("[TUNNEL] WispClient::connect() success");

        let port = {
            let guard = self.proxy_server.lock();
            if let Some(server) = guard.as_ref() {
                {
                    let mut wisp_guard = server.wisp_client.lock();
                    *wisp_guard = Some(wisp_client);
                }
                server.set_mode(ProxyMode::Tunnel);
                server.port
            } else {
                return Err("Глобальный прокси не инициализирован!".to_string());
            }
        };

        Ok(ProxyConnectionInfo {
            local_port: port,
            session_secret: "unused".to_string(),
            is_active: true,
        })
    }

    pub fn stop(&self) {
        let guard = self.proxy_server.lock();
        if let Some(server) = guard.as_ref() {
            server.set_mode(ProxyMode::Transparent);
            let mut wisp_guard = server.wisp_client.lock();
            if let Some(client) = wisp_guard.as_ref() {
                client.close();
            }
            *wisp_guard = None;
        }
    }

    pub fn shutdown(&self) {
        self.stop();
        let mut guard = self.proxy_server.lock();
        if let Some(server) = guard.take() {
            server.shutdown();
        }
    }

    pub fn get_info(&self) -> Option<ProxyConnectionInfo> {
        let guard = self.proxy_server.lock();
        if let Some(server) = guard.as_ref() {
            let mode = {
                let mode_guard = server.mode.lock();
                *mode_guard
            };
            if mode == ProxyMode::Tunnel {
                return Some(ProxyConnectionInfo {
                    local_port: server.port,
                    session_secret: "unused".to_string(),
                    is_active: true,
                });
            }
        }
        None
    }
}

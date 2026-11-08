use byteorder::LittleEndian;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use rand::Rng;
pub const WISP_TYPE_CONNECT: u8 = 0x01;
pub const WISP_TYPE_DATA: u8 = 0x02;
pub const WISP_TYPE_CONTINUE: u8 = 0x03;
pub const WISP_TYPE_CLOSE: u8 = 0x04;
pub const WISP_STREAM_TCP: u8 = 0x01;

#[derive(Clone)]
pub struct WispClient {
    ws_tx: mpsc::Sender<Message>,
    stream_channels: Arc<Mutex<HashMap<u32, mpsc::Sender<Vec<u8>>>>>,
    next_stream_id: Arc<Mutex<u32>>,
    /// Флаг жизни WebSocket-соединения. Когда sender или receiver задача завершается
    /// (WebSocket разорван), флаг ставится в false.
    alive: Arc<AtomicBool>,
}

impl WispClient {
    /// Проверяет, живо ли WebSocket-соединение к WISP-серверу.
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    /// Explicitly close the WISP websocket connection
    pub fn close(&self) {
        let _ = self.ws_tx.try_send(Message::Close(None));
    }

    pub async fn connect(wisp_url: &str) -> Result<Self, String> {
        log::info!("[WISP] connect() вызван с URL: {}", wisp_url);

        let parsed_url = Url::parse(wisp_url).map_err(|e| {
            log::error!("[WISP] Некорректный URL '{}': {}", wisp_url, e);
            format!("Некорректный WISP URL: {}", e)
        })?;
        let host = parsed_url.host_str().ok_or_else(|| {
            log::error!("[WISP] Отсутствует хост в URL: {}", wisp_url);
            "Отсутствует хост в WISP URL".to_string()
        })?.to_string();
        let port = parsed_url.port_or_known_default().unwrap_or(443);
        log::info!("[WISP] Целевой хост: {}:{}", host, port);

        // Проверяем наличие апстрим-прокси
        let upstream_proxy = option_env!("VITE_UPSTREAM_PROXY").unwrap_or("").to_string();
        log::info!("[WISP] VITE_UPSTREAM_PROXY = '{}'", if upstream_proxy.is_empty() { "<пусто — прямое соединение>" } else { &upstream_proxy });

        let tcp_stream = if !upstream_proxy.is_empty() {
            let proxy_url = Url::parse(&upstream_proxy).map_err(|e| {
                log::error!("[WISP] Некорректный VITE_UPSTREAM_PROXY '{}': {}", upstream_proxy, e);
                format!("Некорректный VITE_UPSTREAM_PROXY: {}", e)
            })?;
            let proxy_host = proxy_url.host_str().ok_or_else(|| {
                log::error!("[WISP] Отсутствует хост в прокси URL: {}", upstream_proxy);
                "Отсутствует хост в прокси".to_string()
            })?;
            let proxy_port = proxy_url.port_or_known_default().unwrap_or(3128);

            log::info!("[WISP] Подключаемся к прокси {}:{}...", proxy_host, proxy_port);
            let mut stream = TcpStream::connect((proxy_host, proxy_port)).await
                .map_err(|e| {
                    log::error!("[WISP] TCP-соединение с прокси {}:{} ПРОВАЛИЛОСЬ: {}", proxy_host, proxy_port, e);
                    format!("Не удалось подключиться к прокси {}:{}: {}", proxy_host, proxy_port, e)
                })?;
            log::info!("[WISP] TCP к прокси {}:{} — ОК", proxy_host, proxy_port);

            let connect_req = format!("CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n", host, port, host, port);
            log::info!("[WISP] Отправляем CONNECT {}:{} через прокси...", host, port);
            stream.write_all(connect_req.as_bytes()).await.map_err(|e| {
                log::error!("[WISP] Ошибка отправки CONNECT-запроса: {}", e);
                format!("Ошибка отправки CONNECT: {}", e)
            })?;

            let mut buf = [0u8; 4096];
            let mut read_len = 0;
            loop {
                let n = stream.read(&mut buf[read_len..]).await.map_err(|e| {
                    log::error!("[WISP] Ошибка чтения ответа прокси: {}", e);
                    format!("Ошибка чтения ответа прокси: {}", e)
                })?;
                if n == 0 {
                    log::error!("[WISP] Прокси закрыл соединение до ответа (прочитано {} байт)", read_len);
                    return Err("Прокси закрыл соединение до ответа".to_string());
                }
                read_len += n;
                if buf[..read_len].windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            let response = String::from_utf8_lossy(&buf[..read_len]);
            log::info!("[WISP] Ответ прокси на CONNECT: {}", response.lines().next().unwrap_or("<пустой>"));
            if !response.starts_with("HTTP/1.1 200") && !response.starts_with("HTTP/1.0 200") {
                log::error!("[WISP] Прокси отказал в CONNECT: {}", response);
                return Err(format!("Прокси вернул ошибку: {}", response));
            }
            log::info!("[WISP] CONNECT через прокси — ОК, переходим к TLS");
            stream
        } else {
            log::info!("[WISP] Прямое TCP-подключение к {}:{}...", host, port);
            let stream = TcpStream::connect((host.as_str(), port))
                .await
                .map_err(|e| {
                    log::error!("[WISP] Прямое TCP к {}:{} ПРОВАЛИЛОСЬ: {}", host, port, e);
                    format!("Не удалось подключиться к WISP хосту напрямую {}:{}: {}", host, port, e)
                })?;
            log::info!("[WISP] Прямое TCP к {}:{} — ОК", host, port);
            stream
        };

        let token = parsed_url.path().trim_start_matches("/v1/live/");
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
        let salt = obfstr::obfstr!(env!("WISP_SALT")).to_string();
        
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}", token, timestamp, salt).as_bytes());
        let signature = format!("{:x}", hasher.finalize());

        // Устанавливаем TLS и поднимаем WebSocket
        let request = Request::builder()
            .uri(wisp_url)
            .header("Host", host.as_str())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key())
            .header("X-Wisp-Timestamp", &timestamp)
            .header("X-Wisp-Signature", &signature)
            .body(())
            .map_err(|e| format!("Ошибка формирования WebSocket запроса: {}", e))?;

        // Создаем TLS коннектор, который игнорирует ошибки сертификатов (нужно для обхода SSL-Bumping прокси ЕСПД)
        log::info!("[WISP] Создаём TLS коннектор (accept_invalid_certs=true для ЕСПД SSL-bump)...");
        let mut builder = native_tls::TlsConnector::builder();
        builder.danger_accept_invalid_certs(true);
        builder.danger_accept_invalid_hostnames(true);
        let connector = builder.build().map_err(|e| {
            log::error!("[WISP] Ошибка создания TLS коннектора: {}", e);
            format!("Ошибка создания TLS коннектора: {}", e)
        })?;

        log::info!("[WISP] Выполняем WebSocket handshake с {}...", wisp_url);
        let (ws_stream, ws_response) = tokio_tungstenite::client_async_tls_with_config(
            request, 
            tcp_stream, 
            None,
            Some(tokio_tungstenite::Connector::NativeTls(connector))
        )
        .await
        .map_err(|e| {
            log::error!("[WISP] WebSocket handshake ПРОВАЛИЛСЯ: {}", e);
            format!("Не удалось выполнить WebSocket handshake: {}", e)
        })?;
        log::info!("[WISP] WebSocket handshake — ОК! HTTP статус: {}", ws_response.status());

        let (mut ws_sink, mut ws_source) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<Message>(1000);

        let mut key_hasher = Sha256::new();
        key_hasher.update(salt.as_bytes());
        let key_bytes: [u8; 32] = key_hasher.finalize().into();
        let aes_key = *Key::<Aes256Gcm>::from_slice(&key_bytes);

        let stream_channels = Arc::new(Mutex::new(HashMap::<u32, mpsc::Sender<Vec<u8>>>::new()));
        let stream_channels_rx = stream_channels.clone();

        let alive = Arc::new(AtomicBool::new(true));
        let alive_tx = alive.clone();
        let alive_rx = alive.clone();

        let cipher_tx = Aes256Gcm::new(&aes_key);
        // Клонируем URL в owned String для захвата в spawn (требует 'static)
        let wisp_url_owned = wisp_url.to_string();
        // Sender task: пересылает сообщения из канала в WebSocket
        tokio::spawn(async move {
            let wisp_url = wisp_url_owned;
            while let Some(msg) = rx.recv().await {
                let msg = match msg {
                    Message::Binary(data) => {
                        let mut nonce_bytes = [0u8; 12];
                        rand::thread_rng().fill(&mut nonce_bytes);
                        let nonce = Nonce::from_slice(&nonce_bytes);
                        match cipher_tx.encrypt(nonce, data.as_ref()) {
                            Ok(ciphertext) => {
                                let mut final_payload = Vec::with_capacity(12 + ciphertext.len());
                                final_payload.extend_from_slice(&nonce_bytes);
                                final_payload.extend_from_slice(&ciphertext);
                                Message::Binary(final_payload.into())
                            },
                            Err(e) => {
                                log::error!("[WISP] Ошибка шифрования: {:?}", e);
                                continue;
                            }
                        }
                    },
                    other => other,
                };

                if ws_sink.send(msg).await.is_err() {
                    break;
                }
            }
            alive_tx.store(false, Ordering::SeqCst);
            log::warn!("[WISP] WebSocket sender task завершилась — соединение разорвано. URL был: {}", wisp_url);
        });

        let cipher_rx = Aes256Gcm::new(&aes_key);
        // Receiver task: читает из WebSocket и раздаёт по stream-каналам
        tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_source.next().await {
                if let Message::Binary(mut data) = msg {
                    if data.len() < 28 { // 12 (IV) + 16 (AuthTag)
                        continue;
                    }
                    let nonce = Nonce::from_slice(&data[0..12]);
                    match cipher_rx.decrypt(nonce, &data[12..]) {
                        Ok(plaintext) => {
                            data = plaintext.into();
                        },
                        Err(e) => {
                            log::error!("[WISP] Ошибка расшифровки: {:?}", e);
                            continue;
                        }
                    }

                    if data.len() < 5 {
                        continue;
                    }

                    let packet_type = data[0];
                    let mut cursor = Cursor::new(&data[1..5]);
                    let stream_id = byteorder::ReadBytesExt::read_u32::<LittleEndian>(&mut cursor).unwrap_or(0);

                    if packet_type == WISP_TYPE_DATA {
                        let payload = data[5..].to_vec();
                        let channels = stream_channels_rx.lock().await;
                        if let Some(ch) = channels.get(&stream_id) {
                            let _ = ch.send(payload).await;
                        }
                    } else if packet_type == WISP_TYPE_CLOSE {
                        let mut channels = stream_channels_rx.lock().await;
                        channels.remove(&stream_id);
                    } else if packet_type == WISP_TYPE_CONTINUE {
                        // CONTINUE is used for flow control in WISP v2. We ignore it for now as we don't strictly enforce flow control.
                    }
                }
            }
            alive_rx.store(false, Ordering::SeqCst);
            log::warn!("[WISP] WebSocket receiver task завершилась — соединение разорвано");
        });

        Ok(Self {
            ws_tx: tx,
            stream_channels,
            next_stream_id: Arc::new(Mutex::new(1)),
            alive,
        })
    }

    pub async fn create_stream(&self, host: &str, port: u16) -> Result<WispStream, String> {
        let stream_id = {
            let mut id = self.next_stream_id.lock().await;
            let current = *id;
            *id += 1;
            current
        };

        let (data_tx, data_rx) = mpsc::channel::<Vec<u8>>(500);
        {
            let mut channels = self.stream_channels.lock().await;
            channels.insert(stream_id, data_tx);
        }

        let mut packet = Vec::new();
        packet.push(WISP_TYPE_CONNECT);
        byteorder::WriteBytesExt::write_u32::<LittleEndian>(&mut packet, stream_id).unwrap();
        packet.push(WISP_STREAM_TCP);
        byteorder::WriteBytesExt::write_u16::<LittleEndian>(&mut packet, port).unwrap();
        packet.extend_from_slice(host.as_bytes());

        self.ws_tx
            .send(Message::Binary(packet.into()))
            .await
            .map_err(|e| format!("Ошибка отправки CONNECT в WISP: {}", e))?;

        Ok(WispStream {
            stream_id,
            ws_tx: self.ws_tx.clone(),
            data_rx,
            read_buffer: Vec::new(),
        })
    }
}

pub struct WispStream {
    stream_id: u32,
    ws_tx: mpsc::Sender<Message>,
    data_rx: mpsc::Receiver<Vec<u8>>,
    read_buffer: Vec<u8>,
}

impl tokio::io::AsyncRead for WispStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if !self.read_buffer.is_empty() {
            let to_read = std::cmp::min(buf.remaining(), self.read_buffer.len());
            buf.put_slice(&self.read_buffer[..to_read]);
            self.read_buffer.drain(..to_read);
            return std::task::Poll::Ready(Ok(()));
        }

        match self.data_rx.poll_recv(cx) {
            std::task::Poll::Ready(Some(data)) => {
                let to_read = std::cmp::min(buf.remaining(), data.len());
                buf.put_slice(&data[..to_read]);
                if to_read < data.len() {
                    self.read_buffer.extend_from_slice(&data[to_read..]);
                }
                std::task::Poll::Ready(Ok(()))
            }
            std::task::Poll::Ready(None) => std::task::Poll::Ready(Ok(())),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl tokio::io::AsyncWrite for WispStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let mut packet = Vec::with_capacity(5 + buf.len());
        packet.push(WISP_TYPE_DATA);
        byteorder::WriteBytesExt::write_u32::<LittleEndian>(&mut packet, self.stream_id).unwrap();
        packet.extend_from_slice(buf);

        match self.ws_tx.try_send(Message::Binary(packet.into())) {
            Ok(_) => std::task::Poll::Ready(Ok(buf.len())),
            Err(mpsc::error::TrySendError::Full(_)) => std::task::Poll::Pending,
            Err(mpsc::error::TrySendError::Closed(_)) => {
                std::task::Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "WISP channel closed")))
            }
        }
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        let mut packet = Vec::with_capacity(6);
        packet.push(WISP_TYPE_CLOSE);
        byteorder::WriteBytesExt::write_u32::<LittleEndian>(&mut packet, self.stream_id).unwrap();
        packet.push(0x01); // Reason code 0x01 = Voluntary
        let _ = self.ws_tx.try_send(Message::Binary(packet.into()));
        std::task::Poll::Ready(Ok(()))
    }
}

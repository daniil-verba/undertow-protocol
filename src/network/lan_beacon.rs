//! ## LAN Beacon / LAN-маяк
//!
//! Обнаружение пиров в локальной сети через UDP multicast.
//! / Peer discovery in local network via UDP multicast.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

const MULTICAST_GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 0, 1);
const MULTICAST_PORT: u16 = 9003;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const PEER_TIMEOUT: Duration = Duration::from_secs(30);

/// LAN-сообщение для обнаружения пиров
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanMessage {
    pub peer_id: [u8; 32],
    pub username: String,
    pub port: u16,
    pub timestamp: u64,
    pub msg_type: LanMessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LanMessageType {
    Announce, // "Я здесь!"
    Goodbye,  // "Я ухожу"
    Ping,     // Проверка доступности
    Pong,     // Ответ на Ping
    Message,  // Сообщение чата
}

/// LAN-сообщение чата
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender_id: [u8; 32],
    pub sender_name: String,
    pub content: String,
    pub timestamp: u64,
}

/// Информация о пире в LAN
#[derive(Debug, Clone)]
pub struct LanPeer {
    pub peer_id: [u8; 32],
    pub username: String,
    pub addr: SocketAddr,
    pub last_seen: Instant,
    pub is_active: bool,
}

/// LAN Beacon - управление обнаружением пиров
pub struct LanBeacon {
    socket: Arc<UdpSocket>,
    peers: Arc<Mutex<HashMap<[u8; 32], LanPeer>>>,
    my_peer_id: [u8; 32],
    my_username: Arc<Mutex<String>>,
    my_port: u16,
    running: Arc<Mutex<bool>>,
}

impl LanBeacon {
    /// Создает новый LAN Beacon
    pub async fn new(
        peer_id: [u8; 32],
        username: String,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Привязываемся к порту для получения
        let recv_socket = UdpSocket::bind(format!("0.0.0.0:{}", MULTICAST_PORT)).await?;
        recv_socket.set_multicast_loop_v4(true)?;
        recv_socket.set_multicast_ttl_v4(1)?;

        // Присоединяемся к мультикаст-группе
        let interface = Ipv4Addr::new(0, 0, 0, 0);
        recv_socket.join_multicast_v4(MULTICAST_GROUP, interface)?;

        let socket = Arc::new(recv_socket);

        Ok(Self {
            socket,
            peers: Arc::new(Mutex::new(HashMap::new())),
            my_peer_id: peer_id,
            my_username: Arc::new(Mutex::new(username)),
            my_port: port,
            running: Arc::new(Mutex::new(true)),
        })
    }

    /// Запускает фоновые задачи: отправка heartbeat и прием сообщений
    pub async fn start(&self, tx: tokio::sync::mpsc::UnboundedSender<LanEvent>) {
        let running = self.running.clone();
        let peers = self.peers.clone();
        let socket = self.socket.clone();
        let my_peer_id = self.my_peer_id;
        let my_username = self.my_username.clone();
        let my_port = self.my_port;

        // Задача отправки heartbeat
        let heartbeat_tx = tx.clone();
        let heartbeat_peers = peers.clone();
        let heartbeat_socket = socket.clone();
        let heartbeat_id = my_peer_id;
        let heartbeat_name = my_username.clone();
        let heartbeat_port = my_port;
        let heartbeat_running = running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
            loop {
                interval.tick().await;

                // Проверяем, запущен ли beacon
                if !*heartbeat_running.lock().await {
                    break;
                }

                // Отправляем announce
                let username = heartbeat_name.lock().await.clone();
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let msg = LanMessage {
                    peer_id: heartbeat_id,
                    username: username.clone(),
                    port: heartbeat_port,
                    timestamp,
                    msg_type: LanMessageType::Announce,
                };

                let data = match serde_json::to_vec(&msg) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Failed to serialize announce: {}", e);
                        continue;
                    }
                };

                // Отправляем в multicast группу
                let addr = SocketAddr::from((MULTICAST_GROUP, MULTICAST_PORT));
                if let Err(e) = heartbeat_socket.send_to(&data, addr).await {
                    eprintln!("Failed to send announce: {}", e);
                }

                // Очищаем старые пиры
                let mut peers_guard = heartbeat_peers.lock().await;
                let now = Instant::now();
                let expired_peers: Vec<([u8; 32], String)> = peers_guard
                    .iter()
                    .filter(|(_, peer)| {
                        now.duration_since(peer.last_seen) > PEER_TIMEOUT && peer.is_active
                    })
                    .map(|(id, peer)| (*id, peer.username.clone()))
                    .collect();

                for (id, username) in expired_peers {
                    if let Some(peer) = peers_guard.remove(&id) {
                        let _ = heartbeat_tx.send(LanEvent::PeerLeft {
                            peer_id: id,
                            username: peer.username,
                        });
                    }
                }
            }
        });

        // Задача приема сообщений
        let recv_socket = self.socket.clone();
        let recv_peers = self.peers.clone();
        let recv_tx = tx.clone();
        let recv_running = running.clone();
        let recv_my_id = my_peer_id;
        let recv_my_name = my_username.clone();
        let recv_my_port = my_port;

        tokio::spawn(async move {
            let mut buf = [0u8; 8192];
            loop {
                // Проверяем, запущен ли beacon
                if !*recv_running.lock().await {
                    break;
                }

                match recv_socket.recv_from(&mut buf).await {
                    Ok((size, src_addr)) => {
                        // Игнорируем свои же сообщения
                        let data = &buf[..size];
                        match serde_json::from_slice::<LanMessage>(data) {
                            Ok(msg) => {
                                // Игнорируем свои сообщения
                                if msg.peer_id == recv_my_id {
                                    continue;
                                }

                                match msg.msg_type {
                                    LanMessageType::Announce => {
                                        let now = Instant::now();
                                        let mut peers_guard = recv_peers.lock().await;

                                        // Проверяем, не появился ли новый пир
                                        let is_new = !peers_guard.contains_key(&msg.peer_id);

                                        let peer = LanPeer {
                                            peer_id: msg.peer_id,
                                            username: msg.username.clone(),
                                            addr: SocketAddr::from((src_addr.ip(), msg.port)),
                                            last_seen: now,
                                            is_active: true,
                                        };

                                        peers_guard.insert(msg.peer_id, peer);

                                        if is_new {
                                            // Новый пир обнаружен
                                            let _ = recv_tx.send(LanEvent::PeerJoined {
                                                peer_id: msg.peer_id,
                                                username: msg.username,
                                                addr: src_addr,
                                            });
                                        }
                                    }
                                    LanMessageType::Goodbye => {
                                        let mut peers_guard = recv_peers.lock().await;
                                        if let Some(peer) = peers_guard.remove(&msg.peer_id) {
                                            let _ = recv_tx.send(LanEvent::PeerLeft {
                                                peer_id: msg.peer_id,
                                                username: peer.username,
                                            });
                                        }
                                    }
                                    LanMessageType::Ping => {
                                        // Отвечаем Pong
                                        let timestamp = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs();

                                        let pong = LanMessage {
                                            peer_id: recv_my_id,
                                            username: recv_my_name.lock().await.clone(),
                                            port: recv_my_port,
                                            timestamp,
                                            msg_type: LanMessageType::Pong,
                                        };
                                        if let Ok(data) = serde_json::to_vec(&pong) {
                                            let _ = recv_socket.send_to(&data, src_addr).await;
                                        }
                                    }
                                    LanMessageType::Pong => {
                                        // Обновляем время последнего контакта
                                        let mut peers_guard = recv_peers.lock().await;
                                        if let Some(peer) = peers_guard.get_mut(&msg.peer_id) {
                                            peer.last_seen = Instant::now();
                                            peer.is_active = true;
                                        }
                                    }
                                    LanMessageType::Message => {
                                        // Получено сообщение чата
                                        // Пытаемся десериализовать ChatMessage из оставшейся части данных
                                        if size > data.len() {
                                            // Если данные закончились, пытаемся десериализовать все как ChatMessage
                                            if let Ok(chat_msg) =
                                                serde_json::from_slice::<ChatMessage>(data)
                                            {
                                                let _ = recv_tx.send(LanEvent::ChatMessage {
                                                    sender_id: msg.peer_id,
                                                    sender_name: msg.username,
                                                    content: chat_msg.content,
                                                });
                                            } else {
                                                eprintln!("Failed to parse chat message");
                                            }
                                        } else {
                                            // Пробуем найти ChatMessage в конце данных
                                            // Ищем JSON объект ChatMessage
                                            let data_str = String::from_utf8_lossy(data);
                                            if let Some(start) = data_str.rfind('{') {
                                                if let Ok(chat_msg) =
                                                    serde_json::from_str::<ChatMessage>(
                                                        &data_str[start..],
                                                    )
                                                {
                                                    let _ = recv_tx.send(LanEvent::ChatMessage {
                                                        sender_id: msg.peer_id,
                                                        sender_name: msg.username,
                                                        content: chat_msg.content,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to deserialize LAN message: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving LAN message: {}", e);
                    }
                }
            }
        });
    }

    /// Отправляет сообщение в LAN
    pub async fn send_chat_message(
        &self,
        target_peer_id: [u8; 32],
        content: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let peers_guard = self.peers.lock().await;
        if let Some(peer) = peers_guard.get(&target_peer_id) {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Создаем сообщение чата
            let chat_msg = ChatMessage {
                sender_id: self.my_peer_id,
                sender_name: self.my_username.lock().await.clone(),
                content: content.clone(),
                timestamp,
            };

            // Отправляем как обычное LAN сообщение с типом Message
            let msg = LanMessage {
                peer_id: self.my_peer_id,
                username: self.my_username.lock().await.clone(),
                port: self.my_port,
                timestamp,
                msg_type: LanMessageType::Message,
            };

            // Сериализуем оба сообщения: сначала LanMessage, потом ChatMessage
            let mut data = serde_json::to_vec(&msg)?;
            let chat_data = serde_json::to_vec(&chat_msg)?;
            data.extend_from_slice(&chat_data);

            // Отправляем напрямую пиру
            let addr = peer.addr;
            drop(peers_guard); // Освобождаем блокировку перед отправкой

            self.socket.send_to(&data, addr).await?;
            Ok(())
        } else {
            Err(format!("Peer not found in LAN: {:?}", target_peer_id).into())
        }
    }

    /// Отправляет широковещательное сообщение всем пирам в LAN
    pub async fn broadcast_chat_message(
        &self,
        content: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let peers = self.peers.lock().await;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let msg = LanMessage {
            peer_id: self.my_peer_id,
            username: self.my_username.lock().await.clone(),
            port: self.my_port,
            timestamp,
            msg_type: LanMessageType::Message,
        };

        let chat_msg = ChatMessage {
            sender_id: self.my_peer_id,
            sender_name: self.my_username.lock().await.clone(),
            content,
            timestamp,
        };

        let mut data = serde_json::to_vec(&msg)?;
        let chat_data = serde_json::to_vec(&chat_msg)?;
        data.extend_from_slice(&chat_data);

        for (_, peer) in peers.iter() {
            if peer.is_active {
                let _ = self.socket.send_to(&data, peer.addr).await;
            }
        }

        Ok(())
    }

    /// Обновляет имя пользователя
    pub async fn update_username(&self, new_username: String) {
        let mut username_guard = self.my_username.lock().await;
        *username_guard = new_username;
    }

    /// Получает список активных пиров
    pub async fn get_active_peers(&self) -> Vec<LanPeer> {
        let peers_guard = self.peers.lock().await;
        peers_guard
            .values()
            .filter(|p| p.is_active)
            .cloned()
            .collect()
    }

    /// Останавливает beacon
    pub async fn stop(&self) {
        // Отправляем goodbye
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let msg = LanMessage {
            peer_id: self.my_peer_id,
            username: self.my_username.lock().await.clone(),
            port: self.my_port,
            timestamp,
            msg_type: LanMessageType::Goodbye,
        };

        if let Ok(data) = serde_json::to_vec(&msg) {
            let addr = SocketAddr::from((MULTICAST_GROUP, MULTICAST_PORT));
            let _ = self.socket.send_to(&data, addr).await;
        }

        let mut running_guard = self.running.lock().await;
        *running_guard = false;
    }
}

/// События LAN Beacon
#[derive(Debug, Clone)]
pub enum LanEvent {
    PeerJoined {
        peer_id: [u8; 32],
        username: String,
        addr: SocketAddr,
    },
    PeerLeft {
        peer_id: [u8; 32],
        username: String,
    },
    ChatMessage {
        sender_id: [u8; 32],
        sender_name: String,
        content: String,
    },
}

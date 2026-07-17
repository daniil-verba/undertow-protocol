//! ## LAN Discovery / Локальное обнаружение
//!
//! Discovers peers in the local network via UDP multicast.
//! / Обнаруживает пиров в локальной сети через UDP multicast.
use crate::protocol::peer_id::PeerId;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket as StdUdpSocket};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

/// LAN discovery message format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanDiscoveryMessage {
    pub peer_id: String,
    pub username: String,
    pub port: u16,
    pub msg_type: LanMessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LanMessageType {
    /// I'm here! / Я здесь!
    Announce,
    /// I'm leaving / Я ухожу
    Leave,
}

/// LAN Discovery client.
pub struct LanDiscovery {
    socket: Arc<UdpSocket>,
    multicast_addr: SocketAddr,
    peer_id: PeerId,
    username: String,
    port: u16,
}

impl LanDiscovery {
    /// Creates a new LAN discovery instance.
    pub async fn new(peer_id: PeerId, username: String, port: u16) -> Result<Self, String> {
        // 1. Используем Multicast-адрес (стандарт для локальных сетей, как SSDP).
        // Он гораздо надежнее, чем глобальный broadcast 255.255.255.255, который часто блокируется ОС.
        let multicast_addr: SocketAddr =
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(239, 255, 255, 250), 9999));

        // 2. Создаем стандартный сокет, чтобы включить SO_REUSEADDR.
        // Это КРИТИЧЕСКИ важно для тестирования: позволяет запустить несколько узлов на одном ПК.
        let std_socket = StdUdpSocket::bind("0.0.0.0:9999")
            .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;

        std_socket
            .set_reuse_address(true)
            .map_err(|e| format!("Failed to set reuse address: {}", e))?;

        // 3. Присоединяемся к multicast-группе (требуется в Windows/macOS для получения пакетов)
        let local_addr = std_socket.local_addr().map_err(|e| e.to_string())?;
        if let SocketAddr::V4(local_ipv4) = local_addr {
            // Игнорируем ошибку, так как на некоторых виртуальных интерфейсах это может fail,
            // но на основном сетевом интерфейсе сработает корректно.
            let _ =
                std_socket.join_multicast_v4(&Ipv4Addr::new(239, 255, 255, 250), &local_ipv4.ip());
        }

        std_socket
            .set_nonblocking(true)
            .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

        // 4. Конвертируем в асинхронный сокет tokio
        let socket = UdpSocket::from_std(std_socket)
            .map_err(|e| format!("Failed to convert to tokio socket: {}", e))?;

        Ok(Self {
            socket: Arc::new(socket),
            multicast_addr,
            peer_id,
            username,
            port,
        })
    }

    /// Sends an announcement to the local network.
    pub async fn announce(&self) -> Result<(), String> {
        let msg = LanDiscoveryMessage {
            peer_id: self.peer_id.to_string(),
            username: self.username.clone(),
            port: self.port,
            msg_type: LanMessageType::Announce,
        };
        let data = serde_json::to_vec(&msg).map_err(|e| format!("Failed to serialize: {}", e))?;

        self.socket
            .send_to(&data, self.multicast_addr)
            .await
            .map_err(|e| format!("Failed to send multicast: {}", e))?;
        Ok(())
    }

    /// Sends a leave notification.
    pub async fn leave(&self) -> Result<(), String> {
        let msg = LanDiscoveryMessage {
            peer_id: self.peer_id.to_string(),
            username: self.username.clone(),
            port: self.port,
            msg_type: LanMessageType::Leave,
        };
        let data = serde_json::to_vec(&msg).map_err(|e| format!("Failed to serialize: {}", e))?;

        self.socket
            .send_to(&data, self.multicast_addr)
            .await
            .map_err(|e| format!("Failed to send leave: {}", e))?;
        Ok(())
    }

    /// Listens for LAN discovery messages.
    pub async fn listen<F>(&self, mut callback: F) -> Result<(), String>
    where
        F: FnMut(LanDiscoveryMessage, SocketAddr) + Send + 'static,
    {
        // МЫ ИСПОЛЬЗУЕМ УЖЕ СОЗДАННЫЙ СОКЕТ из new(), а не создаем новый!
        // Это предотвращает ошибку "Address already in use" при запуске второго узла.
        let socket = Arc::clone(&self.socket);
        let my_peer_id_str = self.peer_id.to_string();

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match timeout(Duration::from_secs(60), socket.recv_from(&mut buf)).await {
                    Ok(Ok((size, addr))) => {
                        let data = &buf[..size];
                        match serde_json::from_slice::<LanDiscoveryMessage>(data) {
                            Ok(msg) => {
                                // Игнорируем собственные сообщения
                                if msg.peer_id == my_peer_id_str {
                                    continue;
                                }
                                callback(msg, addr);
                            }
                            Err(e) => {
                                eprintln!("[LAN] Failed to parse message: {}", e);
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        eprintln!("[LAN] Recv error: {}", e);
                    }
                    Err(_) => {
                        // Timeout - продолжаем цикл
                    }
                }
            }
        });
        Ok(())
    }
}

impl Clone for LanDiscovery {
    fn clone(&self) -> Self {
        Self {
            socket: Arc::clone(&self.socket),
            multicast_addr: self.multicast_addr,
            peer_id: self.peer_id.clone(),
            username: self.username.clone(),
            port: self.port,
        }
    }
}

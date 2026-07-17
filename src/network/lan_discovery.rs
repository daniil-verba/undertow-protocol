//! ## LAN Discovery / Локальное обнаружение
//!
//! Discovers peers in the local network via UDP broadcast.
//! / Обнаруживает пиров в локальной сети через UDP broadcast.

use crate::protocol::peer_id::PeerId;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
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
    broadcast_addr: SocketAddr,
    peer_id: PeerId,
    username: String,
    port: u16,
}

impl LanDiscovery {
    /// Creates a new LAN discovery instance.
    pub async fn new(peer_id: PeerId, username: String, port: u16) -> Result<Self, String> {
        // Bind to a random port for receiving
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;

        // Enable broadcast
        socket
            .set_broadcast(true)
            .map_err(|e| format!("Failed to enable broadcast: {}", e))?;

        // Use standard broadcast address
        let broadcast_addr = "255.255.255.255:9999"
            .parse()
            .map_err(|e| format!("Invalid broadcast address: {}", e))?;

        Ok(Self {
            socket: Arc::new(socket),
            broadcast_addr,
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
            .send_to(&data, self.broadcast_addr)
            .await
            .map_err(|e| format!("Failed to send broadcast: {}", e))?;

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
            .send_to(&data, self.broadcast_addr)
            .await
            .map_err(|e| format!("Failed to send leave: {}", e))?;

        Ok(())
    }

    /// Listens for LAN discovery messages.
    pub async fn listen<F>(&self, mut callback: F) -> Result<(), String>
    where
        F: FnMut(LanDiscoveryMessage, SocketAddr) + Send + 'static,
    {
        // Create a new socket for listening (bind to same port)
        let listen_socket = UdpSocket::bind("0.0.0.0:9999")
            .await
            .map_err(|e| format!("Failed to bind listen socket: {}", e))?;

        listen_socket
            .set_broadcast(true)
            .map_err(|e| format!("Failed to enable broadcast on listen socket: {}", e))?;

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match timeout(Duration::from_secs(60), listen_socket.recv_from(&mut buf)).await {
                    Ok(Ok((size, addr))) => {
                        let data = &buf[..size];
                        match serde_json::from_slice::<LanDiscoveryMessage>(data) {
                            Ok(msg) => {
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
                        // Timeout - continue
                    }
                }
            }
        });

        Ok(())
    }

    /// Sends an announce and starts periodic announcements.
    pub async fn start_announce_loop(&self) -> Result<(), String> {
        let socket = Arc::clone(&self.socket);
        let peer_id = self.peer_id.clone();
        let username = self.username.clone();
        let port = self.port;
        let broadcast_addr = self.broadcast_addr;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;

                let msg = LanDiscoveryMessage {
                    peer_id: peer_id.to_string(),
                    username: username.clone(),
                    port,
                    msg_type: LanMessageType::Announce,
                };

                let data = match serde_json::to_vec(&msg) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[LAN] Failed to serialize announce: {}", e);
                        continue;
                    }
                };

                if let Err(e) = socket.send_to(&data, broadcast_addr).await {
                    eprintln!("[LAN] Failed to send announce: {}", e);
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
            broadcast_addr: self.broadcast_addr,
            peer_id: self.peer_id.clone(),
            username: self.username.clone(),
            port: self.port,
        }
    }
}

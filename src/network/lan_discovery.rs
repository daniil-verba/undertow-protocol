//! ## LAN Discovery / Локальное обнаружение
//!
//! Discovers peers in the local network via UDP broadcast.
//! / Обнаруживает пиров в локальной сети через UDP broadcast.

use crate::protocol::peer_id::PeerId;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
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
    socket: UdpSocket,
    broadcast_addr: SocketAddr,
    peer_id: PeerId,
    username: String,
    port: u16,
}

impl LanDiscovery {
    /// Creates a new LAN discovery instance.
    pub fn new(peer_id: PeerId, username: String, port: u16) -> Result<Self, String> {
        // Bind to a random port for receiving
        let socket = UdpSocket::bind("0.0.0.0:0")
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
            socket,
            broadcast_addr,
            peer_id,
            username,
            port,
        })
    }

    /// Sends an announcement to the local network.
    pub fn announce(&self) -> Result<(), String> {
        let msg = LanDiscoveryMessage {
            peer_id: self.peer_id.to_string(),
            username: self.username.clone(),
            port: self.port,
            msg_type: LanMessageType::Announce,
        };

        let data = serde_json::to_vec(&msg).map_err(|e| format!("Failed to serialize: {}", e))?;

        self.socket
            .send_to(&data, self.broadcast_addr)
            .map_err(|e| format!("Failed to send broadcast: {}", e))?;

        Ok(())
    }

    /// Sends a leave notification.
    pub fn leave(&self) -> Result<(), String> {
        let msg = LanDiscoveryMessage {
            peer_id: self.peer_id.to_string(),
            username: self.username.clone(),
            port: self.port,
            msg_type: LanMessageType::Leave,
        };

        let data = serde_json::to_vec(&msg).map_err(|e| format!("Failed to serialize: {}", e))?;

        self.socket
            .send_to(&data, self.broadcast_addr)
            .map_err(|e| format!("Failed to send leave: {}", e))?;

        Ok(())
    }

    /// Listens for LAN discovery messages.
    pub async fn listen<F>(&self, mut callback: F) -> Result<(), String>
    where
        F: FnMut(LanDiscoveryMessage, SocketAddr) + Send + 'static,
    {
        let socket = self
            .socket
            .try_clone()
            .map_err(|e| format!("Failed to clone socket: {}", e))?;

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match timeout(Duration::from_secs(60), socket.recv_from(&mut buf)).await {
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
}

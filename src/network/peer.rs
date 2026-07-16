//! ## Peer Module / Модуль пира
//!
//! Represents a remote node in the network with addresses and metadata.
//! / Представляет удалённый узел в сети с адресами и метаданными.

use crate::protocol::peer_id::PeerId;
use std::net::SocketAddr;

/// Remote peer in the Undertow network.
/// / Удалённый пир в сети Undertow.
pub struct Peer {
    id: PeerId,
    addrs: Vec<SocketAddr>,
    last_seen: u64, // Unix timestamp / Unix timestamp
    is_relay: bool,
    reputation: u32,
}

impl Peer {
    /// Creates a new peer with initial address.
    /// / Создаёт нового пира с начальным адресом.
    pub fn new(id: PeerId, addr: SocketAddr) -> Self {
        Self {
            id,
            addrs: vec![addr],
            last_seen: 0,
            is_relay: false,
            reputation: 0,
        }
    }

    /// Adds an address if not already present.
    /// / Добавляет адрес, если его ещё нет.
    pub fn add_addr(&mut self, addr: SocketAddr) {
        if !self.addrs.contains(&addr) {
            self.addrs.push(addr);
        }
    }

    /// Returns primary endpoint (first address).
    /// / Возвращает основной endpoint (первый адрес).
    pub fn endpoint(&self) -> String {
        self.addrs
            .first()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Checks if peer is stale (not seen for timeout_sec).
    /// / Проверяет, устарел ли пир (не появлялся timeout_sec секунд).
    pub fn is_stale(&self, now: u64, timeout_sec: u64) -> bool {
        now.saturating_sub(self.last_seen) > timeout_sec
    }

    /// Updates last_seen timestamp.
    /// / Обновляет метку времени last_seen.
    pub fn update_seen(&mut self, timestamp: u64) {
        self.last_seen = timestamp;
    }

    pub fn id(&self) -> &PeerId {
        &self.id
    }

    pub fn is_relay(&self) -> bool {
        self.is_relay
    }

    pub fn set_relay(&mut self, relay: bool) {
        self.is_relay = relay;
    }

    // ==================== SERIALIZATION / СЕРИАЛИЗАЦИЯ ====================

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // ID: 32 bytes / 32 байта
        buf.extend_from_slice(self.id.as_bytes());

        // Address count: 1 byte / Количество адресов: 1 байт
        let addr_count = self.addrs.len() as u8;
        buf.push(addr_count);

        // Each address: [ip_type: 1][ip_bytes: 4 or 16][port: 2]
        // Каждый адрес: [тип_ip: 1][байты_ip: 4 или 16][порт: 2]
        for addr in &self.addrs {
            match addr {
                SocketAddr::V4(v4) => {
                    buf.push(4);
                    buf.extend_from_slice(&v4.ip().octets());
                }
                SocketAddr::V6(v6) => {
                    buf.push(6);
                    buf.extend_from_slice(&v6.ip().octets());
                }
            }
            buf.extend_from_slice(&addr.port().to_be_bytes());
        }

        // last_seen: 8 bytes / 8 байт
        buf.extend_from_slice(&self.last_seen.to_be_bytes());

        // is_relay: 1 byte / 1 байт
        buf.push(self.is_relay as u8);

        // reputation: 4 bytes / 4 байта
        buf.extend_from_slice(&self.reputation.to_be_bytes());

        buf
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        let mut pos;

        // ID: 32 bytes / 32 байта
        if data.len() < 32 {
            return Err("Data too short for ID / Данные слишком короткие для ID".to_string());
        }
        let mut id_bytes = [0u8; 32];
        id_bytes.copy_from_slice(&data[0..32]);
        let id = PeerId::new(id_bytes);
        pos = 32;

        // Address count / Количество адресов
        if data.len() < pos + 1 {
            return Err(
                "Data too short for addr count / Данные слишком короткие для количества адресов"
                    .to_string(),
            );
        }
        let addr_count = data[pos] as usize;
        pos += 1;

        // Addresses / Адреса
        let mut addrs = Vec::with_capacity(addr_count);
        for _ in 0..addr_count {
            if data.len() < pos + 1 {
                return Err(
                    "Data too short for addr type / Данные слишком короткие для типа адреса"
                        .to_string(),
                );
            }
            let ip_type = data[pos];
            pos += 1;

            let addr = match ip_type {
                4 => {
                    if data.len() < pos + 4 + 2 {
                        return Err("Data too short for IPv4 / Данные слишком короткие для IPv4"
                            .to_string());
                    }
                    let ip = std::net::Ipv4Addr::new(
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    );
                    pos += 4;
                    let port = u16::from_be_bytes([data[pos], data[pos + 1]]);
                    pos += 2;
                    SocketAddr::from((ip, port))
                }
                6 => {
                    if data.len() < pos + 16 + 2 {
                        return Err("Data too short for IPv6 / Данные слишком короткие для IPv6"
                            .to_string());
                    }
                    let mut ip_bytes = [0u8; 16];
                    ip_bytes.copy_from_slice(&data[pos..pos + 16]);
                    let ip = std::net::Ipv6Addr::from(ip_bytes);
                    pos += 16;
                    let port = u16::from_be_bytes([data[pos], data[pos + 1]]);
                    pos += 2;
                    SocketAddr::from((ip, port))
                }
                _ => {
                    return Err(format!(
                        "Unknown IP type: {} / Неизвестный тип IP: {}",
                        ip_type, ip_type
                    ))
                }
            };
            addrs.push(addr);
        }

        // last_seen / Последнее появление
        if data.len() < pos + 8 {
            return Err(
                "Data too short for timestamp / Данные слишком короткие для timestamp".to_string(),
            );
        }
        let last_seen = u64::from_be_bytes([
            data[pos],
            data[pos + 1],
            data[pos + 2],
            data[pos + 3],
            data[pos + 4],
            data[pos + 5],
            data[pos + 6],
            data[pos + 7],
        ]);
        pos += 8;

        // is_relay / Флаг ретрансляции
        if data.len() < pos + 1 {
            return Err(
                "Data too short for relay flag / Данные слишком короткие для флага ретрансляции"
                    .to_string(),
            );
        }
        let is_relay = data[pos] != 0;
        pos += 1;

        // reputation / Репутация
        if data.len() < pos + 4 {
            return Err(
                "Data too short for reputation / Данные слишком короткие для репутации".to_string(),
            );
        }
        let reputation =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);

        Ok(Self {
            id,
            addrs,
            last_seen,
            is_relay,
            reputation,
        })
    }
}

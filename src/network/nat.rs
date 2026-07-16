//! ## NAT Detection / Определение NAT
//!
//! Detects NAT type and external IP via STUN.
//! / Определяет тип NAT и внешний IP через STUN.

use crate::network::local::LocalDiscovery;
use crate::network::stun::StunClient;
use std::net::SocketAddr;

/// NAT information for this node.
/// / Информация о NAT для этого узла.
#[derive(Debug, Clone)]
pub struct NatInfo {
    pub nat_type: NatType,
    pub external_addr: Option<SocketAddr>,
    pub local_addrs: Vec<SocketAddr>,
}

/// Types of NAT / Типы NAT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    Direct,     // Public IP / Белый IP
    Cone,       // Full Cone NAT
    Restricted, // Restricted Cone NAT
    Symmetric,  // Symmetric NAT — requires relay / Требует ретрансляции
    Unknown,
}

pub struct NatDetector;

impl NatDetector {
    /// Detects NAT type and returns NatInfo.
    /// / Определяет тип NAT и возвращает NatInfo.
    pub async fn detect(local_port: u16) -> NatInfo {
        let local_addrs = LocalDiscovery::get_local_addrs(local_port);
        println!("📍 Local addresses / Локальные адреса: {:?}", local_addrs);

        // Try STUN / Пробуем STUN
        match StunClient::discover().await {
            Ok(external) => {
                let nat_type = Self::classify_nat(&external, local_port);
                println!("🌐 NAT type / Тип NAT: {:?}", nat_type);
                println!("🌍 External address / Внешний адрес: {}", external);

                NatInfo {
                    nat_type,
                    external_addr: Some(external),
                    local_addrs,
                }
            }
            Err(e) => {
                println!("⚠️ STUN failed / STUN не удался: {}", e);
                NatInfo {
                    nat_type: NatType::Unknown,
                    external_addr: None,
                    local_addrs,
                }
            }
        }
    }

    /// Simple NAT classification heuristic.
    /// / Простая эвристика классификации NAT.
    fn classify_nat(external: &SocketAddr, local_port: u16) -> NatType {
        // If external port == local port → likely Direct or Cone
        // Если внешний порт == локальный порт → скорее всего Direct или Cone
        if external.port() == local_port {
            NatType::Cone
        } else {
            // Port changed → likely Symmetric / Порт изменился → скорее всего Symmetric
            NatType::Symmetric
        }
    }
}

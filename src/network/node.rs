//! ## Node Module / Модуль узла
//!
//! Local P2P node with DHT routing and NAT detection.
//! / Локальный P2P-узел с маршрутизацией DHT и определением NAT.

use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::network::dht::RoutingTable;
use crate::network::nat::{NatDetector, NatType};
use crate::protocol::peer_id::PeerId;

/// Local P2P node.
/// / Локальный P2P-узел.
pub struct Node {
    local_id: PeerId,
    local_port: u16,
    listener: TcpListener,
    routing_table: Arc<Mutex<RoutingTable>>,
}

impl Node {
    /// Binds a new node to the specified address.
    /// / Привязывает новый узел к указанному адресу.
    pub async fn bind(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        let local_port = listener.local_addr()?.port();
        let local_id = PeerId::random();
        let routing_table = Arc::new(Mutex::new(RoutingTable::new(local_id.clone())));

        Ok(Self {
            local_id,
            local_port,
            listener,
            routing_table,
        })
    }

    /// Creates a node with a given PeerId and port.
    pub async fn bind_with_peer_id(
        addr: &str,
        peer_id: PeerId,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        let local_port = listener.local_addr()?.port();
        let routing_table = Arc::new(Mutex::new(RoutingTable::new(peer_id.clone())));

        Ok(Self {
            local_id: peer_id,
            local_port,
            listener,
            routing_table,
        })
    }

    pub fn local_id(&self) -> &PeerId {
        &self.local_id
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    /// Runs the node accept loop.
    /// / Запускает цикл приёма соединений узла.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "🚢 Node {} listening on / Узел {} слушает на {}",
            self.local_id.short(),
            self.local_id.short(),
            self.listener.local_addr()?
        );

        loop {
            let (_stream, addr) = self.listener.accept().await?;
            println!("📡 Incoming from / Входящий от {}", addr);

            tokio::spawn(async move {
                // TODO: handle incoming connections
                // TODO: обработка входящих соединений
            });
        }
    }

    /// Determines connection strategy based on NAT type.
    /// / Определяет стратегию соединения на основе типа NAT.
    pub async fn connect_with_nat(&self, _target: &PeerId, beacon_addr: Option<&str>) {
        let nat = NatDetector::detect(self.local_port).await;

        match nat.nat_type {
            NatType::Direct => {
                println!("✅ Direct IP — trying direct / Прямой IP — пробуем напрямую");
            }
            NatType::Cone | NatType::Restricted => {
                println!("🔄 Cone NAT — trying hole punching / Cone NAT — пробуем hole punching");
            }
            NatType::Symmetric | NatType::Unknown => {
                println!("🔒 Symmetric NAT — using Beacon / Symmetric NAT — используем маяк");
                if let Some(beacon) = beacon_addr {
                    println!("🗼 Beacon: {}", beacon);
                } else {
                    println!("⚠️ No beacon! / Нет маяка!");
                }
            }
        }
    }
}

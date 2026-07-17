//! ## Network Module / Модуль сети
//!
//! Сетевая подсистема Undertow: обнаружение пиров, NAT, DHT и P2P соединения.
//! / Undertow network subsystem: peer discovery, NAT, DHT and P2P connections.

pub mod beacon_client;
pub mod dht;
pub mod hole_puncher;
pub mod kbucket;
pub mod lan;
pub mod local;
pub mod nat;
pub mod node;
pub mod peer;
pub mod relay;
pub mod stun;

pub use lan::{LanBeacon, LanEvent, LanPeer};

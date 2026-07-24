//! ## Network Module / Модуль сети
//!
//! Сетевая подсистема Undertow, построенная на базе Iroh.
//! Iroh берет на себя NAT traversal, STUN, relay и шифрование,
//! позволяя нам сосредоточиться на логике протокола (DHT, сообщения).

pub mod beacon_client;
pub mod transport;
// pub mod dht;      // Оставлен для будущей интеграции поверх IrohTransport
// pub mod kbucket;  // Оставлен для будущей интеграции

pub use beacon_client::BeaconClient;
pub use transport::{IrohTransport, TransportError};

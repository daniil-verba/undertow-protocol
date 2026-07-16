//! ## Undertow Protocol Core Library
//!
//! Decentralized P2P network protocol with federated platform.
//!
//! ## Features:
//! - `protocol`: Packet serialization/deserialization, PeerId (always enabled)
//! - `crypto`: Cryptographic primitives (X25519, SHA2) – always enabled
//! - `network`: P2P networking, NAT detection, local discovery
//! - `beacon`: Beacon client and server (relay, rendezvous, credits)
//! - `storage`: Persistent profile storage (optional, for clients)
//! - `dht`: Kademlia DHT routing (optional, requires network)
//!
//! ## Note:
//! - The `ui` module has been moved to the `undertow-client` crate.
//!   It is no longer part of the protocol library.

// ─── Core modules (always available) ────────────────────────
// pub mod crypto;
pub mod protocol;

// ─── Network layer (requires `network` feature) ────────────
#[cfg(feature = "network")]
pub mod network;

// ─── Beacon (requires `network` feature) ──────────────────
// Contains beacon server, client, and credits.
// #[cfg(feature = "network")]
// pub mod beacon;

// ─── DHT (optional, requires `network`) ────────────────────
// #[cfg(feature = "network")]
// pub mod dht;

// ─── Storage (optional, for persistent profiles) ───────────
#[cfg(feature = "storage")]
pub mod storage;

// ─── Re-exports for convenience ─────────────────────────────
#[cfg(feature = "network")]
pub use network::*;

#[cfg(feature = "storage")]
pub use storage::*;

// ─── Version info ────────────────────────────────────────────
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

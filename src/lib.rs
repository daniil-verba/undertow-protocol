//! ## Undertow Protocol Core Library
//!
//! Decentralized P2P network protocol with federated platform.
//!
//! ## Features:
//! -  `protocol` : Packet serialization/deserialization, PeerId (always enabled)
//! -  `crypto` : Cryptographic primitives (X25519, SHA2) – always enabled
//! -  `network` : P2P networking via Iroh (NAT traversal, relay, encryption)
//! -  `beacon` : Beacon client (rendezvous, credits) over Iroh
//! -  `storage` : Persistent profile storage (optional, for clients)
//!
//! ## Note:
//! - The  `ui`  module has been moved to the  `undertow-client`  crate.

// ─── Core modules (always available) ────────────────────────
// pub mod crypto;
pub mod protocol;

// ─── Network layer (requires `network` feature) ────────────
#[cfg(feature = "network")]
pub mod network;

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

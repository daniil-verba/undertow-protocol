<div align="center">

<!-- Replace with your logo:-->
<img src="assets/sea.png" width="120" alt="Undertow">

# 🌊 Undertow Protocol

*The current that cannot be stopped.*

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Prototype-yellow)](https://github.com/daniil-verba/undertow-protocol)
<!-- [![Discord](https://img.shields.io/badge/Discord-Join-5865F2?logo=discord&logoColor=white)](https://discord.gg/rqJJf9WcV6) -->

**Current version: v0.1.0** | [Changelog](CHANGELOG.md)

[Русский](README.ru.md) | **English**

</div>

---

> P2P protocol for decentralized applications. One network. Zero configuration. NAT traversal out of the box.

## What is Undertow

**Undertow** is a protocol for building peer-to-peer networks focused on messaging. The core idea: **one network for all applications**. An indie developer plugs in a ready-made library — and gets access to a global network with DHT, NAT traversal, and relay beacons. No need to deploy your own infrastructure.

The network works like an ocean current: users are ships with floating IPs, **Beacon** servers are lighthouses for navigation, **Harbor** nodes are ports for stable network entry.

## Status

🚧 **Early prototype** — active development, API changes daily. MVP goal: messaging over LAN + NAT traversal.

## Why Undertow over libp2p / WebRTC / DIY

| | Undertow | libp2p | WebRTC | DIY |
|---|---|---|---|---|
| **Ready-made network** | ✅ Unified DHT + beacons | ❌ Grow your own | ❌ Transport only | ❌ Everything from scratch |
| **NAT traversal** | ✅ Out of the box | ⚠️ Complex | ✅ But needs TURN | ❌ Build yourself |
| **Indie-friendly** | ✅ Single dependency | ⚠️ 20+ crates | ⚠️ C++ / complex | ❌ Months of work |
| **Cross-platform** | ✅ Rust | ✅ | ✅ Browser only | ? |
| **Economy** | ✅ Built-in | ❌ None | ❌ None | ❌ None |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    UNIFIED UNDERTOW NETWORK                  │
│                                                              │
│   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐ │
│   │ Game A  │◄──►│ Client  │◄──►│ Discord │◄──►│Telegram │ │
│   │ (lib)   │    │ (TUI)   │    │  bot    │    │  bot    │ │
│   └────┬────┘    └────┬────┘    └────┬────┘    └────┬────┘ │
│        │              │              │              │        │
│   ┌────┴──────────────┴──────────────┴──────────────┴────┐ │
│   │              undertow-protocol (lib)                   │ │
│   │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │ │
│   │  │ Network │  │   DHT   │  │  Crypto │  │ Protocol│  │ │
│   │  │(P2P,   │  │(Kademlia│  │(X25519, │  │(packets,│  │ │
│   │  │ NAT,   │  │ custom) │  │SHA-256) │  │ peer_id)│  │ │
│   │  │ relay) │  │         │  │         │  │         │  │ │
│   │  └─────────┘  └─────────┘  └─────────┘  └─────────┘  │ │
│   └─────────────────────────────────────────────────────────┘ │
│        │              │              │                        │
│   ┌────┴────┐    ┌───┴────┐    ┌───┴────┐                   │
│   │ Harbor  │    │ Beacon │    │ Beacon │  ← public VPS     │
│   │(bootstrap│   │(relay /│    │(relay /│    with white IP   │
│   │  node)  │    │rendezvous│   │rendezvous│                   │
│   └─────────┘    └────────┘    └────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

## Modules

| Module | Status | Description |
|--------|--------|-------------|
| `network` | 🚧 WIP | P2P connections, NAT traversal, hole punching, relay |
| `protocol` | 🚧 WIP | Packet format, `PeerId` (SHA-256 of public key), serialization (bincode) |
| `beacon` | 🚧 WIP | Client for Beacon servers (rendezvous + relay) |
| `crypto` | 📋 Planned | E2E chat encryption (X25519 + AEAD) |
| `storage` | 📋 Planned | Local chat and contact storage |
| `ui` | ✅ Ready | TUI components on ratatui (for Client) |
| `utils` | 📋 Planned | Logging, helpers |

## Feature Flags

```toml
[dependencies]
# For custom app: pick what you need
undertow-protocol = { 
    git = "https://github.com/daniil-verba/undertow-protocol",
    default-features = false,
    features = ["network", "protocol", "storage"]
}
```

| Feature | Includes | Dependencies |
|---------|----------|-------------|
| `network` | P2P, NAT, STUN, hole punching, relay | tokio, mio, socket2 |
| `crypto` | X25519, SHA-256 | x25519-dalek, sha2, ring |
| `protocol` | PeerId, packets, bincode | serde, bincode |
| `storage` | Local storage | — |
| `ui` | TUI components | ratatui, crossterm |
| `beacon` | `network` + `crypto` + `protocol` | — |
| `client` | `network` + `crypto` + `protocol` + `storage` | — |

## Quick Start

### Minimal example

```rust
use undertow_protocol::{PeerId, Network};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create identity (generates X25519 keypair)
    let peer_id = PeerId::generate();
    println!("My ID: {}", peer_id);

    // Connect to network
    let network = Network::builder()
        .bootstrap("harbor.undertow.example:443")
        .connect()
        .await?;

    // Send message
    network.send_to(peer_id, b"Hello, Undertow!").await?;

    Ok(())
}
```

### Full example

See [**Undertow-Client**](https://github.com/daniil-verba/undertow-client) — an open-source TUI messenger demonstrating the full protocol feature set.

## Ecosystem

| Repository | Purpose | Status |
|------------|---------|--------|
| [undertow-protocol](https://github.com/daniil-verba/undertow-protocol) | Core library | 🚧 Prototype |
| [undertow-client](https://github.com/daniil-verba/undertow-client) | TUI messenger, usage example | 🚧 Prototype |
| [undertow-beacon](https://github.com/daniil-verba/undertow-beacon) | Relay / rendezvous server for VPS | 🚧 Prototype |

## "One Account — All Apps" Concept

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Game A    │     │   Client    │     │  Telegram   │
│  (wrapper)  │◄───►│  (TUI)      │◄───►│    bot      │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌──────┴──────┐
                    │  UTW Account  │
                    │   "Daniil"    │
                    │  PeerId + key  │
                    └─────────────┘
                           │
       ┌───────────────────┼───────────────────┐
       │                   │                   │
  ┌────┴────┐         ┌────┴────┐         ┌────┴────┐
  │ Chat w/ │         │ Chat w/ │         │ Chat w/ │
  │ Alice   │         │  Bob    │         │ Charlie │
  │(in game) │         │(in client│        │(in tg bot)│
  └─────────┘         └─────────┘         └─────────┘
  
  → All messages from all apps in unified history
  → Developer decides: show chat in their UI or not
```

## Terminology

| Term | Meaning |
|------|---------|
| **Undertow** | Underwater current. A network that cannot be stopped or blocked |
| **Beacon** | Lighthouse. VPS server with public IP: rendezvous (peer discovery) + relay |
| **Harbor** | Port. Bootstrap node for stable network entry |
| **PeerId** | SHA-256 hash of X25519 public key. Unique node identifier |
| **UTW** | Short for Undertow |

## Roadmap

| Status | Milestone |
| :---: | :--- |
| ✅ | Project structure and feature flags |
| ✅ | TUI prototype (Client) |
| ✅ | Beacon server stub |
| 🔄 | Messaging over LAN *(in progress)* |
| 📋 | NAT traversal (STUN + hole punching) |
| 📋 | Relay through Beacon (TURN-like) |
| 📋 | Cryptography (X25519 + AEAD) |
| 📋 | DHT based on Kademlia |
| 📋 | Network economy (credits, incentives) |

## Requirements

- **Rust** — latest stable (newer is better)
- **OS** — Linux, macOS, Windows
- **Network** — UDP for P2P, TCP/WebSocket for Beacon

## 🤝 Join the Community

Undertow is built by developers for developers. Whether you want to:

- 🐛 **Report a bug** — [Open an issue](https://github.com/daniil-verba/undertow-protocol/issues)
- 💡 **Suggest a feature** — [Start a discussion](https://github.com/daniil-verba/undertow-protocol/discussions)
- 💻 **Write code** — Check the [good first issues](https://github.com/daniil-verba/undertow-protocol/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)
- 📚 **Improve docs** — We need help with that too!

**Quick start for contributors:**
```bash
git clone https://github.com/your-username/undertow-protocol.git
cd undertow-protocol
cargo build
cargo test
```

**Before you start:**
- 📖 Read the [Contributing Guide](CONTRIBUTING.md)
- 📜 Read the [Code of Conduct](CODE_OF_CONDUCT.md)

> 👋 I'm the founder. I personally review every PR and help new contributors get started. No contribution is too small. Join us on [Discord](https://discord.gg/rqJJf9WcV6) — we'll help you find your first issue.

---

## 🌊 The Vision

We're building a network that cannot be stopped. A network where developers don't need to be infrastructure experts, and users own their identity.

**One protocol. One identity. Infinite applications.**

---

## Contacts

- 📧 [daniilverba123@gmail.com](mailto:daniilverba123@gmail.com)
- 💬 [Discord](https://discord.gg/rqJJf9WcV6)
- 🐛 Issues — [GitHub](https://github.com/daniil-verba/undertow-protocol/issues)

---

## 📄 License

[MIT](LICENSE) — free to use, modify, and distribute.

<div align="center">

*Undertow — the current that cannot be stopped.*

</div>

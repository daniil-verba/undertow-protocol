# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.2] - 2026-07-16

### Fixed
- Fixed typos in files README.md & README.ru.md, correct "current version".

---

## [0.1.1] - 2026-07-16

### Fixed
- Fixed typos in files README.md & CONTRIBUTING.ru.md

---

## [0.1.0] - 2026-07-16

### Added
- First release of Undertow Protocol
- PeerId generation based on X25519
- Beacon server client (registration, relay)
- TUI client (`undertow-client`) for testing
- STUN client for external IP detection
- LAN peer discovery
- `network` module: connection management, NAT traversal (stub), DHT (stub)
- `protocol` module: packet formats, bincode serialization
- `storage` module: user profile (keys, settings)
- Linux and Termux (Android) support
- Documentation in Russian and English

### Changed
- None (first release)

### Fixed
- None (first release)

### Security
- X25519 used for key generation
- SHA-256 used for PeerId creation

---

## [Unreleased]

### Added
- [ ] E2E encryption (X25519 + AEAD)
- [ ] Kademlia-based DHT
- [ ] LAN peer discovery
- [ ] WebSocket support for Beacon

### Changed
- [ ] Refactor `network` module for better readability

### Fixed
- [ ] Reconnection issue after Beacon disconnection

### Security
- [ ] Planned code audit

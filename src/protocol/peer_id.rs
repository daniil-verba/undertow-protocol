//! ## PeerId Module / Модуль PeerId
//!
//! 32-byte unique identifier for each node in the network.
// / 32-байтовый уникальный идентификатор каждого узла в сети.

use crate::protocol::crypto::sha256;
use rand::Rng;

/// Unique 32-byte identifier for a network peer.
/// Уникальный 32-байтовый идентификатор сетевого пира.
///
/// Derived from SHA-256 of the public key, ensuring cryptographic identity.
/// / Выводится из SHA-256 публичного ключа, обеспечивая криптографическую идентичность.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeerId([u8; 32]);

impl PeerId {
    /// Creates PeerId from raw 32 bytes.
    /// / Создаёт PeerId из сырых 32 байт.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Derives PeerId from a public key via SHA-256.
    /// / Выводит PeerId из публичного ключа через SHA-256.
    pub fn from_public_key(public_key: &[u8]) -> Self {
        Self(sha256(public_key))
    }

    /// Returns the PeerId as a lowercase hex string (64 chars).
    /// / Возвращает PeerId в виде hex-строки в нижнем регистре (64 символа).
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parses PeerId from a 64-character hex string.
    /// / Парсит PeerId из 64-символьной hex-строки.
    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        if hex_str.len() != 64 {
            return Err(format!(
                "Expected 64 hex chars, got {} / Ожидалось 64 hex-символа, получено {}",
                hex_str.len(),
                hex_str.len()
            ));
        }

        let mut bytes = [0u8; 32];
        for (i, chunk) in hex_str.as_bytes().chunks(2).enumerate() {
            let pair = std::str::from_utf8(chunk)
                .map_err(|_| "Invalid UTF-8 / Некорректный UTF-8".to_string())?;
            bytes[i] = u8::from_str_radix(pair, 16).map_err(|_| {
                format!(
                    "Invalid hex at position {} / Некорректный hex на позиции {}",
                    i * 2,
                    i * 2
                )
            })?;
        }

        Ok(Self(bytes))
    }

    /// Returns a reference to the raw 32-byte array.
    /// / Возвращает ссылку на сырой 32-байтовый массив.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Generates a random PeerId (for testing).
    /// / Генерирует случайный PeerId (для тестирования).
    pub fn random() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill(&mut bytes);
        Self(bytes)
    }

    /// Returns a shortened display form (first 8 hex chars).
    /// / Возвращает сокращённую форму для отображения (первые 8 hex-символов).
    pub fn short(&self) -> String {
        self.to_hex()[..8].to_string()
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

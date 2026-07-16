//! ## Cryptography Module / Модуль криптографии
//!
//! Provides SHA-256 hashing and X25519 key pair generation.
//! / Предоставляет хеширование SHA-256 и генерацию ключевых пар X25519.

use sha2::{Digest, Sha256};

/// Computes SHA-256 hash of input data.
/// / Вычисляет SHA-256 хеш входных данных.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// X25519 key pair for end-to-end encryption.
/// / Ключевая пара X25519 для сквозного шифрования.
pub struct KeyPair {
    secret: [u8; 32],
    public: x25519_dalek::PublicKey,
}

impl KeyPair {
    /// Generates a new random key pair.
    /// / Генерирует новую случайную ключевую пару.
    pub fn generate() -> Self {
        // Generate random 32 bytes for secret key using OS RNG
        // Генерируем случайные 32 байта для секретного ключа через OS RNG
        let mut secret_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut secret_bytes);

        // Create StaticSecret from bytes (requires static_secrets feature)
        // Создаём StaticSecret из байт (требует фичи static_secrets)
        let static_secret = x25519_dalek::StaticSecret::from(secret_bytes);
        let public = x25519_dalek::PublicKey::from(&static_secret);

        Self {
            secret: secret_bytes,
            public,
        }
    }

    /// Returns the 32-byte public key.
    /// / Возвращает 32-байтовый публичный ключ.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.public.to_bytes()
    }

    /// Derives PeerId from the public key (SHA-256 of pubkey).
    /// / Выводит PeerId из публичного ключа (SHA-256 от pubkey).
    pub fn peer_id(&self) -> crate::protocol::peer_id::PeerId {
        crate::protocol::peer_id::PeerId::from_public_key(&self.public_key_bytes())
    }

    /// Returns the raw secret key bytes (for serialization).
    /// / Возвращает сырые байты секретного ключа (для сериализации).
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.secret
    }

    /// Reconstructs KeyPair from saved secret bytes.
    /// / Восстанавливает KeyPair из сохранённых байт секрета.
    pub fn from_secret_bytes(bytes: [u8; 32]) -> Self {
        let static_secret = x25519_dalek::StaticSecret::from(bytes);
        let public = x25519_dalek::PublicKey::from(&static_secret);
        Self {
            secret: bytes,
            public,
        }
    }
}

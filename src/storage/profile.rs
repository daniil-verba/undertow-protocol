//! ## Profile (User Identity) / Профиль (идентичность пользователя)
//!
//! Stores username, private key and derived PeerId.
//! / Хранит имя пользователя, приватный ключ и производный PeerId.

use crate::protocol::crypto::KeyPair;
use crate::protocol::peer_id::PeerId;
use crate::storage::error::StorageError;
use crate::storage::paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

/// User profile with persistent identity.
/// / Профиль пользователя с постоянной идентичностью.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Human-readable username (must be unique in network).
    /// / Человекочитаемое имя пользователя (должно быть уникальным в сети).
    pub username: String,

    /// Private key as 32 bytes (X25519).
    /// / Приватный ключ как 32 байта (X25519).
    #[serde(with = "hex_serde")]
    private_key: [u8; 32],

    /// Derived PeerId (SHA‑256 of public key). Stored for convenience.
    /// / Производный PeerId (SHA‑256 от публичного ключа). Хранится для удобства.
    #[serde(with = "hex_serde")]
    pub peer_id: [u8; 32],
}

// Custom serialization for arrays as hex strings.
// / Пользовательская сериализация массивов как hex-строк.
mod hex_serde {
    // use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = hex::encode(bytes);
        serializer.serialize_str(&hex)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        let bytes = hex::decode(&hex).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Expected 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

impl Profile {
    /// Creates a new profile with given username and freshly generated key pair.
    /// / Создаёт новый профиль с указанным именем и свежесгенерированной ключевой парой.
    pub fn new(username: String) -> Result<Self, StorageError> {
        if username.trim().is_empty() {
            return Err(StorageError::EmptyUsername);
        }
        let keypair = KeyPair::generate();
        let peer_id = keypair.peer_id();

        Ok(Self {
            username,
            private_key: keypair.secret_bytes(),
            peer_id: *peer_id.as_bytes(),
        })
    }

    /// Restores a profile from existing private key and username.
    /// / Восстанавливает профиль из существующего приватного ключа и имени.
    pub fn from_existing(username: String, private_key: [u8; 32]) -> Self {
        let keypair = KeyPair::from_secret_bytes(private_key);
        let peer_id = keypair.peer_id();
        Self {
            username,
            private_key,
            peer_id: *peer_id.as_bytes(),
        }
    }

    /// Returns the PeerId as a `PeerId` type.
    /// / Возвращает PeerId как тип `PeerId`.
    pub fn peer_id(&self) -> PeerId {
        PeerId::new(self.peer_id)
    }

    /// Returns the private key.
    /// / Возвращает приватный ключ.
    pub fn private_key(&self) -> [u8; 32] {
        self.private_key
    }

    /// Reconstructs the full KeyPair from stored secret.
    /// / Восстанавливает полную KeyPair из сохранённого секрета.
    pub fn keypair(&self) -> KeyPair {
        KeyPair::from_secret_bytes(self.private_key)
    }

    pub fn update_username(&mut self, new_username: String) -> Result<(), StorageError> {
        let trimmed = new_username.trim();
        if trimmed.is_empty() {
            return Err(StorageError::EmptyUsername);
        }

        // Проверка на допустимые символы (можно настроить)
        if trimmed.len() > 32 {
            return Err(StorageError::UsernameTooLong);
        }

        self.username = trimmed.to_string();
        save_profile(self)?;
        Ok(())
    }
}

/// Loads the profile from the default file.
/// / Загружает профиль из файла по умолчанию.
pub fn load_profile() -> Result<Profile, StorageError> {
    let path = paths::profile_path();
    if !path.exists() {
        return Err(StorageError::ProfileNotFound(path.display().to_string()));
    }
    let content = fs::read_to_string(&path)?;
    let profile: Profile = toml::from_str(&content)?;
    Ok(profile)
}

/// Saves the profile to the default file (creates directory if missing).
/// / Сохраняет профиль в файл по умолчанию (создаёт директорию при необходимости).
pub fn save_profile(profile: &Profile) -> Result<(), StorageError> {
    let path = paths::profile_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string(profile)?;
    let mut file = fs::File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Initialises the profile: loads existing or creates a new one.
/// / Инициализирует профиль: загружает существующий или создаёт новый.
pub fn init_profile(username: Option<String>) -> Result<Profile, StorageError> {
    match load_profile() {
        Ok(profile) => {
            // If username is provided, update it (optional).
            if let Some(new_username) = username {
                if !new_username.trim().is_empty() && new_username != profile.username {
                    let mut updated = profile;
                    updated.username = new_username;
                    save_profile(&updated)?;
                    return Ok(updated);
                }
            }
            Ok(profile)
        }
        Err(StorageError::ProfileNotFound(_)) => {
            // Create new profile with provided username or ask interactively.
            let username = username.unwrap_or_else(|| {
                println!("New Undertow profile. Please enter your username:");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                input.trim().to_string()
            });
            let profile = Profile::new(username)?;
            save_profile(&profile)?;
            Ok(profile)
        }
        Err(e) => Err(e),
    }
}

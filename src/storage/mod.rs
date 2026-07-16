//! ## Storage Module / Модуль хранения
//!
//! Persistent storage for user profiles (keys, PeerId, username).
//! / Постоянное хранение профилей пользователей (ключи, PeerId, имя пользователя).

pub mod error;
pub mod paths;
pub mod profile;

// Re-export main types / Реэкспорт основных типов
pub use error::StorageError;
pub use paths::{config_dir, profile_path};
pub use profile::{init_profile, load_profile, save_profile, Profile};

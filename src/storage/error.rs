//! ## Storage Errors / Ошибки хранилища

use std::io;
use thiserror::Error;

/// Errors that can occur during storage operations.
/// / Ошибки, возникающие при операциях хранения.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Invalid private key length")]
    InvalidKeyLength,

    #[error("Username cannot be empty")]
    EmptyUsername,

    #[error("Username too long (max 32 characters)")]
    UsernameTooLong,
}

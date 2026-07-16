//! ## Storage Paths / Пути для хранения
//!
//! Cross‑platform configuration directories.
//! / Кросс-платформенные директории конфигурации.

use std::path::PathBuf;

/// Returns the configuration directory for Undertow.
/// / Возвращает директорию конфигурации для Undertow.
pub fn config_dir() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("undertow");
    dir
}

/// Returns the full path to the profile file.
/// / Возвращает полный путь к файлу профиля.
pub fn profile_path() -> PathBuf {
    let mut path = config_dir();
    path.push("profile.toml");
    path
}

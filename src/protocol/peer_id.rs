//! ## PeerId Module / Модуль PeerId
//!
//! 32-байтовый уникальный идентификатор каждого узла в сети.
//! В iroh 1.0 тип NodeId был переименован в EndpointId, так как публичный ключ
//! используется для идентификации именно конечной точки (endpoint) соединения.

use iroh::EndpointId;
use std::str::FromStr;

/// Уникальный 32-байтовый идентификатор сетевого пира.
/// Прозрачная обертка над iroh::EndpointId для сохранения совместимости API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeerId(EndpointId);

impl PeerId {
    /// Создаёт PeerId из сырых 32 байт.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(EndpointId::from_bytes(&bytes).expect("Некорректные байты для EndpointId"))
    }

    /// Безопасное создание PeerId из байтов с обработкой ошибок.
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self, String> {
        EndpointId::from_bytes(&bytes)
            .map(Self)
            .map_err(|e| format!("Некорректные байты EndpointId: {}", e))
    }

    /// Выводит PeerId из публичного ключа.
    pub fn from_public_key(public_key: &[u8]) -> Self {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&public_key[..32]);
        Self::new(bytes)
    }

    /// Возвращает PeerId в виде hex-строки в нижнем регистре (64 символа).
    pub fn to_hex(&self) -> String {
        self.0.to_string()
    }

    /// Парсит PeerId из 64-символьной hex-строки.
    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        EndpointId::from_str(hex_str)
            .map(Self)
            .map_err(|e| format!("Ошибка парсинга hex: {}", e))
    }

    /// Возвращает ссылку на сырой 32-байтовый массив.
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Генерирует случайный PeerId (для тестирования).
    pub fn random() -> Self {
        let secret = iroh::SecretKey::generate();
        Self(secret.public())
    }

    /// Возвращает сокращённую форму для отображения (первые 8 hex-символов).
    pub fn short(&self) -> String {
        self.to_hex()[..8].to_string()
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// Прозрачная конвертация между PeerId и EndpointId
impl From<PeerId> for EndpointId {
    fn from(val: PeerId) -> Self {
        val.0
    }
}

impl From<EndpointId> for PeerId {
    fn from(val: EndpointId) -> Self {
        Self(val)
    }
}

//! ## Packet Module / Модуль пакетов
//!
//! Defines the wire format for all network messages.
//! / Определяет формат передачи всех сетевых сообщений.
//!
//! Packet structure / Структура пакета:
//! ```text
//! [MAGIC: 4 bytes "HIVE"]
//! [VERSION: 1 byte]
//! [TYPE: 1 byte]
//! [PAYLOAD_LEN: 4 bytes BE]
//! [PAYLOAD: variable]
//! [CRC32: 4 bytes BE]
//! ```

use crate::protocol::peer_id::PeerId;

/// Magic bytes identifying Undertow protocol packets.
/// / Магические байты, идентифицирующие пакеты протокола Undertow.
const MAGIC: &[u8] = b"HIVE";

/// Current protocol version.
/// / Текущая версия протокола.
const VERSION: u8 = 0x01;

/// Types of network packets / Типы сетевых пакетов.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    /// Ping — connection check / Проверка соединения
    Ping = 0x01,
    /// Pong — ping response / Ответ на ping
    Pong = 0x02,
    /// FindNode — DHT node lookup / Поиск узла в DHT
    FindNode = 0x03,
    /// Nodes — DHT response with peer list / Ответ DHT со списком пиров
    Nodes = 0x04,
    /// Message — chat message / Сообщение чата
    Message = 0x05,
    /// Relay — relayed traffic / Ретранслируемый трафик
    Relay = 0x06,
}

impl PacketType {
    /// Converts u8 to PacketType, returns None if invalid.
    /// / Преобразует u8 в PacketType, возвращает None если неверно.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Ping),
            0x02 => Some(Self::Pong),
            0x03 => Some(Self::FindNode),
            0x04 => Some(Self::Nodes),
            0x05 => Some(Self::Message),
            0x06 => Some(Self::Relay),
            _ => None,
        }
    }
}

/// A network packet with typed payload and CRC32 integrity check.
/// / Сетевой пакет с типизированной полезной нагрузкой и проверкой целостности CRC32.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub version: u8,
    pub packet_type: PacketType,
    pub payload: Vec<u8>,
}

impl Packet {
    // ==================== CONSTRUCTORS / КОНСТРУКТОРЫ ====================

    /// Creates a Ping packet.
    /// / Создаёт пакет Ping.
    pub fn ping() -> Self {
        Self {
            version: VERSION,
            packet_type: PacketType::Ping,
            payload: Vec::new(),
        }
    }

    /// Creates a Pong packet.
    /// / Создаёт пакет Pong.
    pub fn pong() -> Self {
        Self {
            version: VERSION,
            packet_type: PacketType::Pong,
            payload: Vec::new(),
        }
    }

    /// Creates a FindNode packet for DHT lookups.
    /// / Создаёт пакет FindNode для поиска в DHT.
    pub fn find_node(target: &PeerId) -> Self {
        Self {
            version: VERSION,
            packet_type: PacketType::FindNode,
            payload: target.as_bytes().to_vec(),
        }
    }

    /// Creates a Message packet with sender, recipient, and content.
    /// / Создаёт пакет Message с отправителем, получателем и содержимым.
    ///
    /// Payload format / Формат полезной нагрузки:
    /// ```text
    /// [from: 32 bytes] [to: 32 bytes] [content: variable]
    /// ```
    pub fn message(from: &PeerId, to: &PeerId, content: &[u8]) -> Self {
        let mut payload = Vec::with_capacity(32 + 32 + content.len());
        payload.extend_from_slice(from.as_bytes()); // Sender / Отправитель
        payload.extend_from_slice(to.as_bytes()); // Recipient / Получатель
        payload.extend_from_slice(content); // Content / Содержимое
        Self {
            version: VERSION,
            packet_type: PacketType::Message,
            payload,
        }
    }

    // ==================== SERIALIZATION / СЕРИАЛИЗАЦИЯ ====================

    /// Serializes the packet to bytes for network transmission.
    /// / Сериализует пакет в байты для сетевой передачи.
    pub fn serialize(&self) -> Vec<u8> {
        let payload_len = self.payload.len() as u32;
        // magic(4) + version(1) + type(1) + len(4) + payload + crc(4)
        let total_len = 4 + 1 + 1 + 4 + self.payload.len() + 4;

        let mut buf = Vec::with_capacity(total_len);

        buf.extend_from_slice(MAGIC); // Magic
        buf.push(self.version); // Version
        buf.push(self.packet_type as u8); // Type
        buf.extend_from_slice(&payload_len.to_be_bytes()); // Payload length
        buf.extend_from_slice(&self.payload); // Payload

        // CRC32 of everything after magic
        // CRC32 всего после magic
        let crc = crc32fast::hash(&buf[4..]);
        buf.extend_from_slice(&crc.to_be_bytes());

        buf
    }

    /// Deserializes bytes into a Packet, validating magic, version, and CRC.
    /// / Десериализует байты в Packet, проверяя magic, версию и CRC.
    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        // Minimum size: magic(4) + ver(1) + type(1) + len(4) + crc(4) = 14
        if data.len() < 14 {
            return Err(format!(
                "Data too short: {} bytes / Данные слишком короткие: {} байт",
                data.len(),
                data.len()
            ));
        }

        // Validate magic / Проверка magic
        if &data[0..4] != MAGIC {
            return Err("Invalid magic / Неверный magic".to_string());
        }

        // Validate version / Проверка версии
        let version = data[4];
        if version != VERSION {
            return Err(format!(
                "Unsupported version: {} / Неподдерживаемая версия: {}",
                version, version
            ));
        }

        // Parse type / Парсинг типа
        let type_byte = data[5];
        let packet_type = PacketType::from_u8(type_byte).ok_or_else(|| {
            format!(
                "Unknown packet type: 0x{:02x} / Неизвестный тип пакета: 0x{:02x}",
                type_byte, type_byte
            )
        })?;

        // Parse payload length / Парсинг длины полезной нагрузки
        let payload_len = u32::from_be_bytes([data[6], data[7], data[8], data[9]]) as usize;

        // Validate total length / Проверка общей длины
        let expected_len = 10 + payload_len + 4;
        if data.len() != expected_len {
            return Err(format!(
                "Length mismatch: expected {} bytes, got {} / Несоответствие длины: ожидалось {} байт, получено {}",
                expected_len, data.len(), expected_len, data.len()
            ));
        }

        // Extract payload / Извлечение полезной нагрузки
        let payload = data[10..10 + payload_len].to_vec();

        // Validate CRC32 / Проверка CRC32
        let crc_expected = u32::from_be_bytes([
            data[10 + payload_len],
            data[10 + payload_len + 1],
            data[10 + payload_len + 2],
            data[10 + payload_len + 3],
        ]);

        let crc_actual = crc32fast::hash(&data[4..10 + payload_len]);
        if crc_expected != crc_actual {
            return Err(format!(
                "CRC mismatch: expected 0x{:08x}, got 0x{:08x} / Несоответствие CRC: ожидалось 0x{:08x}, получено 0x{:08x}",
                crc_expected, crc_actual, crc_expected, crc_actual
            ));
        }

        Ok(Self {
            version,
            packet_type,
            payload,
        })
    }

    // ==================== PAYLOAD HELPERS / ПОМОЩНИКИ ====================

    /// Extracts sender PeerId from a Message packet.
    /// / Извлекает PeerId отправителя из пакета Message.
    pub fn message_sender(&self) -> Option<PeerId> {
        if self.packet_type != PacketType::Message || self.payload.len() < 32 {
            return None;
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&self.payload[0..32]);
        Some(PeerId::new(bytes))
    }

    /// Extracts recipient PeerId from a Message packet.
    /// / Извлекает PeerId получателя из пакета Message.
    pub fn message_recipient(&self) -> Option<PeerId> {
        if self.packet_type != PacketType::Message || self.payload.len() < 64 {
            return None;
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&self.payload[32..64]);
        Some(PeerId::new(bytes))
    }

    /// Extracts message content (text) from a Message packet.
    /// / Извлекает содержимое сообщения (текст) из пакета Message.
    pub fn message_content(&self) -> Option<&[u8]> {
        if self.packet_type != PacketType::Message || self.payload.len() < 64 {
            return None;
        }
        Some(&self.payload[64..])
    }
}

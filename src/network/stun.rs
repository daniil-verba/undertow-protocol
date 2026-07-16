//! ## STUN Client / STUN-клиент
//!
//! Discovers external IP:port via STUN servers.
//! / Обнаруживает внешний IP:порт через STUN-серверы.

use std::net::SocketAddr;
use tokio::net::{lookup_host, UdpSocket};

pub struct StunClient;

impl StunClient {
    /// Discovers external address via STUN, trying multiple servers.
    /// / Обнаруживает внешний адрес через STUN, пробуя несколько серверов.
    pub async fn discover() -> Result<SocketAddr, String> {
        // Western servers / Западные серверы
        let western = [
            "stun.l.google.com:19302",
            "stun.cloudflare.com:3478",
            "global.stun.twilio.com:3478",
        ];

        // Russian servers / Российские серверы
        let russian = [
            "stun.sipnet.ru:3478",
            "stun.ipshka.com:3478",
            "stun.magenta.ru:3478",
            "stun.niisi.ru:3478",
            "stun.iptel.ru:3478",
        ];

        // Try western first (usually faster) / Пробуем западные первыми (обычно быстрее)
        for server in &western {
            match Self::query(server).await {
                Ok(addr) => {
                    println!("STUN (.com): {} via {}", addr, server);
                    return Ok(addr);
                }
                Err(e) => println!("STUN {} failed: {}", server, e),
            }
        }

        // Fallback to Russian / Fallback на российские
        for server in &russian {
            match Self::query(server).await {
                Ok(addr) => {
                    println!("STUN (.ru): {} via {}", addr, server);
                    return Ok(addr);
                }
                Err(e) => println!("STUN {} failed: {}", server, e),
            }
        }

        Err("All STUN servers failed / Все STUN-серверы недоступны".to_string())
    }

    /// Queries a single STUN server with 3 retries.
    /// / Запрашивает один STUN-сервер с 3 попытками.
    async fn query(server: &str) -> Result<SocketAddr, String> {
        let server_addr = Self::resolve(server).await?;

        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("UDP bind failed / UDP привязка не удалась: {}", e))?;

        let request = Self::build_binding_request();

        // 3 attempts with 3-second timeout / 3 попытки с таймаутом 3 секунды
        for attempt in 0..3 {
            socket
                .send_to(&request, server_addr)
                .await
                .map_err(|e| format!("STUN send failed / Отправка STUN не удалась: {}", e))?;

            let mut buf = [0u8; 1024];
            match tokio::time::timeout(
                std::time::Duration::from_secs(3),
                socket.recv_from(&mut buf),
            )
            .await
            {
                Ok(Ok((n, _))) => return Self::parse_response(&buf[..n]),
                Ok(Err(e)) if attempt < 2 => {
                    println!("Retry {} after: {}", attempt + 1, e);
                    continue;
                }
                Ok(Err(e)) => return Err(format!("Recv failed / Приём не удался: {}", e)),
                Err(_) if attempt < 2 => continue,
                Err(_) => return Err("Timeout / Таймаут".to_string()),
            }
        }

        Err("All attempts failed / Все попытки не удались".to_string())
    }

    /// Resolves hostname to SocketAddr.
    /// / Резолвит имя хоста в SocketAddr.
    async fn resolve(server: &str) -> Result<SocketAddr, String> {
        let mut addrs = lookup_host(server).await.map_err(|e| {
            format!(
                "DNS lookup failed for '{}' / DNS-запрос не удался для '{}': {}",
                server, server, e
            )
        })?;

        addrs.next().ok_or_else(|| {
            format!(
                "No addresses for '{}' / Нет адресов для '{}'",
                server, server
            )
        })
    }

    /// Builds a STUN Binding Request packet.
    /// / Собирает пакет STUN Binding Request.
    fn build_binding_request() -> Vec<u8> {
        let mut msg = vec![0u8; 20];

        // Message Type: Binding Request (0x0001)
        msg[0] = 0x00;
        msg[1] = 0x01;

        // Message Length: 0
        msg[2] = 0x00;
        msg[3] = 0x00;

        // Magic Cookie: 0x2112A442
        msg[4] = 0x21;
        msg[5] = 0x12;
        msg[6] = 0xA4;
        msg[7] = 0x42;

        // Transaction ID: 12 bytes random / 12 байт случайных
        let tid = rand::random::<[u8; 12]>();
        msg[8..20].copy_from_slice(&tid);

        msg
    }

    /// Parses STUN Binding Response to extract XOR-MAPPED-ADDRESS.
    /// / Парсит ответ STUN Binding Response для извлечения XOR-MAPPED-ADDRESS.
    fn parse_response(data: &[u8]) -> Result<SocketAddr, String> {
        if data.len() < 20 {
            return Err("Response too short / Ответ слишком короткий".to_string());
        }

        // Validate Magic Cookie / Проверка Magic Cookie
        if &data[4..8] != &[0x21, 0x12, 0xA4, 0x42] {
            return Err("Invalid magic cookie / Неверный magic cookie".to_string());
        }

        // Message Type: Binding Response Success (0x0101)
        let msg_type = u16::from_be_bytes([data[0], data[1]]);
        if msg_type != 0x0101 {
            return Err(format!(
                "Unexpected response type: 0x{:04x} / Неожиданный тип ответа: 0x{:04x}",
                msg_type, msg_type
            ));
        }

        let msg_len = u16::from_be_bytes([data[2], data[3]]) as usize;
        if data.len() < 20 + msg_len {
            return Err("Truncated response / Усечённый ответ".to_string());
        }

        // Parse attributes / Парсим атрибуты
        let mut pos = 20;
        while pos < 20 + msg_len {
            if pos + 4 > data.len() {
                break;
            }

            let attr_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
            let attr_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;

            // XOR-MAPPED-ADDRESS = 0x0020
            if attr_type == 0x0020 && pos + 4 + attr_len <= data.len() {
                return Self::parse_xor_mapped(&data[pos + 4..pos + 4 + attr_len], &data[4..8]);
            }

            // Padding to 4 bytes / Выравнивание до 4 байт
            let padded_len = (attr_len + 3) & !3;
            pos += 4 + padded_len;
        }

        Err("XOR-MAPPED-ADDRESS not found / XOR-MAPPED-ADDRESS не найден".to_string())
    }

    /// Parses XOR-MAPPED-ADDRESS attribute.
    /// / Парсит атрибут XOR-MAPPED-ADDRESS.
    fn parse_xor_mapped(data: &[u8], magic_cookie: &[u8]) -> Result<SocketAddr, String> {
        if data.len() < 4 {
            return Err(
                "XOR-MAPPED-ADDRESS too short / XOR-MAPPED-ADDRESS слишком короткий".to_string(),
            );
        }

        let family = data[1];

        if family == 0x01 {
            // IPv4
            if data.len() < 8 {
                return Err(
                    "IPv4 XOR-MAPPED-ADDRESS too short / IPv4 XOR-MAPPED-ADDRESS слишком короткий"
                        .to_string(),
                );
            }

            let xport = u16::from_be_bytes([data[2], data[3]]);
            let port = xport ^ 0x2112;

            let xip = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
            let ip_bytes = xip.to_be_bytes();
            let ip = std::net::Ipv4Addr::new(
                ip_bytes[0] ^ magic_cookie[0],
                ip_bytes[1] ^ magic_cookie[1],
                ip_bytes[2] ^ magic_cookie[2],
                ip_bytes[3] ^ magic_cookie[3],
            );

            Ok(SocketAddr::from((ip, port)))
        } else if family == 0x02 {
            Err("IPv6 not supported yet / IPv6 пока не поддерживается".to_string())
        } else {
            Err(format!(
                "Unknown address family: {} / Неизвестное семейство адресов: {}",
                family, family
            ))
        }
    }
}

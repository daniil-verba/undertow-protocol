//! ## Relay Module / Модуль ретрансляции
//!
//! Simple TCP relay server for fallback connectivity.
//! / Простой TCP-сервер ретрансляции для резервного соединения.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Simple echo relay server (for testing).
/// / Простой echo-сервер ретрансляции (для тестирования).
pub struct RelayServer {
    listener: TcpListener,
}

impl RelayServer {
    pub async fn bind(addr: &str) -> Result<Self, String> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Bind failed / Привязка не удалась: {}", e))?;
        Ok(Self { listener })
    }

    pub async fn run(&self) -> Result<(), String> {
        println!(
            "Relay server on / Сервер ретрансляции на {}",
            self.listener.local_addr().unwrap()
        );

        loop {
            let (stream, addr) = self
                .listener
                .accept()
                .await
                .map_err(|e| format!("Accept error / Ошибка приёма: {}", e))?;

            println!("Relay client / Клиент ретрансляции: {}", addr);

            tokio::spawn(async move {
                if let Err(e) = handle_relay_client(stream).await {
                    eprintln!("Relay error / Ошибка ретрансляции: {}", e);
                }
            });
        }
    }
}

async fn handle_relay_client(mut stream: TcpStream) -> Result<(), String> {
    let mut buf = [0u8; 4096];

    loop {
        let n = stream
            .read(&mut buf)
            .await
            .map_err(|e| format!("Read error / Ошибка чтения: {}", e))?;

        if n == 0 {
            break; // Connection closed / Соединение закрыто
        }

        // TODO: parse packet, determine destination, forward
        // TODO: парсить пакет, определить назначение, переслать
        // For now: echo / Пока: echo
        stream
            .write_all(&buf[0..n])
            .await
            .map_err(|e| format!("Write error / Ошибка записи: {}", e))?;
    }

    Ok(())
}

/// Relay client connector.
/// / Клиент подключения к ретранслятору.
pub struct RelayClient;

impl RelayClient {
    pub async fn connect(relay_addr: &str) -> Result<TcpStream, String> {
        TcpStream::connect(relay_addr).await.map_err(|e| {
            format!(
                "Connect to relay failed / Подключение к ретранслятору не удалось: {}",
                e
            )
        })
    }
}

// use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
// use tokio::sync::Mutex;

// use crate::protocol::packet::{Packet, PacketType};

pub struct RelayServer {
    listener: TcpListener,
}

impl RelayServer {
    pub async fn bind(addr: &str) -> Result<Self, String> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Bind failed: {}", e))?;
        Ok(Self { listener })
    }

    pub async fn run(&self) -> Result<(), String> {
        println!("Relay server on {}", self.listener.local_addr().unwrap());

        loop {
            let (stream, addr) = self
                .listener
                .accept()
                .await
                .map_err(|e| format!("Accept error: {}", e))?;

            println!("Relay client: {}", addr);

            tokio::spawn(async move {
                if let Err(e) = handle_relay_client(stream).await {
                    eprintln!("Relay error: {}", e);
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
            .map_err(|e| format!("Read error: {}", e))?;

        if n == 0 {
            break; // Соединение закрыто
        }

        // TODO: парсить пакет, определить назначение, переслать
        // Пока просто echo для теста
        stream
            .write_all(&buf[0..n])
            .await
            .map_err(|e| format!("Write error: {}", e))?;
    }

    Ok(())
}

pub struct RelayClient;

impl RelayClient {
    pub async fn connect(relay_addr: &str) -> Result<TcpStream, String> {
        TcpStream::connect(relay_addr)
            .await
            .map_err(|e| format!("Connect to relay failed: {}", e))
    }
}

//! ## Iroh Transport Layer / Транспортный слой на базе Iroh

use bytes::Bytes;
use iroh::endpoint::presets;
use iroh::Endpoint;
use iroh::EndpointId;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::protocol::packet::Packet;
use crate::protocol::peer_id::PeerId;

/// Ошибки транспортного уровня
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Ошибка инициализации или работы Iroh: {0}")]
    Iroh(#[from] anyhow::Error),
    #[error("Ошибка сериализации/десериализации пакета: {0}")]
    PacketError(String),
    #[error("Соединение с пиром потеряно или не может быть установлено")]
    ConnectionLost,
    #[error("Неверный PeerId: {0}")]
    InvalidPeerId(String),
}

/// Основной транспортный слой на базе Iroh.
pub struct IrohTransport {
    endpoint: Endpoint,
}

impl IrohTransport {
    /// Инициализирует транспортный слой.
    pub async fn new(secret_key: Option<iroh::SecretKey>) -> Result<Self, TransportError> {
        // В iroh 1.0 builder требует preset. Используем стандартный N0 (relay + discovery)
        let mut builder = Endpoint::builder(presets::N0);

        if let Some(key) = secret_key {
            builder = builder.secret_key(key);
        }

        // bind() автоматически настраивает UDP-сокеты, STUN и relay-серверы
        let endpoint = builder.bind().await.map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(Self { endpoint })
    }

    /// Возвращает криптографический идентификатор (EndpointId) текущего узла.
    pub fn endpoint_id(&self) -> EndpointId {
        // В iroh 1.0 метод node_id() был переименован в id()
        self.endpoint.id()
    }

    /// Возвращает PeerId текущего узла.
    pub fn peer_id(&self) -> PeerId {
        PeerId::from(self.endpoint_id())
    }

    /// Отправляет пакет конкретному пиру.
    pub async fn send_packet(&self, target: PeerId, packet: &Packet) -> Result<(), TransportError> {
        let target_endpoint_id = EndpointId::from_bytes(target.as_bytes())
            .map_err(|e| TransportError::InvalidPeerId(e.to_string()))?;

        let packet_bytes = Bytes::from(packet.serialize());

        let conn = self
            .endpoint
            .connect(target_endpoint_id, b"undertow/1")
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let mut send_stream = conn
            .open_uni()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        send_stream
            .write_all(&packet_bytes)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        send_stream.finish().map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(())
    }

    /// Запускает фоновую задачу для прослушивания входящих соединений и пакетов.
    pub fn start_listening(&self) -> mpsc::Receiver<(PeerId, Packet)> {
        let (tx, rx) = mpsc::channel(1000);
        let endpoint = self.endpoint.clone();

        tokio::spawn(async move {
            while let Some(conn) = endpoint.accept().await {
                let tx_clone = tx.clone();

                tokio::spawn(async move {
                    let conn = match conn.await {
                        Ok(c) => c,
                        Err(_) => return,
                    };

                    // В iroh 1.0 remote_id() возвращает EndpointId напрямую (без Result)
                    let remote_endpoint_id = conn.remote_id();
                    let peer_id = PeerId::from(remote_endpoint_id);

                    // accept_uni() возвращает Result<RecvStream, ConnectionError> (без Option)
                    while let Ok(mut stream) = conn.accept_uni().await {
                        let tx_inner = tx_clone.clone();
                        let peer_id_inner = peer_id;

                        tokio::spawn(async move {
                            // read_to_end возвращает Result<Vec<u8>, ReadToEndError>
                            if let Ok(bytes) = stream.read_to_end(1024 * 1024).await {
                                match Packet::deserialize(&bytes) {
                                    Ok(packet) => {
                                        let _ = tx_inner.send((peer_id_inner, packet)).await;
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[Transport] Ошибка десериализации пакета от {}: {}",
                                            peer_id_inner.short(),
                                            e
                                        );
                                    }
                                }
                            }
                        });
                    }
                });
            }
        });

        rx
    }

    /// Корректное завершение работы транспорта.
    pub async fn shutdown(&self) {
        self.endpoint.close().await;
    }
}

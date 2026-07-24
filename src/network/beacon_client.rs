//! ## Beacon Client / Клиент маяка
//!
//! Отвечает за взаимодействие с серверами Beacon (rendezvous, relay, начисление кредитов).
//! Теперь использует надежный зашифрованный транспорт Iroh вместо сырых TCP-сокетов,
//! что гарантирует работу даже за строгими NAT и фаерволами.

use crate::network::transport::IrohTransport;
use crate::protocol::packet::Packet;
use crate::protocol::peer_id::PeerId;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Клиент для взаимодействия с узлом Beacon.
pub struct BeaconClient {
    /// Ссылка на основной транспортный слой (для отправки пакетов)
    #[allow(dead_code)]
    transport: Arc<IrohTransport>,
    /// PeerId сервера Beacon, к которому мы подключаемся
    beacon_peer_id: PeerId,
    /// Канал для асинхронной отправки пакетов на Beacon
    tx: mpsc::Sender<Packet>,
}

impl BeaconClient {
    /// Создает нового клиента Beacon.
    ///
    /// # Аргументы
    /// * `transport` - Инициализированный транспортный слой Iroh.
    /// * `beacon_peer_id` - Известный PeerId сервера Beacon (загружается из конфига или хардкода).
    pub fn new(transport: Arc<IrohTransport>, beacon_peer_id: PeerId) -> Self {
        // Создаем канал для отправки пакетов.
        // Размер 100 достаточен для очередей служебных сообщений (ping, credits, rendezvous).
        let (tx, mut rx) = mpsc::channel::<Packet>(100);

        let transport_clone = transport.clone();
        let beacon_id = beacon_peer_id;

        // Запускаем фоновую задачу для обработки исходящих пакетов на Beacon.
        // Это развязывает руки основному потоку приложения.
        tokio::spawn(async move {
            while let Some(packet) = rx.recv().await {
                if let Err(e) = transport_clone.send_packet(beacon_id, &packet).await {
                    eprintln!("[BeaconClient] Ошибка отправки пакета на Beacon: {}", e);
                    // TODO: Здесь можно добавить логику повторной отправки (retry with backoff)
                }
            }
        });

        Self {
            transport,
            beacon_peer_id,
            tx,
        }
    }

    /// Асинхронно отправляет пакет на сервер Beacon.
    pub async fn send_packet(&self, packet: &Packet) -> Result<(), String> {
        self.tx
            .send(packet.clone())
            .await
            .map_err(|e| e.to_string())
    }

    /// Возвращает PeerId сервера Beacon, с которым работает клиент.
    pub fn beacon_peer_id(&self) -> PeerId {
        self.beacon_peer_id
    }
}

//! ## K-Bucket Module / Модуль K-бакета
//!
//! Kademlia bucket for DHT routing table.
//! / Kademlia-бакет для таблицы маршрутизации DHT.

use crate::network::dht::xor_distance;
use crate::network::peer::Peer;
use crate::protocol::peer_id::PeerId;

/// Result of inserting a peer into a bucket.
/// / Результат вставки пира в бакет.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertResult {
    Added,   // New peer added / Новый пир добавлен
    Updated, // Existing peer updated / Существующий пир обновлён
    Full,    // Bucket is full / Бакет полон
}

/// Kademlia bucket storing up to max_size peers.
/// / Kademlia-бакет, хранящий до max_size пиров.
pub struct KBucket {
    peers: Vec<Peer>,
    max_size: usize,
}

impl KBucket {
    pub fn new(max_size: usize) -> Self {
        Self {
            peers: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Inserts or updates a peer. Moves to end if updated (LRU).
    /// / Вставляет или обновляет пира. Перемещает в конец при обновлении (LRU).
    pub fn insert(&mut self, peer: Peer) -> InsertResult {
        if let Some(pos) = self.peers.iter().position(|p| p.id() == peer.id()) {
            self.peers.remove(pos);
            self.peers.push(peer);
            return InsertResult::Updated;
        }

        if self.peers.len() < self.max_size {
            self.peers.push(peer);
            InsertResult::Added
        } else {
            InsertResult::Full
        }
    }

    /// Finds k closest peers to target by XOR distance.
    /// / Находит k ближайших пиров к цели по XOR-расстоянию.
    pub fn find_closest(&self, target: &PeerId, k: usize) -> Vec<&Peer> {
        let mut result: Vec<&Peer> = self.peers.iter().collect();

        result.sort_by(|a, b| {
            let dist_a = xor_distance(a.id().as_bytes(), target.as_bytes());
            let dist_b = xor_distance(b.id().as_bytes(), target.as_bytes());
            dist_a.cmp(&dist_b)
        });

        result.into_iter().take(k).collect()
    }

    pub fn peers(&self) -> &[Peer] {
        &self.peers
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn is_full(&self) -> bool {
        self.peers.len() >= self.max_size
    }
}

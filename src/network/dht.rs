//! ## DHT Routing Table / Таблица маршрутизации DHT
//!
//! Kademlia-style routing with 256 buckets.
//! / Маршрутизация в стиле Kademlia с 256 бакетами.

use crate::network::kbucket::{InsertResult, KBucket};
use crate::network::peer::Peer;
use crate::protocol::peer_id::PeerId;

/// Kademlia routing table with 256 k-buckets.
/// / Таблица маршрутизации Kademlia с 256 k-бакетами.
pub struct RoutingTable {
    local_id: PeerId,
    buckets: Vec<KBucket>, // 256 buckets / 256 бакетов
}

impl RoutingTable {
    pub fn new(local_id: PeerId) -> Self {
        let mut buckets = Vec::with_capacity(256);
        for _ in 0..256 {
            buckets.push(KBucket::new(20)); // k = 20
        }

        Self { local_id, buckets }
    }

    /// Finds bucket index = position of highest differing bit.
    /// / Находит индекс бакета = позицию старшего отличающегося бита.
    pub fn bucket_index(&self, peer_id: &PeerId) -> usize {
        let local = self.local_id.as_bytes();
        let peer = peer_id.as_bytes();

        for i in 0..32 {
            if local[i] != peer[i] {
                let xor = local[i] ^ peer[i];
                let bit_pos = xor.leading_zeros() as usize;
                return i * 8 + bit_pos;
            }
        }

        255 // Identical IDs (shouldn't happen) / Идентичные ID (не должно случаться)
    }

    pub fn insert(&mut self, peer: Peer) -> InsertResult {
        let idx = self.bucket_index(peer.id());
        self.buckets[idx].insert(peer)
    }

    /// Finds k closest peers to target across all buckets.
    /// / Находит k ближайших пиров к цели во всех бакетах.
    pub fn find_closest(&self, target: &PeerId, k: usize) -> Vec<&Peer> {
        let mut all_peers: Vec<&Peer> = Vec::new();

        for bucket in &self.buckets {
            all_peers.extend(bucket.peers());
        }

        all_peers.sort_by(|a, b| {
            let dist_a = xor_distance(a.id().as_bytes(), target.as_bytes());
            let dist_b = xor_distance(b.id().as_bytes(), target.as_bytes());
            dist_a.cmp(&dist_b)
        });

        all_peers.into_iter().take(k).collect()
    }

    pub fn local_id(&self) -> &PeerId {
        &self.local_id
    }

    pub fn bucket(&self, index: usize) -> Option<&KBucket> {
        self.buckets.get(index)
    }

    pub fn iter_buckets(&self) -> impl Iterator<Item = &KBucket> {
        self.buckets.iter()
    }

    pub fn total_peers(&self) -> usize {
        self.buckets.iter().map(|b| b.len()).sum()
    }
}

/// XOR distance between two 32-byte arrays.
/// / XOR-расстояние между двумя 32-байтовыми массивами.
pub fn xor_distance(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    for i in 0..32 {
        result[i] = a[i] ^ b[i];
    }
    result
}

//! Глобальный дедуп: один блок хранится один раз. Bloom-фильтр для быстрого отсева (раздел 1.1).

use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BlockRef {
    pub hash: [u8; 32],
    pub algo: u8,
    pub offset: u64,
    pub len: u32,
}

const BLOOM_K: u32 = 4;
const BLOOM_BITS: usize = 256 * 1024 * 8;

pub struct BlockStore {
    pub blocks: HashMap<[u8; 32], (u8, Vec<u8>)>,
    bloom: Vec<u8>,
}

fn bloom_pos(hash: &[u8; 32], seed: u32) -> usize {
    let a = u64::from_le_bytes(hash[0..8].try_into().unwrap());
    let b = u64::from_le_bytes(hash[8..16].try_into().unwrap());
    let c = u64::from_le_bytes(hash[16..24].try_into().unwrap());
    let mix = a.wrapping_add(b.wrapping_mul(seed as u64)).wrapping_add(c);
    (mix as usize) % BLOOM_BITS
}

impl BlockStore {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            bloom: vec![0; BLOOM_BITS.div_ceil(8)],
        }
    }

    pub fn block_hash(data: &[u8]) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(data);
        h.finalize().into()
    }

    fn bloom_may_contain(&self, hash: &[u8; 32]) -> bool {
        for i in 0..BLOOM_K {
            let pos = bloom_pos(hash, i);
            let byte = pos / 8;
            let bit = pos % 8;
            if byte >= self.bloom.len() || (self.bloom[byte] & (1 << bit)) == 0 {
                return false;
            }
        }
        true
    }

    fn bloom_insert(&mut self, hash: &[u8; 32]) {
        for i in 0..BLOOM_K {
            let pos = bloom_pos(hash, i);
            let byte = pos / 8;
            let bit = pos % 8;
            if byte < self.bloom.len() {
                self.bloom[byte] |= 1 << bit;
            }
        }
    }

    pub fn add_file(&mut self, data: &[u8], compressed: &[u8], algo_id: u8) -> BlockRef {
        let hash = Self::block_hash(data);
        let need_insert = !self.bloom_may_contain(&hash) || !self.blocks.contains_key(&hash);
        let used_algo = if need_insert {
            algo_id
        } else {
            self.blocks.get(&hash).map(|(a, _)| *a).unwrap_or(algo_id)
        };
        if need_insert {
            self.blocks.insert(hash, (algo_id, compressed.to_vec()));
            self.bloom_insert(&hash);
        }
        BlockRef {
            hash,
            algo: used_algo,
            offset: 0,
            len: data.len() as u32,
        }
    }

    /// Добавляет чанк (для chunked dedup). Возвращает ссылку на блок; если чанк уже был — переиспользуется.
    pub fn add_chunk(&mut self, data: &[u8], compressed: &[u8], algo_id: u8) -> BlockRef {
        let hash = Self::block_hash(data);
        let need_insert = !self.bloom_may_contain(&hash) || !self.blocks.contains_key(&hash);
        let used_algo = if need_insert {
            algo_id
        } else {
            self.blocks.get(&hash).map(|(a, _)| *a).unwrap_or(algo_id)
        };
        if need_insert {
            self.blocks.insert(hash, (algo_id, compressed.to_vec()));
            self.bloom_insert(&hash);
        }
        BlockRef {
            hash,
            algo: used_algo,
            offset: 0,
            len: data.len() as u32,
        }
    }

    pub fn get_block(&self, hash: &[u8; 32]) -> Option<&(u8, Vec<u8>)> {
        self.blocks.get(hash)
    }
}

impl Default for BlockStore {
    fn default() -> Self {
        Self::new()
    }
}

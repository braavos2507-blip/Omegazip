//! Кодирование по энтропии: Huffman (из раздела Entropy + Huffman).

use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::io::Write;

#[derive(Clone)]
struct Node {
    count: u64,
    sym: Option<u8>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Eq for Node {}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.count.cmp(&self.count)
    }
}

fn build_tree(freq: &[u64; 256]) -> Option<Box<Node>> {
    let mut heap: BinaryHeap<Box<Node>> = (0..256)
        .filter(|&i| freq[i] > 0)
        .map(|i| Box::new(Node {
            count: freq[i],
            sym: Some(i as u8),
            left: None,
            right: None,
        }))
        .collect();
    if heap.is_empty() {
        return None;
    }
    if heap.len() == 1 {
        let only = heap.pop().unwrap();
        return Some(Box::new(Node {
            count: only.count,
            sym: None,
            left: Some(only),
            right: None,
        }));
    }
    while heap.len() > 1 {
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        let sum = a.count + b.count;
        heap.push(Box::new(Node {
            count: sum,
            sym: None,
            left: Some(a),
            right: Some(b),
        }));
    }
    heap.pop()
}

fn build_codes(node: &Node, prefix: u32, len: u8, out: &mut [(u32, u8); 256]) {
    if let Some(s) = node.sym {
        out[s as usize] = (prefix, len);
        return;
    }
    if let Some(ref l) = node.left {
        build_codes(l, prefix << 1, len + 1, out);
    }
    if let Some(ref r) = node.right {
        build_codes(r, (prefix << 1) | 1, len + 1, out);
    }
}

pub fn encode(data: &[u8]) -> std::io::Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(vec![]);
    }
    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let root = match build_tree(&freq) {
        Some(r) => r,
        None => return Ok(vec![]),
    };
    let mut codes = [(0u32, 0u8); 256];
    build_codes(&root, 0, 0, &mut codes);

    let mut out = Vec::new();
    out.write_all(&(data.len() as u32).to_le_bytes())?;
    for i in 0..256 {
        if freq[i] > 0 {
            out.push(i as u8);
            let len = codes[i].1 as usize;
            out.push(len as u8);
            let prefix = codes[i].0;
            let n = len.div_ceil(8);
            if n > 0 {
                let shifted = prefix << (n * 8 - len);
                for j in 0..n {
                    out.push((shifted >> ((n - 1 - j) * 8)) as u8);
                }
            }
        }
    }
    out.push(0xff);

    let mut buf: u64 = 0;
    let mut bit_count: u32 = 0;
    for &b in data {
        let (prefix, len) = codes[b as usize];
        buf = (buf << len) | prefix as u64;
        bit_count += len as u32;
        while bit_count >= 8 {
            bit_count -= 8;
            out.push((buf >> bit_count) as u8);
            buf &= (1u64 << bit_count) - 1;
        }
    }
    if bit_count > 0 {
        out.push((buf << (8 - bit_count)) as u8);
    }
    Ok(out)
}

pub fn decode(data: &[u8]) -> std::io::Result<Vec<u8>> {
    if data.len() < 4 {
        return Ok(vec![]);
    }
    let orig_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut pos = 4usize;
    let mut sym_to_bits: Vec<(u8, u32, u8)> = Vec::new();
    while pos < data.len() && data[pos] != 0xff {
        let sym = data[pos];
        pos += 1;
        let len = data.get(pos).copied().unwrap_or(0) as usize;
        pos += 1;
        let bytes = len.div_ceil(8);
        if pos + bytes > data.len() {
            break;
        }
        let mut raw = 0u32;
        for j in 0..bytes {
            raw = (raw << 8) | data[pos + j] as u32;
        }
        pos += bytes;
        let code = raw >> (bytes * 8 - len);
        sym_to_bits.push((sym, code, len as u8));
    }
    if pos >= data.len() || data[pos] != 0xff {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad table"));
    }
    pos += 1;
    let bit_slice = &data[pos..];
    let _total_bits = orig_len * 8;
    let mut out = Vec::with_capacity(orig_len);
    let mut bit_pos = 0usize;
    while out.len() < orig_len {
        if bit_pos >= bit_slice.len() * 8 {
            break;
        }
        let mut acc: u32 = 0;
        let mut acc_bits = 0u8;
        let mut matched = false;
        while bit_pos < bit_slice.len() * 8 && !matched {
            let byte = bit_slice[bit_pos / 8];
            let bit = 7 - (bit_pos % 8);
            acc = (acc << 1) | (((byte >> bit) & 1) as u32);
            acc_bits += 1;
            bit_pos += 1;
            for &(sym, code, len) in &sym_to_bits {
                if len == acc_bits && code == (acc & ((1u32 << len) - 1)) {
                    out.push(sym);
                    matched = true;
                    break;
                }
            }
        }
        if !matched && acc_bits > 0 && bit_pos >= bit_slice.len() * 8 {
            break;
        }
    }
    Ok(out)
}

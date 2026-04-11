//! Кодирование по энтропии: Huffman.
//! Формат v2: после `orig_len` — magic `H`, `u16` длина таблицы, таблица, битовый поток
//! (без 0xff-разделителя: символ 255 и байты кода могут быть 0xff).
//! Старый формат (таблица до первого 0xff) по-прежнему распознаётся при отсутствии magic.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::io::Write;

const TABLE_MAGIC_V2: u8 = b'H';

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
        .map(|i| {
            Box::new(Node {
                count: freq[i],
                sym: Some(i as u8),
                left: None,
                right: None,
            })
        })
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

fn collect_lengths(node: &Node, depth: u8, lengths: &mut [u8; 256]) {
    if let Some(s) = node.sym {
        lengths[s as usize] = depth;
        return;
    }
    if let Some(ref l) = node.left {
        collect_lengths(l, depth.saturating_add(1), lengths);
    }
    if let Some(ref r) = node.right {
        collect_lengths(r, depth.saturating_add(1), lengths);
    }
}

fn canonical_codes(freq: &[u64; 256], lengths: &[u8; 256]) -> std::io::Result<[(u128, u8); 256]> {
    let mut codes = [(0u128, 0u8); 256];
    let mut pairs: Vec<(u8, u8)> = (0u16..256)
        .filter(|&i| freq[i as usize] > 0)
        .map(|i| (i as u8, lengths[i as usize]))
        .collect();
    pairs.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    let mut code: u128 = 0;
    let mut curr_len = pairs[0].1;
    for (i, &(sym, len)) in pairs.iter().enumerate() {
        if i > 0 {
            let d = (len.saturating_sub(curr_len)) as u32;
            code = code.checked_shl(d).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "huffman: code overflow")
            })?;
            curr_len = len;
        }
        codes[sym as usize] = (code, len);
        code = code.checked_add(1).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "huffman: code overflow")
        })?;
    }
    Ok(codes)
}

fn push_code_bytes(out: &mut Vec<u8>, code: u128, len: u8) {
    let len = len as usize;
    let n = len.div_ceil(8);
    if n == 0 {
        return;
    }
    let pad = n * 8 - len;
    let shifted = code << pad;
    for j in 0..n {
        out.push((shifted >> ((n - 1 - j) * 8)) as u8);
    }
}

fn write_payload_bits(out: &mut Vec<u8>, cur: &mut u8, cur_bits: &mut u8, code: u128, len: u8) {
    for s in (0..len).rev() {
        let bit = ((code >> s) & 1) as u8;
        *cur = (*cur << 1) | bit;
        *cur_bits += 1;
        if *cur_bits == 8 {
            out.push(*cur);
            *cur = 0;
            *cur_bits = 0;
        }
    }
}

fn build_symbol_table_bytes(freq: &[u64; 256], codes: &[(u128, u8); 256]) -> Vec<u8> {
    let mut table = Vec::new();
    for i in 0..256 {
        if freq[i] > 0 {
            let (code, len) = codes[i];
            table.push(i as u8);
            table.push(len);
            push_code_bytes(&mut table, code, len);
        }
    }
    table
}

fn parse_symbol_table(data: &[u8], mut pos: usize, end: usize) -> std::io::Result<(Vec<(u8, u128, u8)>, usize)> {
    let mut sym_to_bits = Vec::new();
    while pos < end {
        let sym = data[pos];
        pos += 1;
        let len = data.get(pos).copied().unwrap_or(0) as usize;
        pos += 1;
        if len > 128 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "huffman: code length too large",
            ));
        }
        let bytes = len.div_ceil(8);
        if pos + bytes > end {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "huffman: truncated code table",
            ));
        }
        let mut raw: u128 = 0;
        for j in 0..bytes {
            raw = (raw << 8) | data[pos + j] as u128;
        }
        pos += bytes;
        let code = raw >> (bytes * 8 - len);
        sym_to_bits.push((sym, code, len as u8));
    }
    Ok((sym_to_bits, pos))
}

fn decode_payload(
    orig_len: usize,
    sym_to_bits: &[(u8, u128, u8)],
    bit_slice: &[u8],
) -> std::io::Result<Vec<u8>> {
    let max_code_len = sym_to_bits
        .iter()
        .map(|(_, _, l)| *l as u32)
        .max()
        .unwrap_or(1)
        .max(1);

    let mut out = Vec::with_capacity(orig_len);
    let mut bit_pos = 0usize;
    while out.len() < orig_len {
        if bit_pos >= bit_slice.len() * 8 {
            break;
        }
        let mut acc: u128 = 0;
        let mut acc_bits: u32 = 0;
        let mut matched = false;
        while bit_pos < bit_slice.len() * 8 && !matched {
            if acc_bits >= max_code_len {
                break;
            }
            let byte = bit_slice[bit_pos / 8];
            let bit = 7 - (bit_pos % 8);
            acc = (acc << 1) | (((byte >> bit) & 1) as u128);
            acc_bits += 1;
            bit_pos += 1;
            for &(sym, code, len) in sym_to_bits {
                let len_u = len as u32;
                if len_u != acc_bits {
                    continue;
                }
                let mask = if len_u >= 128 {
                    u128::MAX
                } else {
                    (1u128 << len_u) - 1
                };
                if code == (acc & mask) {
                    out.push(sym);
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            if bit_pos >= bit_slice.len() * 8 {
                break;
            }
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "huffman: invalid code stream",
            ));
        }
    }
    Ok(out)
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
    let mut lengths = [0u8; 256];
    collect_lengths(&root, 0, &mut lengths);
    let codes = canonical_codes(&freq, &lengths)?;
    let max_len = codes.iter().map(|(_, l)| *l).max().unwrap_or(0);
    if max_len as usize > 128 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "huffman: code length > 128 (pathological frequencies)",
        ));
    }

    let table = build_symbol_table_bytes(&freq, &codes);
    let n_table: u16 = table
        .len()
        .try_into()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "huffman: table too large"))?;

    let mut out = Vec::new();
    out.write_all(&(data.len() as u32).to_le_bytes())?;
    out.push(TABLE_MAGIC_V2);
    out.write_all(&n_table.to_le_bytes())?;
    out.extend_from_slice(&table);

    let mut cur = 0u8;
    let mut cur_bits = 0u8;
    for &b in data {
        let (code, len) = codes[b as usize];
        write_payload_bits(&mut out, &mut cur, &mut cur_bits, code, len);
    }
    if cur_bits > 0 {
        out.push(cur << (8 - cur_bits));
    }
    Ok(out)
}

pub fn decode(data: &[u8]) -> std::io::Result<Vec<u8>> {
    if data.len() < 4 {
        return Ok(vec![]);
    }
    let orig_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut pos = 4usize;

    let (sym_to_bits, payload_start) = if data.get(4) == Some(&TABLE_MAGIC_V2) {
        if data.len() < 7 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "huffman: truncated v2 header",
            ));
        }
        let n_table = u16::from_le_bytes([data[5], data[6]]) as usize;
        let table_start: usize = 7;
        let table_end = table_start.checked_add(n_table).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "huffman: table size overflow")
        })?;
        if table_end > data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "huffman: truncated v2 table",
            ));
        }
        let (syms, _) = parse_symbol_table(data, table_start, table_end)?;
        (syms, table_end)
    } else {
        let mut sym_to_bits: Vec<(u8, u128, u8)> = Vec::new();
        while pos < data.len() && data[pos] != 0xff {
            let sym = data[pos];
            pos += 1;
            let len = data.get(pos).copied().unwrap_or(0) as usize;
            pos += 1;
            if len > 128 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "huffman: code length too large",
                ));
            }
            let bytes = len.div_ceil(8);
            if pos + bytes > data.len() {
                break;
            }
            let mut raw: u128 = 0;
            for j in 0..bytes {
                raw = (raw << 8) | data[pos + j] as u128;
            }
            pos += bytes;
            let code = raw >> (bytes * 8 - len);
            sym_to_bits.push((sym, code, len as u8));
        }
        if pos >= data.len() || data[pos] != 0xff {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad table",
            ));
        }
        pos += 1;
        (sym_to_bits, pos)
    };

    let bit_slice = &data[payload_start..];
    decode_payload(orig_len, &sym_to_bits, bit_slice)
}

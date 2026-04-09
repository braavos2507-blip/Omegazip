//! Конвейер архивации/разархивации: v1 (совместимость) и v2 (chunked, solid, шифрование, прогресс).

use std::io::Write;
use std::path::Path;

use crate::chunked::chunks;
use crate::codec::{best_compress, decompress, codec_id, codec_from_id, Codec};
use crate::crypto::{decrypt_block, derive_key, encrypt_block, random_salt};
use crate::dedup::BlockStore;
use crate::recovery::{encode_stripe, decode_stripe, STRIPE_DATA_SHARDS, STRIPE_PARITY_SHARDS};
use crc32fast::Hasher as Crc32Hasher;

fn crc32_bytes(data: &[u8]) -> u32 {
    let mut h = Crc32Hasher::new();
    h.update(data);
    h.finalize()
}
use crate::{
    analyze_bytes, preprocess, read_preprocess_result,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use rayon::prelude::*;

/// Относительный путь для манифеста: оба пути приводятся через `canonicalize`
/// (на macOS снимает рассинхрон `/var/...` и `/private/var/...` у базы и файлов из обхода).
fn relative_path_for_manifest(base: &Path, path: &Path) -> String {
    let base_canon = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());
    let path_canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    path_canon
        .strip_prefix(&base_canon)
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            path.strip_prefix(base)
                .unwrap_or(path)
                .to_path_buf()
        })
        .to_string_lossy()
        .into_owned()
}

// ============== Манифест и формат ==============

#[derive(serde::Serialize, serde::Deserialize)]
struct ManifestEntry {
    path: String,
    #[serde(default)]
    algo: u8,
    #[serde(default)]
    hash_hex: String,
    #[serde(default)]
    len: u32,
    /// Чанки (v2 chunked)
    #[serde(skip_serializing_if = "Option::is_none")]
    chunks: Option<Vec<ChunkRef>>,
    /// Solid stream (v2 solid)
    #[serde(skip_serializing_if = "Option::is_none")]
    solid: Option<SolidRef>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ChunkRef {
    hash_hex: String,
    algo: u8,
    len: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SolidRef {
    stream_id: u32,
    offset: u64,
    length: u32,
}

fn hex_to_hash(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut a = [0u8; 32];
    for (i, c) in s.as_bytes().chunks(2).enumerate() {
        if i >= 32 {
            break;
        }
        let h = std::str::from_utf8(c).ok()?;
        a[i] = u8::from_str_radix(h, 16).ok()?;
    }
    Some(a)
}

// ============== Опции и прогресс ==============

/// Фаза операции для отображения в GUI.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressPhase {
    Scanning,
    Compressing,
    Writing,
    Done,
}

/// Состояние прогресса (для GUI и многопоточности).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Progress {
    pub phase: ProgressPhase,
    pub files_done: u32,
    pub files_total: u32,
    pub current_path: Option<String>,
}

/// Пресет сжатия (fast / balanced / max / ultra).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Fast,
    Balanced,
    Max,
    Ultra,
}

impl Preset {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fast" => Some(Preset::Fast),
            "balanced" | "balance" => Some(Preset::Balanced),
            "max" => Some(Preset::Max),
            "ultra" | "maxcompression" | "maxcomp" => Some(Preset::Ultra),
            _ => None,
        }
    }
}

/// Опции сжатия (v2).
#[derive(Clone)]
pub struct CompressOptions {
    /// Размер чанка в байтах; None = целый файл (как v1).
    pub chunk_size: Option<usize>,
    /// Solid-сжатие: несколько файлов в один поток (как 7-Zip).
    pub solid: bool,
    /// Пароль для шифрования (None = без шифрования).
    pub password: Option<String>,
    /// Количество восстановительных блоков на полосу (0, 1 или 2).
    pub recovery_parity: u32,
    /// Пресет (если задан — переопределяет chunk_size/solid/recovery).
    pub preset: Option<Preset>,
    /// Параллельное сжатие файлов (rayon).
    pub parallel: bool,
    /// Callback прогресса.
    pub progress: Option<Arc<dyn Fn(Progress) + Send + Sync>>,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            chunk_size: None,
            solid: false,
            password: None,
            recovery_parity: 0,
            preset: None,
            parallel: true,
            progress: None,
        }
    }
}

fn apply_preset(opts: &mut CompressOptions) {
    if let Some(p) = opts.preset {
        match p {
            Preset::Fast => {
                opts.chunk_size = None;
                opts.solid = false;
                opts.recovery_parity = 0;
            }
            Preset::Balanced => {
                opts.chunk_size = Some(crate::DEFAULT_CHUNK_SIZE);
                opts.solid = false;
                opts.recovery_parity = 0;
            }
            Preset::Max => {
                opts.chunk_size = None;
                opts.solid = true;
                opts.recovery_parity = 0;
            }
            Preset::Ultra => {
                opts.chunk_size = None;
                opts.solid = true;
                opts.recovery_parity = 2;
            }
        }
    }
}

// ============== Сжатие ==============

/// Сжимает файл или папку в архив .oz (v1 по умолчанию).
pub fn compress_to_path(
    input: &Path,
    output: &Path,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    compress_to_path_with_options(input, output, CompressOptions::default())
}

/// Сжимает с опциями (chunked, solid, шифрование, прогресс, пресет).
pub fn compress_to_path_with_options(
    input: &Path,
    output: &Path,
    mut options: CompressOptions,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    apply_preset(&mut options);
    let files: Vec<_> = if input.is_dir() {
        walkdir::WalkDir::new(input)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    } else {
        vec![input.to_path_buf()]
    };
    let total = files.len() as u32;
    let base = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
    let base = if base.is_dir() {
        base
    } else {
        base.parent().unwrap_or(&base).to_path_buf()
    };

    if let Some(ref cb) = options.progress {
        cb(Progress {
            phase: ProgressPhase::Scanning,
            files_done: 0,
            files_total: total,
            current_path: None,
        });
    }

    let chunk_size = options.chunk_size;
    let use_solid = options.solid && chunk_size.is_none();
    let password = options.password.as_deref();
    let recovery = options.recovery_parity.min(2);
    let progress = options.progress.clone();

    if use_solid {
        let use_ultra = options.preset == Some(Preset::Ultra);
        return compress_solid(&files, &base, output, password, recovery, progress, total, use_ultra);
    }

    let mut store = BlockStore::new();
    let mut manifest: Vec<ManifestEntry> = Vec::new();

    type Prepared = (String, Vec<u8>, crate::analyzer::AnalysisResult);
    let prepared: Vec<Prepared> = if options.parallel {
        files
            .par_iter()
            .map(|path| {
                let rel_str = relative_path_for_manifest(&base, path);
                let data = fs::read(path)?;
                let analysis = analyze_bytes(&data, Some(path))?;
                let pre = preprocess(path, &data)?;
                let data_ready = read_preprocess_result(pre)?;
                Ok((rel_str, data_ready, analysis))
            })
            .collect::<Result<Vec<_>, std::io::Error>>()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?
    } else {
        let mut out = Vec::with_capacity(files.len());
        for path in &files {
            let rel_str = relative_path_for_manifest(&base, path);
            let data = fs::read(path)?;
            let analysis = analyze_bytes(&data, Some(path))?;
            let pre = preprocess(path, &data)?;
            let data_ready = read_preprocess_result(pre)?;
            out.push((rel_str, data_ready, analysis));
        }
        out
    };

    for (idx, (rel_str, data_ready, analysis)) in prepared.into_iter().enumerate() {
        if let Some(ref cb) = progress {
            cb(Progress {
                phase: ProgressPhase::Compressing,
                files_done: idx as u32,
                files_total: total,
                current_path: Some(rel_str.clone()),
            });
        }
        if let Some(cs) = chunk_size {
            let chunk_refs: Vec<ChunkRef> = chunks(&data_ready, cs)
                .map(|chunk| {
                    let (codec, compressed) = best_compress(chunk, analysis.context).unwrap_or_else(|_| (Codec::Store, chunk.to_vec()));
                    let algo_id = codec_id(&codec);
                    let ref_ = store.add_chunk(chunk, &compressed, algo_id);
                    ChunkRef {
                        hash_hex: ref_.hash.iter().map(|b| format!("{:02x}", b)).collect::<String>(),
                        algo: ref_.algo,
                        len: ref_.len,
                    }
                })
                .collect();
            manifest.push(ManifestEntry {
                path: rel_str,
                algo: 0,
                hash_hex: String::new(),
                len: data_ready.len() as u32,
                chunks: Some(chunk_refs),
                solid: None,
            });
        } else {
            let (codec, compressed) = best_compress(&data_ready, analysis.context)?;
            let algo_id = codec_id(&codec);
            let block_ref = store.add_file(&data_ready, &compressed, algo_id);
            manifest.push(ManifestEntry {
                path: rel_str,
                algo: algo_id,
                hash_hex: block_ref.hash.iter().map(|b| format!("{:02x}", b)).collect::<String>(),
                len: block_ref.len,
                chunks: None,
                solid: None,
            });
        }
    }

    if let Some(ref cb) = progress {
        cb(Progress {
            phase: ProgressPhase::Writing,
            files_done: total,
            files_total: total,
            current_path: None,
        });
    }

    let use_recovery = recovery > 0;
    let version: u8 = if chunk_size.is_some() || password.is_some() || use_recovery {
        2
    } else {
        1
    };
    let flags: u8 = (chunk_size.is_some() as u8)
        | ((password.is_some() as u8) << 1)
        | ((use_recovery as u8) << 3);

    let salt = if password.is_some() { random_salt() } else { [0u8; 16] };
    let key = password.map(|p| derive_key(p, &salt));

    let mut out = fs::File::create(output)?;
    out.write_all(b"OMEGAZIP")?;
    out.write_all(&[version])?;
    if version == 2 {
        out.write_all(&[flags])?;
        if (flags & 2) != 0 {
            out.write_all(&salt)?;
        }
    }

    let manifest_json = serde_json::to_string(&manifest)?;
    let manifest_len = manifest_json.len() as u32;
    out.write_all(&manifest_len.to_le_bytes())?;
    out.write_all(manifest_json.as_bytes())?;

    let block_order: Vec<_> = store.blocks.keys().copied().collect();
    let mut block_payloads: Vec<Vec<u8>> = Vec::with_capacity(block_order.len());
    for hash in &block_order {
        let (_algo, data) = store.blocks.get(hash).unwrap();
        let payload = if let Some(ref k) = key {
            encrypt_block(k, data)
        } else {
            data.clone()
        };
        block_payloads.push(payload);
    }

    let num_blocks = block_order.len() as u32;
    if use_recovery {
        out.write_all(&num_blocks.to_le_bytes())?;
    }

    for (i, hash) in block_order.iter().enumerate() {
        let payload = &block_payloads[i];
        out.write_all(hash)?;
        let (algo, _) = store.blocks.get(hash).unwrap();
        out.write_all(&[*algo])?;
        out.write_all(&(payload.len() as u32).to_le_bytes())?;
        if use_recovery {
            let crc = crc32_bytes(payload);
            out.write_all(&crc.to_le_bytes())?;
        }
        out.write_all(payload)?;
    }

    if use_recovery && !block_payloads.is_empty() {
        let num_stripes = (block_payloads.len() + STRIPE_DATA_SHARDS - 1) / STRIPE_DATA_SHARDS;
        out.write_all(&(num_stripes as u32).to_le_bytes())?;
        for stripe_start in (0..block_payloads.len()).step_by(STRIPE_DATA_SHARDS) {
            let stripe_blocks: Vec<Vec<u8>> = block_payloads[stripe_start..]
                .iter()
                .take(STRIPE_DATA_SHARDS)
                .cloned()
                .collect();
            if stripe_blocks.len() == STRIPE_DATA_SHARDS {
                if let Ok(parity) = encode_stripe(&stripe_blocks) {
                    let max_len = stripe_blocks.iter().map(|b| b.len()).max().unwrap_or(0);
                    out.write_all(&(max_len as u32).to_le_bytes())?;
                    for p in &parity {
                        out.write_all(p)?;
                    }
                }
            }
        }
    }

    if let Some(ref cb) = progress {
        cb(Progress {
            phase: ProgressPhase::Done,
            files_done: total,
            files_total: total,
            current_path: None,
        });
    }
    Ok(total)
}

/// Solid: один сжатый поток на все файлы (как 7-Zip). use_ultra = XZ-9.
fn compress_solid(
    files: &[std::path::PathBuf],
    base: &Path,
    output: &Path,
    password: Option<&str>,
    _recovery: u32,
    progress: Option<Arc<dyn Fn(Progress) + Send + Sync>>,
    total: u32,
    use_ultra: bool,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream_raw: Vec<u8> = Vec::new();
    let mut offsets: Vec<(String, u64, u32)> = Vec::new();
    let mut pos: u64 = 0;

    for (i, path) in files.iter().enumerate() {
        let rel_str = relative_path_for_manifest(base, path);
        if let Some(ref cb) = progress {
            cb(Progress {
                phase: ProgressPhase::Compressing,
                files_done: i as u32,
                files_total: total,
                current_path: Some(rel_str.clone()),
            });
        }
        let data = fs::read(path)?;
        let pre = preprocess(path, &data)?;
        let data_ready = read_preprocess_result(pre)?;
        let len = data_ready.len() as u32;
        stream_raw.extend_from_slice(&data_ready);
        offsets.push((rel_str, pos, len));
        pos += len as u64;
    }

    let compressed = if use_ultra {
        crate::codec_backend::max_ratio_ultra_encode(&stream_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
    } else {
        crate::codec_backend::balanced_encode(&stream_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
    };
    let hash = BlockStore::block_hash(&stream_raw);
    let algo_id = if use_ultra { 3u8 } else { 1u8 }; // 3 = MaxRatio/ultra

    let version = 2u8;
    let flags = 4u8 | (if password.is_some() { 2u8 } else { 0 }); // bit2=solid, bit1=encrypted
    let mut out = fs::File::create(output)?;
    out.write_all(b"OMEGAZIP")?;
    out.write_all(&[version])?;
    out.write_all(&[flags])?;
    let salt = if password.is_some() {
        let s = random_salt();
        out.write_all(&s)?;
        s
    } else {
        [0u8; 16]
    };

    let manifest: Vec<ManifestEntry> = offsets
        .into_iter()
        .map(|(path, offset, length)| ManifestEntry {
            path,
            algo: 0,
            hash_hex: String::new(),
            len: 0,
            chunks: None,
            solid: Some(SolidRef {
                stream_id: 0,
                offset,
                length,
            }),
        })
        .collect();
    let manifest_json = serde_json::to_string(&manifest)?;
    out.write_all(&(manifest_json.len() as u32).to_le_bytes())?;
    out.write_all(manifest_json.as_bytes())?;

    let payload = if let Some(pass) = password {
        let key = derive_key(pass, &salt);
        encrypt_block(&key, &compressed)
    } else {
        compressed
    };
    out.write_all(&hash)?;
    out.write_all(&[algo_id])?;
    out.write_all(&(payload.len() as u32).to_le_bytes())?;
    out.write_all(&payload)?;

    if let Some(ref cb) = progress {
        cb(Progress {
            phase: ProgressPhase::Done,
            files_done: total,
            files_total: total,
            current_path: None,
        });
    }
    Ok(total)
}

// ============== Распаковка ==============

/// Распаковывает архив .oz (v1 или v2).
pub fn decompress_to_path(
    archive: &Path,
    out_dir: &Path,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    decompress_to_path_with_password(archive, out_dir, None)
}

/// Распаковывает с паролем и опциональным callback прогресса.
pub fn decompress_to_path_with_password(
    archive: &Path,
    out_dir: &Path,
    password: Option<&str>,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    decompress_to_path_with_options(archive, out_dir, password, None)
}

/// Распаковывает с паролем и прогрессом (для GUI).
pub fn decompress_to_path_with_options(
    archive: &Path,
    out_dir: &Path,
    password: Option<&str>,
    progress: Option<Arc<dyn Fn(Progress) + Send + Sync>>,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let data = fs::read(archive)?;
    if !data.starts_with(b"OMEGAZIP") {
        return Err("Invalid archive".into());
    }
    let version = data.get(8).copied().unwrap_or(1);
    let mut pos = 9usize;

    let (flags, salt) = if version >= 2 {
        let f = data.get(pos).copied().unwrap_or(0);
        pos += 1;
        let s = if (f & 2) != 0 {
            if data.len() < pos + 16 {
                return Err("Archive truncated (salt)".into());
            }
            let s: [u8; 16] = data[pos..pos + 16].try_into().unwrap();
            pos += 16;
            s
        } else {
            [0u8; 16]
        };
        (f, s)
    } else {
        (0, [0u8; 16])
    };

    let encrypted = (flags & 2) != 0;
    let key: Option<[u8; 32]> = if encrypted {
        Some(derive_key(
            password.ok_or("Encrypted archive requires password")?,
            &salt,
        ))
    } else {
        None
    };

    let manifest_len = u32::from_le_bytes(
        data[pos..pos + 4].try_into().map_err(|_| "manifest len")?,
    ) as usize;
    pos += 4;
    if data.len() < pos + manifest_len {
        return Err("Archive truncated (manifest)".into());
    }
    let manifest: Vec<ManifestEntry> = serde_json::from_slice(&data[pos..pos + manifest_len])?;
    pos += manifest_len;

    let has_recovery = (flags & 8) != 0;
    let num_blocks = if has_recovery {
        if data.len() < pos + 4 {
            return Err("Archive truncated (num_blocks)".into());
        }
        let n = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        n
    } else {
        usize::MAX
    };

    let block_header_len = if has_recovery { 32 + 1 + 4 + 4 } else { 32 + 1 + 4 };
    let mut blocks_vec: Vec<([u8; 32], u8, usize, Option<Vec<u8>>)> = Vec::new();
    let mut bad_indices: Vec<usize> = Vec::new();
    let mut block_count = 0usize;
    while block_count < num_blocks && pos + block_header_len <= data.len() {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&data[pos..pos + 32]);
        pos += 32;
        let algo = data[pos];
        pos += 1;
        let blen = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let stored_crc = if has_recovery {
            let c = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
            pos += 4;
            Some(c)
        } else {
            None
        };
        if data.len() < pos + blen {
            break;
        }
        let raw = data[pos..pos + blen].to_vec();
        pos += blen;
        let block = if let Some(ref k) = key {
            decrypt_block(k, &raw).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?
        } else {
            raw
        };
        let ok = stored_crc.map_or(true, |c| crc32_bytes(&block) == c);
        if ok {
            blocks_vec.push((hash, algo, blen, Some(block)));
        } else {
            blocks_vec.push((hash, algo, blen, None));
            bad_indices.push(blocks_vec.len() - 1);
        }
        block_count += 1;
    }

    if has_recovery && !bad_indices.is_empty() && pos + 4 <= data.len() {
        let num_stripes = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let mut parity_data: Vec<(usize, Vec<Vec<u8>>)> = Vec::new();
        for _ in 0..num_stripes {
            if pos + 4 > data.len() {
                break;
            }
            let max_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            let p0 = if pos + max_len <= data.len() {
                data[pos..pos + max_len].to_vec()
            } else {
                break;
            };
            pos += max_len;
            let p1 = if pos + max_len <= data.len() {
                data[pos..pos + max_len].to_vec()
            } else {
                break;
            };
            pos += max_len;
            parity_data.push((max_len, vec![p0, p1]));
        }
        for (stripe_idx, (max_len, parity)) in parity_data.iter().enumerate() {
            let start = stripe_idx * STRIPE_DATA_SHARDS;
            let end = (start + STRIPE_DATA_SHARDS).min(blocks_vec.len());
            let stripe_bad: Vec<usize> = bad_indices
                .iter()
                .copied()
                .filter(|&i| i >= start && i < end)
                .collect();
            if stripe_bad.is_empty() || stripe_bad.len() > STRIPE_PARITY_SHARDS {
                continue;
            }
            let mut shards: Vec<Option<Vec<u8>>> = (start..end)
                .map(|i| {
                    blocks_vec[i].3.as_ref().map(|v| {
                        let mut b = v.clone();
                        b.resize(*max_len, 0);
                        b
                    })
                })
                .collect();
            while shards.len() < STRIPE_DATA_SHARDS {
                shards.push(Some(vec![0u8; *max_len]));
            }
            shards.truncate(STRIPE_DATA_SHARDS);
            for p in parity {
                shards.push(Some(p.clone()));
            }
            if shards.len() == STRIPE_DATA_SHARDS + STRIPE_PARITY_SHARDS {
                if decode_stripe(&mut shards, *max_len).is_ok() {
                    for (j, i) in (start..end).enumerate() {
                        if blocks_vec[i].3.is_none() {
                            let mut rec = shards[j].clone().unwrap_or_default();
                            rec.truncate(blocks_vec[i].2);
                            blocks_vec[i].3 = Some(rec);
                        }
                    }
                }
            }
        }
    }

    let blocks: std::collections::HashMap<[u8; 32], (u8, Vec<u8>)> = blocks_vec
        .into_iter()
        .filter_map(|(hash, algo, _len, data)| data.map(|d| (hash, (algo, d))))
        .collect();

    fs::create_dir_all(out_dir)?;
    let total_files = manifest.len() as u32;
    let mut count = 0u32;
    for (idx, entry) in manifest.iter().enumerate() {
        if let Some(ref cb) = progress {
            cb(Progress {
                phase: ProgressPhase::Compressing,
                files_done: idx as u32,
                files_total: total_files,
                current_path: Some(entry.path.clone()),
            });
        }
        let out_path = out_dir.join(&entry.path);
        if let Some(p) = out_path.parent() {
            fs::create_dir_all(p)?;
        }
        let file_data: Vec<u8> = if let Some(ref solid_ref) = entry.solid {
            let (_, (algo, stream_comp)) = blocks.iter().next().ok_or("No solid stream")?;
            let stream_raw = decompress(codec_from_id(*algo), stream_comp)?;
            let o = solid_ref.offset as usize;
            let l = solid_ref.length as usize;
            stream_raw[o..o + l].to_vec()
        } else if let Some(ref chunk_refs) = entry.chunks {
            let mut out_data = Vec::new();
            for cr in chunk_refs {
                let hash = hex_to_hash(&cr.hash_hex).unwrap_or([0u8; 32]);
                if let Some((_, comp)) = blocks.get(&hash) {
                    let dec = decompress(codec_from_id(cr.algo), comp)?;
                    out_data.extend_from_slice(&dec);
                }
            }
            out_data
        } else {
            let hash = hex_to_hash(&entry.hash_hex).unwrap_or([0u8; 32]);
            if let Some((_, comp)) = blocks.get(&hash) {
                decompress(codec_from_id(entry.algo), comp)?
            } else {
                continue;
            }
        };
        fs::write(&out_path, file_data)?;
        count += 1;
    }
    if let Some(ref cb) = progress {
        cb(Progress {
            phase: ProgressPhase::Done,
            files_done: total_files,
            files_total: total_files,
            current_path: None,
        });
    }
    Ok(count)
}

/// Распаковка `.oz` или распространённых форматов (ZIP, tar, gzip, xz, bzip2).
pub fn decompress_any_to_path(
    archive: &Path,
    out_dir: &Path,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    decompress_any_to_path_with_options(archive, out_dir, None, None)
}

pub fn decompress_any_to_path_with_password(
    archive: &Path,
    out_dir: &Path,
    password: Option<&str>,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    decompress_any_to_path_with_options(archive, out_dir, password, None)
}

pub fn decompress_any_to_path_with_options(
    archive: &Path,
    out_dir: &Path,
    password: Option<&str>,
    progress: Option<Arc<dyn Fn(Progress) + Send + Sync>>,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let mut f = fs::File::open(archive)?;
    let mut magic = [0u8; 8];
    let n = std::io::Read::read(&mut f, &mut magic)?;
    if n >= 8 && magic.starts_with(b"OMEGAZIP") {
        return decompress_to_path_with_options(archive, out_dir, password, progress);
    }
    crate::compat::extract_foreign_with_password(archive, out_dir, password)
}

/// Список путей в `.oz`, `.zip` или tar-архиве.
pub fn list_any_archive(archive: &Path) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    list_any_archive_with_password(archive, None)
}

/// Список файлов в архиве; пароль передаётся в 7-Zip (ZIP с паролем по-прежнему ограничен нативным ZIP).
pub fn list_any_archive_with_password(
    archive: &Path,
    password: Option<&str>,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut f = fs::File::open(archive)?;
    let mut magic = [0u8; 8];
    let n = std::io::Read::read(&mut f, &mut magic)?;
    if n >= 8 && magic.starts_with(b"OMEGAZIP") {
        return list_archive(archive);
    }
    crate::compat::list_foreign_with_password(archive, password)
}

/// Сжатие в `.oz`, `.zip`, `.tar.gz`/`.tgz` или `.7z` (7-Zip в PATH).
pub fn compress_dispatch(source: &Path, dest: &Path) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    if crate::compat::output_is_zip(dest) {
        crate::compat::compress_to_zip(source, dest)
    } else if crate::compat::output_is_tar_gz(dest) {
        crate::compat::compress_to_tar_gz(source, dest)
    } else if crate::compat::output_is_7z(dest) {
        crate::compat::compress_to_7z(source, dest, None)
    } else {
        compress_to_path(source, dest)
    }
}

/// Расширенное сжатие; для `.zip` — только deflate ZIP (без пресетов .oz).
pub fn compress_advanced_dispatch(
    source: &Path,
    dest: &Path,
    options: CompressOptions,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    if crate::compat::output_is_zip(dest) {
        if options.password.is_some() {
            return Err(
                "Для ZIP шифрование не поддерживается — сохраните как .oz или .7z (7-Zip) или уберите пароль.".into(),
            );
        }
        crate::compat::compress_to_zip(source, dest)
    } else if crate::compat::output_is_tar_gz(dest) {
        if options.password.is_some() {
            return Err("Для .tar.gz пароль не поддерживается — используйте .oz или .7z.".into());
        }
        crate::compat::compress_to_tar_gz(source, dest)
    } else if crate::compat::output_is_7z(dest) {
        crate::compat::compress_to_7z(source, dest, options.password.as_deref())
    } else {
        compress_to_path_with_options(source, dest, options)
    }
}

// ============== Экспорт в ZIP (совместимость) ==============

/// Экспортирует содержимое .oz в обычный ZIP (откроется в любом архиваторе).
pub fn export_to_zip(
    archive_oz: &Path,
    output_zip: &Path,
    password: Option<&str>,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let data = fs::read(archive_oz)?;
    if !data.starts_with(b"OMEGAZIP") {
        return Err("Invalid archive".into());
    }
    let version = data.get(8).copied().unwrap_or(1);
    let mut pos = 9usize;
    let (flags, salt) = if version >= 2 {
        let f = data.get(pos).copied().unwrap_or(0);
        pos += 1;
        let s = if (f & 2) != 0 {
            if data.len() < pos + 16 {
                return Err("Archive truncated".into());
            }
            let s: [u8; 16] = data[pos..pos + 16].try_into().unwrap();
            pos += 16;
            s
        } else {
            [0u8; 16]
        };
        (f, s)
    } else {
        (0, [0u8; 16])
    };
    let key: Option<[u8; 32]> = if (flags & 2) != 0 {
        Some(derive_key(
            password.ok_or("Password required for encrypted archive")?,
            &salt,
        ))
    } else {
        None
    };
    let manifest_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    let manifest: Vec<ManifestEntry> = serde_json::from_slice(&data[pos..pos + manifest_len])?;
    pos += manifest_len;
    let has_recovery_export = (flags & 8) != 0;
    let num_blocks_export = if has_recovery_export && pos + 4 <= data.len() {
        let n = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        n
    } else {
        usize::MAX
    };
    let block_header_export = if has_recovery_export { 41 } else { 37 };
    let mut blocks: std::collections::HashMap<[u8; 32], (u8, Vec<u8>)> = std::collections::HashMap::new();
    let mut blocks_read = 0usize;
    while blocks_read < num_blocks_export && pos + block_header_export <= data.len() {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&data[pos..pos + 32]);
        pos += 32;
        let algo = data[pos];
        pos += 1;
        let blen = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        if has_recovery_export {
            pos += 4;
        }
        if data.len() < pos + blen {
            break;
        }
        let raw = data[pos..pos + blen].to_vec();
        pos += blen;
        let block = key.as_ref().map(|k| decrypt_block(k, &raw).unwrap()).unwrap_or(raw);
        blocks.insert(hash, (algo, block));
        blocks_read += 1;
    }

    let file = fs::File::create(output_zip)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut count = 0u32;
    for entry in manifest {
        let file_data: Vec<u8> = if let Some(ref solid_ref) = entry.solid {
            let (_, (algo, stream_comp)) = blocks.iter().next().ok_or("No stream")?;
            let stream_raw = decompress(codec_from_id(*algo), stream_comp)?;
            let o = solid_ref.offset as usize;
            let l = solid_ref.length as usize;
            stream_raw[o..o + l].to_vec()
        } else if let Some(ref chunk_refs) = entry.chunks {
            let mut out_data = Vec::new();
            for cr in chunk_refs {
                let hash = hex_to_hash(&cr.hash_hex).unwrap_or([0u8; 32]);
                if let Some((_, comp)) = blocks.get(&hash) {
                    let dec = decompress(codec_from_id(cr.algo), comp)?;
                    out_data.extend_from_slice(&dec);
                }
            }
            out_data
        } else {
            let hash = hex_to_hash(&entry.hash_hex).unwrap_or([0u8; 32]);
            blocks.get(&hash).and_then(|(algo, comp)| decompress(codec_from_id(*algo), comp).ok()).unwrap_or_default()
        };
        zip.start_file(entry.path, options)?;
        zip.write_all(&file_data)?;
        count += 1;
    }
    zip.finish()?;
    Ok(count)
}

// ============== Информация и список файлов ==============

/// Информация об архиве (без чтения блоков).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ArchiveInfo {
    pub version: u8,
    pub flags: u8,
    pub file_count: u32,
    pub encrypted: bool,
    pub chunked: bool,
    pub solid: bool,
    pub recovery: bool,
}

/// Читает заголовок и манифест, возвращает информацию об архиве.
pub fn archive_info(archive: &Path) -> Result<ArchiveInfo, Box<dyn std::error::Error + Send + Sync>> {
    let data = fs::read(archive)?;
    if !data.starts_with(b"OMEGAZIP") {
        return Err("Invalid archive".into());
    }
    let version = data.get(8).copied().unwrap_or(1);
    let mut pos = 9usize;
    let flags = if version >= 2 {
        let f = data.get(pos).copied().unwrap_or(0);
        pos += 1;
        if (f & 2) != 0 {
            pos += 16;
        }
        f
    } else {
        0
    };
    if data.len() < pos + 4 {
        return Err("Archive truncated".into());
    }
    let manifest_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if data.len() < pos + manifest_len {
        return Err("Archive truncated".into());
    }
    let manifest: Vec<ManifestEntry> = serde_json::from_slice(&data[pos..pos + manifest_len])?;
    Ok(ArchiveInfo {
        version,
        flags,
        file_count: manifest.len() as u32,
        encrypted: (flags & 2) != 0,
        chunked: (flags & 1) != 0,
        solid: (flags & 4) != 0,
        recovery: (flags & 8) != 0,
    })
}

/// Список путей файлов в архиве (только манифест).
pub fn list_archive(archive: &Path) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let data = fs::read(archive)?;
    if !data.starts_with(b"OMEGAZIP") {
        return Err("Invalid archive".into());
    }
    let version = data.get(8).copied().unwrap_or(1);
    let mut pos = 9usize;
    if version >= 2 {
        let f = data.get(pos).copied().unwrap_or(0);
        pos += 1;
        if (f & 2) != 0 {
            pos += 16;
        }
    }
    if data.len() < pos + 4 {
        return Err("Archive truncated".into());
    }
    let manifest_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if data.len() < pos + manifest_len {
        return Err("Archive truncated".into());
    }
    let manifest: Vec<ManifestEntry> = serde_json::from_slice(&data[pos..pos + manifest_len])?;
    Ok(manifest.into_iter().map(|e| e.path).collect())
}

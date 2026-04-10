//! Конвейер архивации/разархивации: v1 (совместимость) и v2 (chunked, solid, шифрование, прогресс).

use std::io::Write;
use std::path::Path;
use std::str::FromStr;

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
use std::io::{self, BufReader, Read};
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

/// Манифест на диске: либо массив записей (legacy), либо объект с `solid_segments` для multi-segment solid.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ManifestFromDisk {
    Legacy(Vec<ManifestEntry>),
    Wrapped {
        files: Vec<ManifestEntry>,
        #[serde(default)]
        solid_segments: Option<Vec<ChunkRef>>,
    },
}

#[derive(serde::Serialize)]
struct ManifestWrapped<'a> {
    files: &'a [ManifestEntry],
    #[serde(skip_serializing_if = "Option::is_none")]
    solid_segments: Option<&'a [ChunkRef]>,
}

fn parse_manifest_json(slice: &[u8]) -> Result<(Vec<ManifestEntry>, Option<Vec<ChunkRef>>), serde_json::Error> {
    match serde_json::from_slice::<ManifestFromDisk>(slice)? {
        ManifestFromDisk::Legacy(files) => Ok((files, None)),
        ManifestFromDisk::Wrapped { files, solid_segments } => Ok((files, solid_segments)),
    }
}

/// Multi-segment solid (>1 сырого блока) сериализуется как объект; иначе — legacy-массив.
fn serialize_manifest_for_solid(
    files: &[ManifestEntry],
    solid_segments: Option<&[ChunkRef]>,
) -> Result<String, serde_json::Error> {
    if solid_segments.map(|s| s.len()).unwrap_or(0) > 1 {
        serde_json::to_string(&ManifestWrapped {
            files,
            solid_segments,
        })
    } else {
        serde_json::to_string(files)
    }
}

fn decompress_solid_stream(
    blocks: &std::collections::HashMap<[u8; 32], (u8, Vec<u8>)>,
    solid_segments: Option<&[ChunkRef]>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    match solid_segments {
        Some(segs) if !segs.is_empty() => {
            let mut out = Vec::new();
            for cr in segs {
                let h = hex_to_hash(&cr.hash_hex).ok_or("invalid solid segment hash")?;
                let (_, comp) = blocks
                    .get(&h)
                    .ok_or("missing solid segment block")?;
                let dec = decompress(codec_from_id(cr.algo), comp)?;
                out.extend_from_slice(&dec);
            }
            Ok(out)
        }
        _ => {
            let (_, (algo, stream_comp)) = blocks.iter().next().ok_or("no solid stream")?;
            decompress(codec_from_id(*algo), stream_comp).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })
        }
    }
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

fn read_u32_le<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

/// Заголовок .oz + только JSON манифеста (без чтения блоков в память целиком).
struct OzReadHeader {
    version: u8,
    flags: u8,
    salt: [u8; 16],
    manifest_bytes: Vec<u8>,
}

fn read_oz_header<R: Read>(r: &mut R) -> Result<OzReadHeader, Box<dyn std::error::Error + Send + Sync>> {
    let mut magic = [0u8; 8];
    r.read_exact(&mut magic)?;
    if &magic != b"OMEGAZIP" {
        return Err("Invalid archive".into());
    }
    let mut vb = [0u8; 1];
    r.read_exact(&mut vb)?;
    let version = vb[0];
    let flags;
    let mut salt = [0u8; 16];
    if version >= 2 {
        let mut fb = [0u8; 1];
        r.read_exact(&mut fb)?;
        flags = fb[0];
        if (flags & 2) != 0 {
            r.read_exact(&mut salt)?;
        }
    } else {
        flags = 0;
    }
    let manifest_len = read_u32_le(r)? as usize;
    let mut manifest_bytes = vec![0u8; manifest_len];
    r.read_exact(&mut manifest_bytes)?;
    Ok(OzReadHeader {
        version,
        flags,
        salt,
        manifest_bytes,
    })
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
    pub fn parse_name(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

impl FromStr for Preset {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fast" => Ok(Preset::Fast),
            "balanced" | "balance" => Ok(Preset::Balanced),
            "max" => Ok(Preset::Max),
            "ultra" | "maxcompression" | "maxcomp" => Ok(Preset::Ultra),
            _ => Err(()),
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
    /// Solid: макс. размер сырого сегмента в байтах (≥ 1 MiB при задании). Несколько сегментов → отдельный манифест v2.
    pub solid_block_size_bytes: Option<usize>,
    /// ZIP: пройти анализ/preprocess и выбирать Stored vs Deflate по размеру (макс. совместимость .zip).
    pub zip_analyzed: bool,
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
            solid_block_size_bytes: None,
            zip_analyzed: false,
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
        return compress_solid(SolidCompressPlan {
            files: &files,
            base: &base,
            output,
            password,
            progress,
            total,
            use_ultra,
            solid_block_size_bytes: options.solid_block_size_bytes,
        });
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
                algo: block_ref.algo,
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
        let num_stripes = block_payloads.len().div_ceil(STRIPE_DATA_SHARDS);
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
struct SolidCompressPlan<'a> {
    files: &'a [std::path::PathBuf],
    base: &'a Path,
    output: &'a Path,
    password: Option<&'a str>,
    progress: Option<Arc<dyn Fn(Progress) + Send + Sync>>,
    total: u32,
    use_ultra: bool,
    solid_block_size_bytes: Option<usize>,
}

/// Solid: один сжатый поток на все файлы (как 7-Zip). use_ultra = XZ-9.
fn compress_solid(plan: SolidCompressPlan<'_>) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    const MIN_SOLID_BLOCK: usize = 1024 * 1024;
    let SolidCompressPlan {
        files,
        base,
        output,
        password,
        progress,
        total,
        use_ultra,
        solid_block_size_bytes,
    } = plan;
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

    let raw_slices: Vec<(usize, usize)> = match solid_block_size_bytes {
        None => vec![(0, stream_raw.len())],
        Some(bs) => {
            let cap = bs.max(MIN_SOLID_BLOCK);
            let mut v = Vec::new();
            let mut off = 0usize;
            while off < stream_raw.len() {
                let end = (off + cap).min(stream_raw.len());
                v.push((off, end - off));
                off = end;
            }
            v
        }
    };
    let multi_solid = raw_slices.len() > 1;

    let mut solid_segments: Option<Vec<ChunkRef>> = None;
    let mut blocks_to_write: Vec<([u8; 32], u8, Vec<u8>)> = Vec::new();

    for (start, len) in &raw_slices {
        let seg = &stream_raw[*start..*start + *len];
        let compressed = if use_ultra {
            crate::codec_backend::max_ratio_ultra_encode(seg).map_err(std::io::Error::other)?
        } else {
            crate::codec_backend::balanced_encode(seg).map_err(std::io::Error::other)?
        };
        let algo_id = if use_ultra { 3u8 } else { 1u8 };
        let hash = BlockStore::block_hash(seg);
        if multi_solid {
            let hash_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
            solid_segments.get_or_insert_with(Vec::new).push(ChunkRef {
                hash_hex,
                algo: algo_id,
                len: seg.len() as u32,
            });
        }
        blocks_to_write.push((hash, algo_id, compressed));
    }

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
    let manifest_json = serialize_manifest_for_solid(&manifest, solid_segments.as_deref())?;
    out.write_all(&(manifest_json.len() as u32).to_le_bytes())?;
    out.write_all(manifest_json.as_bytes())?;

    let enc_key = password.map(|p| derive_key(p, &salt));
    for (hash, algo_id, compressed) in blocks_to_write {
        let payload = if let Some(ref k) = enc_key {
            encrypt_block(k, &compressed)
        } else {
            compressed
        };
        out.write_all(&hash)?;
        out.write_all(&[algo_id])?;
        out.write_all(&(payload.len() as u32).to_le_bytes())?;
        out.write_all(&payload)?;
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
    let f = fs::File::open(archive)?;
    let mut r = BufReader::new(f);
    let OzReadHeader {
        flags,
        salt,
        manifest_bytes,
        ..
    } = read_oz_header(&mut r)?;

    let encrypted = (flags & 2) != 0;
    let key = if encrypted {
        Some(derive_key(
            password.ok_or("Encrypted archive requires password")?,
            &salt,
        ))
    } else {
        None
    };

    let (manifest, solid_segments_meta) = parse_manifest_json(&manifest_bytes)?;

    let has_recovery = (flags & 8) != 0;
    let num_blocks = if has_recovery {
        read_u32_le(&mut r)? as usize
    } else {
        usize::MAX
    };
    type BlockRow = ([u8; 32], u8, usize, Option<Vec<u8>>);
    let mut blocks_vec: Vec<BlockRow> = Vec::new();
    let mut bad_indices: Vec<usize> = Vec::new();
    let mut block_count = 0usize;

    loop {
        if has_recovery && block_count >= num_blocks {
            break;
        }
        let mut hash = [0u8; 32];
        match r.read_exact(&mut hash) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                if has_recovery {
                    return Err("Archive truncated (blocks)".into());
                }
                break;
            }
            Err(e) => return Err(e.into()),
        }
        if !has_recovery && block_count > 0 {
            // allow EOF only before a new block header
        }
        let mut algo_b = [0u8; 1];
        r.read_exact(&mut algo_b)?;
        let algo = algo_b[0];
        let blen = read_u32_le(&mut r)? as usize;
        let stored_crc = if has_recovery {
            Some(read_u32_le(&mut r)?)
        } else {
            None
        };
        let mut raw = vec![0u8; blen];
        r.read_exact(&mut raw)?;
        let block = if let Some(ref k) = key {
            decrypt_block(k, &raw).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?
        } else {
            raw
        };
        let ok = stored_crc.is_none_or(|c| crc32_bytes(&block) == c);
        if ok {
            blocks_vec.push((hash, algo, blen, Some(block)));
        } else {
            blocks_vec.push((hash, algo, blen, None));
            bad_indices.push(blocks_vec.len() - 1);
        }
        block_count += 1;
    }

    if has_recovery && !bad_indices.is_empty() {
        let mut ns_buf = [0u8; 4];
        if r.read_exact(&mut ns_buf).is_ok() {
            let num_stripes = u32::from_le_bytes(ns_buf) as usize;
            let mut parity_data: Vec<(usize, Vec<Vec<u8>>)> = Vec::new();
            for _ in 0..num_stripes {
                let max_len = match read_u32_le(&mut r) {
                    Ok(v) => v as usize,
                    Err(_) => break,
                };
                let mut p0 = vec![0u8; max_len];
                if r.read_exact(&mut p0).is_err() {
                    break;
                }
                let mut p1 = vec![0u8; max_len];
                if r.read_exact(&mut p1).is_err() {
                    break;
                }
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
                if shards.len() == STRIPE_DATA_SHARDS + STRIPE_PARITY_SHARDS
                    && decode_stripe(&mut shards, *max_len).is_ok()
                {
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
    let has_solid_entries = manifest.iter().any(|e| e.solid.is_some());
    let solid_stream_raw: Option<Vec<u8>> = if has_solid_entries {
        Some(decompress_solid_stream(
            &blocks,
            solid_segments_meta.as_deref(),
        )?)
    } else {
        None
    };
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
            let stream_raw = solid_stream_raw.as_ref().ok_or("No solid stream")?;
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
        if options.zip_analyzed {
            return crate::compat::compress_to_zip_analyzed(source, dest);
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
    let f_in = fs::File::open(archive_oz)?;
    let mut r = BufReader::new(f_in);
    let h = read_oz_header(&mut r)?;
    let flags = h.flags;
    let salt = h.salt;
    let key = if (flags & 2) != 0 {
        Some(derive_key(
            password.ok_or("Password required for encrypted archive")?,
            &salt,
        ))
    } else {
        None
    };
    let (manifest, solid_segments_export) = parse_manifest_json(&h.manifest_bytes)?;
    let has_recovery_export = (flags & 8) != 0;
    let num_blocks_export = if has_recovery_export {
        read_u32_le(&mut r)? as usize
    } else {
        usize::MAX
    };
    let mut blocks: std::collections::HashMap<[u8; 32], (u8, Vec<u8>)> = std::collections::HashMap::new();
    let mut blocks_read = 0usize;
    loop {
        if has_recovery_export && blocks_read >= num_blocks_export {
            break;
        }
        let mut hash = [0u8; 32];
        match r.read_exact(&mut hash) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
        let mut algo_b = [0u8; 1];
        r.read_exact(&mut algo_b)?;
        let algo = algo_b[0];
        let blen = read_u32_le(&mut r)? as usize;
        if has_recovery_export {
            let _crc = read_u32_le(&mut r)?;
        }
        let mut raw = vec![0u8; blen];
        r.read_exact(&mut raw)?;
        let block = key
            .as_ref()
            .map(|k| decrypt_block(k, &raw).unwrap())
            .unwrap_or(raw);
        blocks.insert(hash, (algo, block));
        blocks_read += 1;
    }

    let file = fs::File::create(output_zip)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut count = 0u32;
    for entry in manifest {
        let file_data: Vec<u8> = if let Some(ref solid_ref) = entry.solid {
            let stream_raw = decompress_solid_stream(&blocks, solid_segments_export.as_deref())?;
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
    let f = fs::File::open(archive)?;
    let mut r = BufReader::new(f);
    let h = read_oz_header(&mut r)?;
    let (manifest, _) = parse_manifest_json(&h.manifest_bytes)?;
    let flags = h.flags;
    Ok(ArchiveInfo {
        version: h.version,
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
    let f = fs::File::open(archive)?;
    let mut r = BufReader::new(f);
    let h = read_oz_header(&mut r)?;
    let (manifest, _) = parse_manifest_json(&h.manifest_bytes)?;
    Ok(manifest.into_iter().map(|e| e.path).collect())
}

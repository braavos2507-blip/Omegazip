//! Репозиторий бэкапов: chunk store + снапшоты (как Borg/restic).

use crate::chunked::{chunks, DEFAULT_CHUNK_SIZE};
use crate::codec::{best_compress, decompress, codec_id, codec_from_id, Codec};
use crate::dedup::BlockStore;
use crate::{analyze_bytes, preprocess, read_preprocess_result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const CHUNK_PREFIX_LEN: usize = 4;

fn chunk_path(repo: &Path, hash_hex: &str) -> PathBuf {
    let a = &hash_hex[..2];
    let b = &hash_hex[2..CHUNK_PREFIX_LEN];
    repo.join("chunks").join(a).join(b).join(hash_hex)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SnapshotFile {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    chunks: Option<Vec<ChunkRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hash_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    algo: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    len: Option<u32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ChunkRef {
    hash_hex: String,
    algo: u8,
    len: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Snapshot {
    id: u64,
    files: Vec<SnapshotFile>,
}

/// Создаёт пустой репозиторий в path.
pub fn repo_init(path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    fs::create_dir_all(path.join("chunks"))?;
    fs::create_dir_all(path.join("snapshots"))?;
    Ok(())
}

/// Делает бэкап source в репозиторий. Возвращает id снапшота.
pub fn backup(
    repo_path: &Path,
    source: &Path,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let snapshots_dir = repo_path.join("snapshots");
    let existing = fs::read_dir(&snapshots_dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.strip_suffix(".json").and_then(|s| s.strip_prefix("snapshot_"))
                .and_then(|s| s.parse::<u64>().ok())
        })
        .max()
        .unwrap_or(0);
    let snapshot_id = existing + 1;

    let files: Vec<PathBuf> = if source.is_dir() {
        WalkDir::new(source)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    } else {
        vec![source.to_path_buf()]
    };
    let base = source.canonicalize().unwrap_or_else(|_| source.to_path_buf());
    let base = if base.is_dir() {
        base
    } else {
        base.parent().unwrap_or(&base).to_path_buf()
    };

    let mut snapshot_files: Vec<SnapshotFile> = Vec::new();
    let chunk_size = DEFAULT_CHUNK_SIZE;

    for path in &files {
        let rel = path.strip_prefix(&base).unwrap_or(path);
        let rel_str = rel.to_string_lossy().to_string();
        let data = fs::read(path)?;
        let analysis = analyze_bytes(&data, Some(path))?;
        let pre = preprocess(path, &data)?;
        let data_ready = read_preprocess_result(pre)?;

        let chunk_refs: Vec<ChunkRef> = chunks(&data_ready, chunk_size)
            .map(|chunk| -> Result<ChunkRef, Box<dyn std::error::Error + Send + Sync>> {
                let (codec, compressed) = best_compress(chunk, analysis.context).unwrap_or_else(|_| (Codec::Store, chunk.to_vec()));
                let algo_id = codec_id(&codec);
                let hash = BlockStore::block_hash(chunk);
                let hash_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
                let chunk_file = chunk_path(repo_path, &hash_hex);
                if !chunk_file.exists() {
                    if let Some(p) = chunk_file.parent() {
                        fs::create_dir_all(p)?;
                    }
                    fs::write(&chunk_file, &compressed)?;
                }
                Ok(ChunkRef {
                    hash_hex,
                    algo: algo_id,
                    len: chunk.len() as u32,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        snapshot_files.push(SnapshotFile {
            path: rel_str,
            chunks: Some(chunk_refs),
            hash_hex: None,
            algo: None,
            len: None,
        });
    }

    let snapshot = Snapshot {
        id: snapshot_id,
        files: snapshot_files,
    };
    let path = snapshots_dir.join(format!("snapshot_{}.json", snapshot_id));
    fs::write(&path, serde_json::to_string_pretty(&snapshot)?)?;
    Ok(snapshot_id)
}

/// Восстанавливает снапшот в dest_dir.
pub fn restore(
    repo_path: &Path,
    snapshot_id: u64,
    dest_dir: &Path,
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let path = repo_path.join("snapshots").join(format!("snapshot_{}.json", snapshot_id));
    let data = fs::read(&path)?;
    let snapshot: Snapshot = serde_json::from_slice(&data)?;
    fs::create_dir_all(dest_dir)?;
    let mut count = 0u32;
    for file in &snapshot.files {
        let out_path = dest_dir.join(&file.path);
        if let Some(p) = out_path.parent() {
            fs::create_dir_all(p)?;
        }
        let file_data = if let Some(ref refs) = file.chunks {
            let mut out = Vec::new();
            for cr in refs {
                let chunk_file = chunk_path(repo_path, &cr.hash_hex);
                let compressed = fs::read(&chunk_file)?;
                let dec = decompress(codec_from_id(cr.algo), &compressed)?;
                out.extend_from_slice(&dec);
            }
            out
        } else {
            return Err("Snapshot without chunks not supported".into());
        };
        fs::write(&out_path, file_data)?;
        count += 1;
    }
    Ok(count)
}

/// Список id снапшотов в репозитории.
pub fn list_snapshots(repo_path: &Path) -> Result<Vec<u64>, Box<dyn std::error::Error + Send + Sync>> {
    let snapshots_dir = repo_path.join("snapshots");
    if !snapshots_dir.exists() {
        return Ok(Vec::new());
    }
    let mut ids: Vec<u64> = fs::read_dir(&snapshots_dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.strip_suffix(".json").and_then(|s| s.strip_prefix("snapshot_"))
                .and_then(|s| s.parse::<u64>().ok())
        })
        .collect();
    ids.sort();
    Ok(ids)
}

/// Синхронизирует репозиторий в destination (для бэкапа в облако: rclone mount, Synced папка и т.д.).
/// Копирует chunks/ и snapshots/ в destination, создавая destination при необходимости.
pub fn repo_push(
    repo_path: &Path,
    destination: &Path,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::{Read, Write};
    fs::create_dir_all(destination)?;
    let chunks_src = repo_path.join("chunks");
    let snapshots_src = repo_path.join("snapshots");
    let chunks_dst = destination.join("chunks");
    let snapshots_dst = destination.join("snapshots");
    fs::create_dir_all(&chunks_dst)?;
    fs::create_dir_all(&snapshots_dst)?;
    let mut files_copied: u64 = 0;

    fn copy_dir(
        src: &Path,
        dst: &Path,
        count: &mut u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for e in fs::read_dir(src)? {
            let e = e?;
            let name = e.file_name();
            let src_p = src.join(&name);
            let dst_p = dst.join(&name);
            if e.file_type()?.is_dir() {
                fs::create_dir_all(&dst_p)?;
                copy_dir(&src_p, &dst_p, count)?;
            } else {
                if let Some(p) = dst_p.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut f = fs::File::open(&src_p)?;
                let mut out = fs::File::create(&dst_p)?;
                let mut buf = [0u8; 65536];
                loop {
                    let n = f.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    out.write_all(&buf[..n])?;
                }
                *count += 1;
            }
        }
        Ok(())
    }

    if chunks_src.exists() {
        copy_dir(&chunks_src, &chunks_dst, &mut files_copied)?;
    }
    if snapshots_src.exists() {
        copy_dir(&snapshots_src, &snapshots_dst, &mut files_copied)?;
    }
    Ok(files_copied)
}

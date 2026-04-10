//! Семантический анализ: тип данных по содержимому (скрипты + энтропия + magic).

use crate::magic;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::fs::File;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataContext {
    Text,
    Binary,
    Realtime,
    /// Контейнеры видео/аудио с кодеками (H.264, AAC, Opus…): архиватором почти не сжимаются.
    Media,
    Archive,
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub context: DataContext,
    pub mime: Option<String>,
    pub is_pdf: bool,
    pub entropy: f64,
    pub text_ratio: f64,
}

fn entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut counts = [0u64; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let len = data.len() as f64;
    let mut h = 0.0f64;
    for &c in &counts {
        if c > 0 {
            let p = c as f64 / len;
            h -= p * p.log2();
        }
    }
    h / 8.0
}

fn text_ratio(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let printable = data
        .iter()
        .filter(|&&b| (32..127).contains(&b) || b == b'\n' || b == b'\r' || b == b'\t')
        .count();
    printable as f64 / data.len() as f64
}

fn script_mime(path: &Path) -> Option<String> {
    let cmd = std::env::var("OMEGAZIP_MIME_SCRIPT").ok()?;
    let out = Command::new("sh").args(["-c", &cmd]).arg(path).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Расширения файлов, которые обычно уже сжаты кодеком (не путать с «сырым» PCM/WAV).
fn media_extension(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|ext| {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "mp4" | "m4v" | "mov" | "mkv" | "webm" | "avi" | "wmv" | "flv" | "mpg" | "mpeg" | "3gp"
                | "ts" | "mts" | "m2ts" | "mp3" | "aac" | "m4a" | "opus" | "ogg" | "ogv"
        )
    })
}

fn script_context(path: &Path) -> Option<DataContext> {
    let script = std::env::var("OMEGAZIP_ANALYZER_SCRIPT").ok()?;
    let out = Command::new("sh").args(["-c", &script]).arg(path).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let out_s = String::from_utf8_lossy(&out.stdout).into_owned();
    let s = out_s.trim();
    Some(match s {
        "text" => DataContext::Text,
        "binary" => DataContext::Binary,
        "realtime" => DataContext::Realtime,
        "media" => DataContext::Media,
        _ => DataContext::Archive,
    })
}

pub fn analyze_file(path: &Path) -> std::io::Result<AnalysisResult> {
    let mut f = File::open(path)?;
    let mut buf = vec![0u8; 64 * 1024];
    let n = f.read(&mut buf)?;
    buf.truncate(n);
    analyze_bytes(&buf, Some(path))
}

pub fn analyze_bytes(data: &[u8], path: Option<&Path>) -> std::io::Result<AnalysisResult> {
    let mime_from_magic = magic::mime_from_magic(data);
    let mime_from_script = path.and_then(script_mime);
    let mime = mime_from_script.or(mime_from_magic);

    let is_pdf = mime
        .as_deref()
        .map(|m| m == "application/pdf")
        .unwrap_or_else(|| data.starts_with(b"%PDF"));

    let entropy = entropy(data);
    let text_ratio = text_ratio(data);

    let context = path
        .and_then(script_context)
        .unwrap_or_else(|| {
            if path.is_some_and(media_extension) {
                DataContext::Media
            } else {
                choose_context(entropy, text_ratio, &mime, is_pdf)
            }
        });

    Ok(AnalysisResult {
        context,
        mime,
        is_pdf,
        entropy,
        text_ratio,
    })
}

fn choose_context(
    entropy: f64,
    text_ratio: f64,
    mime: &Option<String>,
    is_pdf: bool,
) -> DataContext {
    if is_pdf {
        return DataContext::Archive;
    }
    if let Some(m) = mime {
        if m.starts_with("text/") || m == "application/json" || m == "application/xml" {
            return DataContext::Text;
        }
        if m.starts_with("video/")
            || m == "audio/mpeg"
            || m == "audio/mp4"
            || m == "audio/aac"
            || m == "application/ogg"
            || m == "audio/opus"
        {
            return DataContext::Media;
        }
        if m == "application/octet-stream" && entropy > 0.9 {
            return DataContext::Realtime;
        }
    }
    if text_ratio > 0.85 && entropy < 0.9 {
        return DataContext::Text;
    }
    if entropy > 0.95 {
        return DataContext::Realtime;
    }
    if text_ratio > 0.5 {
        return DataContext::Binary;
    }
    DataContext::Archive
}

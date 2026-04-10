//! Эвристики выбора пресета .oz по расширениям (SMART-01 / SMART-02).
//! Не меняет формат архива; только chunk_size / solid / recovery через `Preset`.
//!
//! Поведение «как в Finder Services» (см. `scripts/install-context-menu.sh`): переменные
//! `OMEGAZIP_CONTEXT_PRESET`, `OMEGAZIP_AUTO_UPGRADE_FOLDER_MB` и файлы
//! `~/.config/omegazip/context_preset`, `auto_upgrade_folder_mb`.

use std::path::{Path, PathBuf};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::Instant;

use walkdir::WalkDir;

use crate::analyze_bytes;
use crate::chunked::chunks;
use crate::codec::best_compress;
use crate::codec_backend;
use crate::dedup::BlockStore;
use crate::pipeline::Preset;

/// Максимум файлов для обхода при анализе папки (производительность).
pub const DIRECTORY_SAMPLE_CAP: usize = 400;

/// Максимальная глубина обхода от корня папки.
pub const DIRECTORY_MAX_DEPTH: usize = 32;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompressPresetHint {
    /// Имя для `Preset::from_str`: `fast`, `balanced`, …
    pub preset: String,
    /// Краткое пояснение для UI (RU).
    pub reason: String,
}

fn extension_lower(path: &Path) -> Option<String> {
    path.extension()?.to_str().map(|s| s.to_lowercase())
}

/// Код, текст, lossless-изображения → сильнее сжимать (balanced), без потерь.
fn is_lossless_or_text_ext(ext: &str) -> bool {
    matches!(
        ext,
        // исходники и разметка
        "rs" | "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hxx" | "go" | "py" | "pyw" | "js"
            | "mjs" | "cjs" | "ts" | "tsx" | "jsx" | "java" | "kt" | "kts" | "swift" | "rb" | "php"
            | "cs" | "fs" | "fsx" | "scala" | "clj" | "cljs" | "ex" | "exs" | "erl" | "hrl" | "lua"
            | "r" | "m" | "mm" | "zig" | "nim" | "v" | "sv" | "vh" | "svh" | "dart" | "vue"
            | "svelte"
            // конфиги и текст
            | "md" | "txt" | "text" | "rst" | "adoc" | "json" | "yaml" | "yml" | "toml" | "xml"
            | "html" | "htm" | "xhtml" | "css" | "scss" | "sass" | "less" | "sql" | "sh" | "bash"
            | "epub" | "fb2" | "mobi" | "azw" | "azw3"
            | "zsh" | "fish" | "ps1" | "bat" | "cmd" | "ini" | "cfg" | "conf" | "properties"
            | "env" | "gitignore" | "editorconfig" | "lock"
            // lossless-изображения (SMART-02)
            | "png" | "apng" | "tiff" | "tif" | "bmp" | "dib" | "exr" | "hdr" | "dds" | "tga" | "pcx"
    )
}

/// Уже сжатые медиа и контейнеры → минимум CPU (fast).
fn is_already_compressed_ext(ext: &str) -> bool {
    matches!(
        ext,
        // видео / аудио
        "mp4" | "m4v" | "mpeg" | "mpg" | "avi" | "mov" | "mkv" | "webm" | "flv" | "wmv" | "3gp"
            | "mp3" | "m4a" | "aac" | "ogg" | "oga" | "opus" | "flac" | "wma" | "aiff" | "aif"
            // lossy / часто сжатые растры
            | "jpg" | "jpeg" | "jfif" | "heic" | "heif" | "webp" | "gif" | "jxl"
            // архивы и потоки
            | "zip" | "7z" | "rar" | "gz" | "tgz" | "bz2" | "tbz2" | "tbz" | "xz" | "txz" | "zst"
            | "tzst" | "cab" | "br" | "lz4" | "sz" | "oz"
            | "wasm"
            // образы дисков (как один большой бинарник)
            | "iso" | "img" | "vmdk" | "vdi" | "qcow2" | "qcow" | "dmg" | "bin"
    )
}

fn preset_label(p: Preset) -> &'static str {
    match p {
        Preset::Fast => "fast",
        Preset::Balanced => "balanced",
        Preset::Max => "max",
        Preset::Ultra => "ultra",
    }
}

fn reason_for(p: Preset) -> &'static str {
    match p {
        Preset::Fast => {
            "В основном медиа, архивы или образы — профиль «Быстрее», меньше нагрузка на CPU."
        }
        Preset::Balanced => {
            "Код, текст, lossless-изображения или смешанный набор — «Сбалансированно» (без потерь)."
        }
        Preset::Max | Preset::Ultra => {
            "Рекомендуется только при явном выборе — для авто не используется."
        }
    }
}

/// Пресет для одного файла по расширению.
pub fn preset_for_extension(ext: Option<&str>) -> Preset {
    let Some(ext) = ext.map(|s| s.to_lowercase()) else {
        return Preset::Balanced;
    };
    if is_lossless_or_text_ext(ext.as_str()) {
        Preset::Balanced
    } else if is_already_compressed_ext(ext.as_str()) {
        Preset::Fast
    } else {
        Preset::Balanced
    }
}

/// Эвристика для файла или каталога.
pub fn suggested_preset_for_path(path: &Path) -> Preset {
    if path.is_file() {
        return preset_for_extension(extension_lower(path).as_deref());
    }
    if path.is_dir() {
        return suggested_preset_for_directory(path);
    }
    Preset::Balanced
}

fn suggested_preset_for_directory(dir: &Path) -> Preset {
    let mut n_files: usize = 0;
    let mut all_compressed = true;

    for entry in WalkDir::new(dir)
        .max_depth(DIRECTORY_MAX_DEPTH)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if n_files >= DIRECTORY_SAMPLE_CAP {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        n_files += 1;
        let Some(ext) = extension_lower(entry.path()) else {
            all_compressed = false;
            continue;
        };
        if is_lossless_or_text_ext(ext.as_str()) {
            return Preset::Balanced;
        }
        if !is_already_compressed_ext(ext.as_str()) {
            all_compressed = false;
        }
    }

    if n_files == 0 {
        Preset::Balanced
    } else if all_compressed {
        Preset::Fast
    } else {
        Preset::Balanced
    }
}

/// Подсказка для GUI / CLI.
pub fn suggest_compress_preset_hint(path: &Path) -> CompressPresetHint {
    let p = suggested_preset_for_path(path);
    CompressPresetHint {
        preset: preset_label(p).to_string(),
        reason: reason_for(p).to_string(),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompetitivePresetPlan {
    pub preset: String,
    pub chunked: bool,
    pub reason: String,
}

/// Быстрый dry-run анализ для "competitive" режима:
/// выбирает между chunked+balanced и не-chunked режимом на основе
/// вероятности дедупа и типа данных (сжатые медиа/архивы против текста/кода).
pub fn suggest_competitive_plan(path: &Path) -> CompetitivePresetPlan {
    if path.is_file() {
        let ext = extension_lower(path).unwrap_or_default();
        if is_already_compressed_ext(&ext) {
            return CompetitivePresetPlan {
                preset: "fast".to_string(),
                chunked: false,
                reason: "Одиночный уже-сжатый файл: fast без chunked (минимум накладных расходов).".to_string(),
            };
        }
        return CompetitivePresetPlan {
            preset: "balanced".to_string(),
            chunked: false,
            reason: "Одиночный файл: balanced без chunked (дедуп обычно не окупается).".to_string(),
        };
    }

    if !path.is_dir() {
        return CompetitivePresetPlan {
            preset: "balanced".to_string(),
            chunked: true,
            reason: "Путь не распознан как файл/папка: conservative fallback (chunked+balanced).".to_string(),
        };
    }

    if let Some(trial_plan) = suggest_competitive_plan_by_trial(path) {
        return trial_plan;
    }

    let mut sampled_files = 0usize;
    let mut compressed_like = 0usize;
    let mut small_files = 0usize;
    let mut duplicate_prefixes = 0usize;
    let mut prefix_seen = std::collections::HashSet::<u64>::new();
    let mut sampled_bytes = 0u64;

    for entry in WalkDir::new(path)
        .max_depth(DIRECTORY_MAX_DEPTH)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if sampled_files >= DIRECTORY_SAMPLE_CAP {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        sampled_files += 1;
        let p = entry.path();
        let meta_len = entry.metadata().map(|m| m.len()).unwrap_or(0);
        sampled_bytes += meta_len;
        if meta_len <= 4 * 1024 {
            small_files += 1;
        }
        if let Some(ext) = extension_lower(p) {
            if is_already_compressed_ext(&ext) {
                compressed_like += 1;
            }
        }
        if let Ok(data) = std::fs::read(p) {
            let prefix = &data[..data.len().min(64 * 1024)];
            let mut h = DefaultHasher::new();
            prefix.hash(&mut h);
            let hv = h.finish();
            if !prefix_seen.insert(hv) {
                duplicate_prefixes += 1;
            }
        }
    }

    if sampled_files == 0 {
        return CompetitivePresetPlan {
            preset: "balanced".to_string(),
            chunked: true,
            reason: "Пустая папка: fallback chunked+balanced.".to_string(),
        };
    }

    let dup_ratio = duplicate_prefixes as f64 / sampled_files as f64;
    let compressed_ratio = compressed_like as f64 / sampled_files as f64;
    let small_ratio = small_files as f64 / sampled_files as f64;

    if compressed_ratio >= 0.55 || small_ratio >= 0.70 {
        return CompetitivePresetPlan {
            preset: "fast".to_string(),
            chunked: false,
            reason: format!(
                "Много уже-сжатых/мелких файлов (compressed {:.0}%, small {:.0}%): fast без chunked.",
                compressed_ratio * 100.0,
                small_ratio * 100.0
            ),
        };
    }

    // Для mixed-наборов с умеренной долей уже-сжатых файлов chunked часто даёт накладные расходы.
    if dup_ratio < 0.10 && compressed_ratio >= 0.25 {
        return CompetitivePresetPlan {
            preset: "balanced".to_string(),
            chunked: false,
            reason: format!(
                "Низкая похожесть ({:.0}%) и заметная доля сжатых файлов ({:.0}%): balanced без chunked.",
                dup_ratio * 100.0,
                compressed_ratio * 100.0
            ),
        };
    }

    if dup_ratio >= 0.12 {
        return CompetitivePresetPlan {
            preset: "balanced".to_string(),
            chunked: true,
            reason: format!(
                "Высокая похожесть данных ({:.0}% дублей префиксов): chunked+balanced.",
                dup_ratio * 100.0
            ),
        };
    }

    if sampled_bytes < 8 * 1024 * 1024 && dup_ratio < 0.03 {
        return CompetitivePresetPlan {
            preset: "balanced".to_string(),
            chunked: false,
            reason: "Небольшой объём и низкая похожесть: balanced без chunked.".to_string(),
        };
    }

    CompetitivePresetPlan {
        preset: "balanced".to_string(),
        chunked: true,
        reason: format!(
            "Смешанный набор (compressed {:.0}%, дубли {:.0}%): balanced+chunked.",
            compressed_ratio * 100.0,
            dup_ratio * 100.0
        ),
    }
}

#[derive(Clone, Copy)]
enum CompetitiveCandidate {
    Balanced,
    ChunkedBalanced,
    MaxSolid,
}

impl CompetitiveCandidate {
    fn label(self) -> &'static str {
        match self {
            CompetitiveCandidate::Balanced => "balanced",
            CompetitiveCandidate::ChunkedBalanced => "balanced+chunked",
            CompetitiveCandidate::MaxSolid => "max(solid)",
        }
    }
}

fn collect_trial_samples(path: &Path) -> Vec<Vec<u8>> {
    const SAMPLE_FILES_CAP: usize = 72;
    const SAMPLE_FILE_BYTES_CAP: usize = 256 * 1024;
    const SAMPLE_TOTAL_BYTES_CAP: usize = 24 * 1024 * 1024;

    let mut out = Vec::new();
    let mut total = 0usize;
    for entry in WalkDir::new(path)
        .max_depth(DIRECTORY_MAX_DEPTH)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if out.len() >= SAMPLE_FILES_CAP || total >= SAMPLE_TOTAL_BYTES_CAP {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        if let Ok(mut data) = std::fs::read(entry.path()) {
            if data.is_empty() {
                continue;
            }
            if data.len() > SAMPLE_FILE_BYTES_CAP {
                data.truncate(SAMPLE_FILE_BYTES_CAP);
            }
            total += data.len();
            out.push(data);
        }
    }
    out
}

fn estimate_candidate(candidate: CompetitiveCandidate, samples: &[Vec<u8>]) -> Option<(usize, f64)> {
    let t0 = Instant::now();
    let est = match candidate {
        CompetitiveCandidate::Balanced => {
            let mut total = 0usize;
            for data in samples {
                let analysis = analyze_bytes(data, None).ok()?;
                let (_codec, c) = best_compress(data, analysis.context).ok()?;
                total += c.len();
            }
            total
        }
        CompetitiveCandidate::ChunkedBalanced => {
            let mut total = 0usize;
            let mut seen = HashSet::<[u8; 32]>::new();
            for data in samples {
                let analysis = analyze_bytes(data, None).ok()?;
                for ch in chunks(data, crate::DEFAULT_CHUNK_SIZE) {
                    let h = BlockStore::block_hash(ch);
                    if !seen.insert(h) {
                        continue;
                    }
                    let (_codec, c) = best_compress(ch, analysis.context).ok()?;
                    total += c.len();
                }
            }
            total
        }
        CompetitiveCandidate::MaxSolid => {
            let mut joined = Vec::new();
            for data in samples {
                joined.extend_from_slice(data);
            }
            codec_backend::balanced_encode(&joined).ok()?.len()
        }
    };
    Some((est, t0.elapsed().as_secs_f64()))
}

fn suggest_competitive_plan_by_trial(path: &Path) -> Option<CompetitivePresetPlan> {
    let samples = collect_trial_samples(path);
    if samples.len() < 20 {
        return None;
    }

    let candidates = [
        CompetitiveCandidate::Balanced,
        CompetitiveCandidate::ChunkedBalanced,
        CompetitiveCandidate::MaxSolid,
    ];

    let mut measured: Vec<(CompetitiveCandidate, usize, f64)> = Vec::new();
    for c in candidates {
        let (size, t) = estimate_candidate(c, &samples)?;
        measured.push((c, size, t));
    }
    measured.sort_by_key(|(_, sz, _)| *sz);
    let (best_c, best_sz, best_t) = measured[0];

    // Если размер почти одинаковый (<=3%), предпочесть более быстрый пресет.
    let mut selected = (best_c, best_sz, best_t);
    for (c, sz, t) in &measured {
        if (*sz as f64) <= (best_sz as f64) * 1.03 && *t < selected.2 {
            selected = (*c, *sz, *t);
        }
    }

    let (preset, chunked) = match selected.0 {
        CompetitiveCandidate::Balanced => ("balanced".to_string(), false),
        CompetitiveCandidate::ChunkedBalanced => ("balanced".to_string(), true),
        CompetitiveCandidate::MaxSolid => ("max".to_string(), false),
    };
    let reason = format!(
        "trial: winner={} est_size={}B est_time={:.3}s (samples: {} files)",
        selected.0.label(),
        selected.1,
        selected.2,
        samples.len()
    );
    Some(CompetitivePresetPlan {
        preset,
        chunked,
        reason,
    })
}

fn omegazip_config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("omegazip")
}

fn read_first_config_line(path: &Path) -> Option<String> {
    let s = std::fs::read_to_string(path).ok()?;
    for line in s.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        return t.split_whitespace().next().map(|s| s.to_string());
    }
    None
}

fn load_context_preset_str() -> String {
    if let Ok(v) = std::env::var("OMEGAZIP_CONTEXT_PRESET") {
        if !v.trim().is_empty() {
            return v.trim().to_lowercase();
        }
    }
    let path = omegazip_config_dir().join("context_preset");
    if let Some(line) = read_first_config_line(&path) {
        return line.to_lowercase();
    }
    "auto".to_string()
}

fn load_auto_upgrade_folder_mb() -> u64 {
    match std::env::var("OMEGAZIP_AUTO_UPGRADE_FOLDER_MB") {
        Ok(v) if !v.trim().is_empty() => v.trim().parse().unwrap_or(200),
        _ => {
            let path = omegazip_config_dir().join("auto_upgrade_folder_mb");
            read_first_config_line(&path)
                .and_then(|s| s.parse().ok())
                .unwrap_or(200)
        }
    }
}

/// Суммарный размер файлов под `path` (как ориентир для `du`), до `DIRECTORY_MAX_DEPTH`.
fn folder_disk_usage_bytes(path: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    for entry in WalkDir::new(path)
        .max_depth(DIRECTORY_MAX_DEPTH)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    Ok(total)
}

fn folder_bumped_to_max(path: &Path, auto_mb: u64) -> bool {
    if auto_mb == 0 || !path.is_dir() {
        return false;
    }
    match folder_disk_usage_bytes(path) {
        Ok(bytes) => bytes / (1024 * 1024) >= auto_mb,
        Err(_) => false,
    }
}

/// Та же семантика, что `load_context_preset` + `bump_preset_for_large_folder` в
/// `install-context-menu.sh`, затем ветка `compress --preset auto` для профиля auto.
pub(crate) fn effective_oz_preset_from_context_str(
    path: &Path,
    context_preset: &str,
    auto_upgrade_folder_mb: u64,
) -> Preset {
    let c = context_preset.trim().to_lowercase();
    match c.as_str() {
        "max" | "aggressive" => Preset::Max,
        "ultra" => Preset::Ultra,
        "auto" => {
            if auto_upgrade_folder_mb > 0 && folder_bumped_to_max(path, auto_upgrade_folder_mb) {
                return Preset::Max;
            }
            suggested_preset_for_path(path)
        }
        "" => effective_oz_preset_from_context_str(path, "auto", auto_upgrade_folder_mb),
        _ => effective_oz_preset_from_context_str(path, "auto", auto_upgrade_folder_mb),
    }
}

/// Пресет для .oz с учётом `OMEGAZIP_CONTEXT_PRESET`, файлов в `~/.config/omegazip/` и порога
/// больших папок (по умолчанию 200 МБ; `0` — отключить).
pub fn effective_oz_preset_from_service_context(path: &Path) -> Preset {
    let ctx = load_context_preset_str();
    let mb = load_auto_upgrade_folder_mb();
    effective_oz_preset_from_context_str(path, &ctx, mb)
}

fn effective_hint_reason(path: &Path, resolved: Preset, ctx: &str, auto_mb: u64) -> String {
    let c = ctx.trim().to_lowercase();
    match c.as_str() {
        "max" | "aggressive" => {
            "Профиль max задан в context_preset (как в Finder Services).".to_string()
        }
        "ultra" => "Профиль ultra задан в context_preset (как в Finder Services).".to_string(),
        _ => {
            if matches!(resolved, Preset::Max) && folder_bumped_to_max(path, auto_mb) {
                return format!(
                    "Папка по размеру ≥ порога {} МБ (auto_upgrade_folder_mb) — для .oz выбран max, как в контекстном меню macOS.",
                    auto_mb
                );
            }
            reason_for(resolved).to_string()
        }
    }
}

/// Подсказка для GUI: тот же пресет, что даст [`effective_oz_preset_from_service_context`].
pub fn effective_compress_preset_hint(path: &Path) -> CompressPresetHint {
    let ctx = load_context_preset_str();
    let auto_mb = load_auto_upgrade_folder_mb();
    let p = effective_oz_preset_from_context_str(path, &ctx, auto_mb);
    CompressPresetHint {
        preset: preset_label(p).to_string(),
        reason: effective_hint_reason(path, p, &ctx, auto_mb),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn text_file_balanced() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("readme.md");
        fs::write(&p, b"# hi").unwrap();
        assert_eq!(suggested_preset_for_path(&p), Preset::Balanced);
    }

    #[test]
    fn media_file_fast() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("clip.mp4");
        fs::write(&p, b"fake").unwrap();
        assert_eq!(suggested_preset_for_path(&p), Preset::Fast);
    }

    #[test]
    fn lossless_image_balanced() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("shot.png");
        fs::write(&p, [0u8; 8]).unwrap();
        assert_eq!(suggested_preset_for_path(&p), Preset::Balanced);
    }

    #[test]
    fn folder_only_mp4_fast() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.mp4"), b"x").unwrap();
        fs::write(dir.path().join("b.mp4"), b"y").unwrap();
        assert_eq!(suggested_preset_for_path(dir.path()), Preset::Fast);
    }

    #[test]
    fn folder_mixed_rs_mp4_balanced() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.mp4"), b"x").unwrap();
        fs::write(dir.path().join("lib.rs"), b"fn main() {}").unwrap();
        assert_eq!(suggested_preset_for_path(dir.path()), Preset::Balanced);
    }

    #[test]
    fn hint_contains_preset_key() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("x.jpg");
        fs::write(&p, b"x").unwrap();
        let h = suggest_compress_preset_hint(&p);
        assert_eq!(h.preset, "fast");
        assert!(!h.reason.is_empty());
    }

    #[test]
    fn pdf_and_epub_use_balanced_for_oz() {
        let dir = tempdir().unwrap();
        let pdf = dir.path().join("a.pdf");
        fs::write(&pdf, b"%PDF-1.4").unwrap();
        assert_eq!(suggested_preset_for_path(&pdf), Preset::Balanced);
        let epub = dir.path().join("b.epub");
        fs::write(&epub, b"PK\x03\x04").unwrap();
        assert_eq!(suggested_preset_for_path(&epub), Preset::Balanced);
    }

    #[test]
    fn effective_context_max_ultra() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("x.jpg");
        fs::write(&p, b"x").unwrap();
        assert_eq!(
            effective_oz_preset_from_context_str(&p, "max", 200),
            Preset::Max
        );
        assert_eq!(
            effective_oz_preset_from_context_str(&p, "ultra", 200),
            Preset::Ultra
        );
    }

    #[test]
    fn effective_context_large_folder_bumps_to_max() {
        let dir = tempdir().unwrap();
        let big = vec![0u8; 2 * 1024 * 1024 + 1];
        // .xyz — не в списках «сжатых» расширений (в отличие от .bin).
        fs::write(dir.path().join("blob.xyz"), &big).unwrap();
        assert_eq!(
            effective_oz_preset_from_context_str(dir.path(), "auto", 1),
            Preset::Max
        );
        assert_eq!(
            effective_oz_preset_from_context_str(dir.path(), "auto", 0),
            Preset::Balanced
        );
    }

    #[test]
    fn effective_context_unknown_falls_back_to_auto() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("z.mp4");
        fs::write(&p, b"x").unwrap();
        assert_eq!(
            effective_oz_preset_from_context_str(&p, "weird", 0),
            Preset::Fast
        );
    }

    #[test]
    fn competitive_prefers_chunked_on_duplicates() {
        let dir = tempdir().unwrap();
        let payload = vec![b'A'; 64 * 1024];
        for i in 0..16 {
            fs::write(dir.path().join(format!("f{i}.txt")), &payload).unwrap();
        }
        let plan = suggest_competitive_plan(dir.path());
        assert!(plan.chunked);
        assert_eq!(plan.preset, "balanced");
    }

    #[test]
    fn competitive_prefers_fast_on_compressed_like_set() {
        let dir = tempdir().unwrap();
        for i in 0..12 {
            fs::write(dir.path().join(format!("m{i}.mp4")), b"fake").unwrap();
        }
        let plan = suggest_competitive_plan(dir.path());
        assert!(!plan.chunked);
        assert_eq!(plan.preset, "fast");
    }
}

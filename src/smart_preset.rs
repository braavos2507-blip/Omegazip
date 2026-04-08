//! Эвристики выбора пресета .oz по расширениям (SMART-01 / SMART-02).
//! Не меняет формат архива; только chunk_size / solid / recovery через `Preset`.

use std::path::Path;

use walkdir::WalkDir;

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
}

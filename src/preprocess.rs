//! Предварительная оптимизация формата перед сжатием.
//! PDF — по env-скрипту, PNG/JPEG — lossy по умолчанию (если есть утилиты),
//! с возможностью переопределить команду через env.

use std::path::Path;
use std::process::Command;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum PreprocessResult {
    WrittenTo(std::path::PathBuf),
    Unchanged(Vec<u8>),
}

fn unique_tmp_path(tag: &str, ext: &str) -> std::path::PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("omegazip_{tag}_{}_{}.{}", std::process::id(), now, ext))
}

fn run_preprocess_script(data: &[u8], script: &str, ext: &str) -> std::io::Result<PreprocessResult> {
    let in_path = unique_tmp_path("in", ext);
    let out_path = unique_tmp_path("out", ext);
    fs::write(&in_path, data)?;
    let status = Command::new("sh")
        .args(["-c", script])
        .env("INPUT", &in_path)
        .env("OUTPUT", &out_path)
        .status();
    let _ = fs::remove_file(&in_path);

    if let Ok(s) = status {
        if s.success() && out_path.exists() {
            let out_data = fs::read(&out_path)?;
            let _ = fs::remove_file(&out_path);
            if out_data.len() < data.len() {
                let final_path = unique_tmp_path("pp", "bin");
                fs::write(&final_path, &out_data)?;
                return Ok(PreprocessResult::WrittenTo(final_path));
            }
        }
    }
    let _ = fs::remove_file(&out_path);
    Ok(PreprocessResult::Unchanged(data.to_vec()))
}

fn env_or_default_script(var_name: &str, default_script: &str) -> String {
    match std::env::var(var_name) {
        Ok(v) if !v.trim().is_empty() => v,
        _ => default_script.to_string(),
    }
}

pub fn preprocess(path: &Path, data: &[u8]) -> std::io::Result<PreprocessResult> {
    // PDF оптимизация (например Ghostscript) — существующее поведение.
    if data.starts_with(b"%PDF") {
        if let Ok(script) = std::env::var("OMEGAZIP_PREPROCESS_PDF") {
            if !script.trim().is_empty() {
                return run_preprocess_script(data, &script, "pdf");
            }
        }
        return Ok(PreprocessResult::Unchanged(data.to_vec()));
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    // PNG lossy по умолчанию (pngquant), можно переопределить env-скриптом.
    if ext == "png" {
        let script = env_or_default_script(
            "OMEGAZIP_PREPROCESS_PNG",
            // Если pngquant отсутствует — silently fallback на исходный файл.
            "pngquant --force --strip --quality=65-85 --output \"$OUTPUT\" -- \"$INPUT\" >/dev/null 2>&1",
        );
        return run_preprocess_script(data, &script, "png");
    }

    // APNG обычно ломается после pngquant; для безопасности не трогаем.
    if ext == "apng" {
        return Ok(PreprocessResult::Unchanged(data.to_vec()));
    }

    // JPEG lossy по умолчанию (jpegoptim --max=82), можно переопределить env-скриптом.
    if ext == "jpg" || ext == "jpeg" || ext == "jfif" {
        let script = env_or_default_script(
            "OMEGAZIP_PREPROCESS_JPEG",
            // Если jpegoptim отсутствует — silently fallback на исходный файл.
            "jpegoptim --max=82 --strip-all --stdout \"$INPUT\" > \"$OUTPUT\" 2>/dev/null",
        );
        return run_preprocess_script(data, &script, "jpg");
    }

    Ok(PreprocessResult::Unchanged(data.to_vec()))
}

pub fn read_preprocess_result(r: PreprocessResult) -> std::io::Result<Vec<u8>> {
    match r {
        PreprocessResult::WrittenTo(p) => fs::read(&p).inspect(|_| {
            let _ = fs::remove_file(&p);
        }),
        PreprocessResult::Unchanged(d) => Ok(d),
    }
}

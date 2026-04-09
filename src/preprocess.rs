//! Предварительная оптимизация формата перед сжатием (только через скрипты из env).

use std::path::Path;
use std::process::Command;
use std::fs;

pub enum PreprocessResult {
    WrittenTo(std::path::PathBuf),
    Unchanged(Vec<u8>),
}

pub fn preprocess(_path: &Path, data: &[u8]) -> std::io::Result<PreprocessResult> {
    if !data.starts_with(b"%PDF") {
        return Ok(PreprocessResult::Unchanged(data.to_vec()));
    }
    let script = match std::env::var("OMEGAZIP_PREPROCESS_PDF") {
        Ok(s) => s,
        Err(_) => return Ok(PreprocessResult::Unchanged(data.to_vec())),
    };
    let tmp = std::env::temp_dir().join("omegazip_in.pdf");
    let out_path = std::env::temp_dir().join("omegazip_out.pdf");
    fs::write(&tmp, data)?;
    let ok = Command::new("sh")
        .args(["-c", &script])
        .env("INPUT", &tmp)
        .env("OUTPUT", &out_path)
        .status();
    let _ = fs::remove_file(&tmp);
    if let Ok(s) = ok {
        if s.success() && out_path.exists() {
            let out_data = fs::read(&out_path)?;
            let _ = fs::remove_file(&out_path);
            if out_data.len() < data.len() {
                let final_path = std::env::temp_dir().join("omegazip_pp.bin");
                fs::write(&final_path, &out_data)?;
                return Ok(PreprocessResult::WrittenTo(final_path));
            }
        }
    }
    Ok(PreprocessResult::Unchanged(data.to_vec()))
}

pub fn read_preprocess_result(r: PreprocessResult) -> std::io::Result<Vec<u8>> {
    match r {
        PreprocessResult::WrittenTo(p) => fs::read(&p).and_then(|d| {
            let _ = fs::remove_file(&p);
            Ok(d)
        }),
        PreprocessResult::Unchanged(d) => Ok(d),
    }
}

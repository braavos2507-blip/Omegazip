//! Тихая распаковка и тихое сжатие при открытии файлов из ОС.

#[cfg(not(target_os = "android"))]
use std::fs;
#[cfg(not(target_os = "android"))]
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(not(target_os = "android"))]
use serde::{Deserialize, Serialize};
#[cfg(not(target_os = "android"))]
use tauri::{AppHandle, Emitter, Manager};

#[cfg(not(target_os = "android"))]
const CONFIG_FILE: &str = "gui-prefs.json";

#[cfg(not(target_os = "android"))]
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct GuiPrefs {
    silent_extract_on_open: bool,
    silent_compress_on_open: bool,
    silent_compress_format: String,
}

#[cfg(not(target_os = "android"))]
impl Default for GuiPrefs {
    fn default() -> Self {
        Self {
            silent_extract_on_open: true,
            silent_compress_on_open: true,
            silent_compress_format: "oz".to_string(),
        }
    }
}

#[cfg(not(target_os = "android"))]
fn prefs_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join(CONFIG_FILE))
}

#[cfg(not(target_os = "android"))]
fn read_gui_prefs(app: &AppHandle) -> GuiPrefs {
    prefs_path(app)
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<GuiPrefs>(&s).ok())
        .unwrap_or_default()
}

#[cfg(not(target_os = "android"))]
fn write_gui_prefs(app: &AppHandle, prefs: &GuiPrefs) -> Result<(), String> {
    let path = prefs_path(app).ok_or_else(|| "Не удалось получить каталог настроек".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[cfg(not(target_os = "android"))]
fn normalized_silent_compress_format(fmt: &str) -> &'static str {
    if fmt.eq_ignore_ascii_case("zip") {
        "zip"
    } else {
        "oz"
    }
}

#[cfg(not(target_os = "android"))]
pub fn silent_extract_enabled(app: &AppHandle) -> bool {
    let v = std::env::var("OMEGAZIP_NO_SILENT_EXTRACT").unwrap_or_default();
    if v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes") {
        return false;
    }
    read_gui_prefs(app).silent_extract_on_open
}

#[cfg(not(target_os = "android"))]
pub fn set_silent_extract_enabled(app: &AppHandle, on: bool) -> Result<(), String> {
    let mut prefs = read_gui_prefs(app);
    prefs.silent_extract_on_open = on;
    write_gui_prefs(app, &prefs)
}

#[cfg(not(target_os = "android"))]
pub fn silent_compress_enabled(app: &AppHandle) -> bool {
    let v = std::env::var("OMEGAZIP_NO_SILENT_COMPRESS").unwrap_or_default();
    if v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes") {
        return false;
    }
    read_gui_prefs(app).silent_compress_on_open
}

#[cfg(not(target_os = "android"))]
pub fn set_silent_compress_enabled(app: &AppHandle, on: bool) -> Result<(), String> {
    let mut prefs = read_gui_prefs(app);
    prefs.silent_compress_on_open = on;
    write_gui_prefs(app, &prefs)
}

#[cfg(not(target_os = "android"))]
pub fn silent_compress_format_stored(app: &AppHandle) -> String {
    normalized_silent_compress_format(&read_gui_prefs(app).silent_compress_format).to_string()
}

#[cfg(not(target_os = "android"))]
pub fn set_silent_compress_format(app: &AppHandle, format: String) -> Result<(), String> {
    let fmt = normalized_silent_compress_format(format.trim());
    let mut prefs = read_gui_prefs(app);
    prefs.silent_compress_format = fmt.to_string();
    write_gui_prefs(app, &prefs)
}

#[cfg(not(target_os = "android"))]
fn silent_compress_dest_path(source: &Path, format: &str) -> Option<PathBuf> {
    let parent = source.parent()?;
    let ext = normalized_silent_compress_format(format);
    let stem = if source.is_dir() {
        source.file_name()?.to_str()?.to_string()
    } else {
        match source.file_stem().and_then(|s| s.to_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => source.file_name()?.to_str()?.to_string(),
        }
    };
    if stem.is_empty() {
        return None;
    }
    Some(parent.join(format!("{stem}.{ext}")))
}

#[cfg(not(target_os = "android"))]
fn single_existing_input_path(paths: &[String]) -> Option<PathBuf> {
    let mut uniq: Vec<PathBuf> = Vec::new();
    for raw in paths {
        let s = raw.trim().trim_matches('"');
        if s.is_empty() {
            continue;
        }
        let pb = if let Some(rest) = s.strip_prefix("file://") {
            // NSServices иногда присылает file:// URL строкой вместо обычного пути.
            let without_host = rest.strip_prefix("localhost/").unwrap_or(rest);
            PathBuf::from(percent_decode_url_component(without_host))
        } else {
            PathBuf::from(s)
        };
        if !pb.exists() {
            continue;
        }
        if !uniq.iter().any(|p| p == &pb) {
            uniq.push(pb);
        }
    }
    if uniq.len() == 1 {
        uniq.into_iter().next()
    } else {
        None
    }
}

#[cfg(not(target_os = "android"))]
fn percent_decode_url_component(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h1 = bytes[i + 1];
            let h2 = bytes[i + 2];
            let v1 = (h1 as char).to_digit(16);
            let v2 = (h2 as char).to_digit(16);
            if let (Some(a), Some(b)) = (v1, v2) {
                out.push(((a << 4) as u8) | (b as u8));
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(target_os = "macos")]
fn notify_macos(title: &str, message: &str) {
    let title = title.replace('"', "'");
    let message = message.replace('"', "'");
    let script = format!("display notification \"{message}\" with title \"{title}\"");
    let _ = Command::new("osascript").arg("-e").arg(script).status();
}

#[cfg(not(target_os = "android"))]
fn error_suggests_password(msg: &str) -> bool {
    let m = msg.to_lowercase();
    m.contains("парол")
        || m.contains("password")
        || m.contains("encrypted")
        || m.contains("зашифрован")
}

/// Старт фоновой распаковки: окно скрыто, при успехе — `exit(0)`, при ошибке — показ окна и события.
/// Возвращает `true`, если сценарий silent запущен (не эмитить `open-files` сразу).
#[cfg(not(target_os = "android"))]
pub fn try_start_silent_background(app: &AppHandle, paths: &[String]) -> bool {
    try_start_silent_background_impl(app, paths, false)
}

/// То же, но игнорирует пользовательский флаг в GUI-pref (для NSServices/Quick Actions).
#[cfg(not(target_os = "android"))]
pub fn try_start_silent_background_forced(app: &AppHandle, paths: &[String]) -> bool {
    try_start_silent_background_impl(app, paths, true)
}

#[cfg(not(target_os = "android"))]
fn try_start_silent_background_impl(app: &AppHandle, paths: &[String], force: bool) -> bool {
    let Some(pb) = single_existing_input_path(paths) else {
        return false;
    };
    if !pb.is_file() {
        return false;
    }
    if !force && !silent_extract_enabled(app) {
        return false;
    }
    if !omegazip::looks_like_supported_archive_path(&pb) {
        return false;
    }
    let dest = match pb.parent() {
        Some(d) => d.to_path_buf(),
        None => return false,
    };
    let archive = pb;

    if force {
        let r = omegazip::decompress_any_to_path(&archive, &dest);
        match r {
            Ok(_) => {
                #[cfg(target_os = "macos")]
                notify_macos(
                    "OmegaZip: распаковка",
                    &format!("Готово в папке: {}", dest.to_string_lossy()),
                );
                app.exit(0);
            }
            Err(e) => {
                #[cfg(target_os = "macos")]
                notify_macos("OmegaZip: ошибка распаковки", &e.to_string());
                app.exit(1);
            }
        }
        return true;
    }

    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
    let h = app.clone();
    std::thread::spawn(move || {
        let r = omegazip::decompress_any_to_path(&archive, &dest);
        match r {
            Ok(_) => {
                #[cfg(target_os = "macos")]
                if force {
                    notify_macos(
                        "OmegaZip: распаковка",
                        &format!("Готово в папке: {}", dest.to_string_lossy()),
                    );
                }
                h.exit(0);
            }
            Err(e) => {
                if force {
                    #[cfg(target_os = "macos")]
                    notify_macos("OmegaZip: ошибка распаковки", &e.to_string());
                    h.exit(1);
                    return;
                }
                let msg = e.to_string();
                let path_str = archive.to_string_lossy().into_owned();
                let needs_pw = error_suggests_password(&msg);
                let h2 = h.clone();
                let _ = h.run_on_main_thread(move || {
                    if let Some(w) = h2.get_webview_window("main") {
                        let _ = w.show();
                    }
                    let _ = h2.emit("open-files", vec![path_str]);
                    let payload = serde_json::json!({
                        "message": msg,
                        "needs_password": needs_pw,
                    });
                    let _ = h2.emit("silent-extract-failed", payload);
                });
            }
        }
    });

    true
}

/// Тихое сжатие одного файла или папки в каталог источника (`.oz` или `.zip` по настройке).
/// Возвращает `true`, если сценарий запущен (не эмитить `open-files` сразу).
#[cfg(not(target_os = "android"))]
pub fn try_start_silent_compress_background(app: &AppHandle, paths: &[String]) -> bool {
    try_start_silent_compress_background_impl(app, paths, false)
}

/// То же, но игнорирует пользовательский флаг в GUI-pref (для NSServices/Quick Actions).
#[cfg(not(target_os = "android"))]
pub fn try_start_silent_compress_background_forced(app: &AppHandle, paths: &[String]) -> bool {
    try_start_silent_compress_background_impl(app, paths, true)
}

#[cfg(not(target_os = "android"))]
fn try_start_silent_compress_background_impl(app: &AppHandle, paths: &[String], force: bool) -> bool {
    let Some(pb) = single_existing_input_path(paths) else {
        return false;
    };
    if !pb.exists() || !(pb.is_file() || pb.is_dir()) {
        return false;
    }
    if omegazip::looks_like_supported_archive_path(&pb) {
        return false;
    }
    if !force && !silent_compress_enabled(app) {
        return false;
    }
    let prefs = read_gui_prefs(app);
    let dest = match silent_compress_dest_path(&pb, &prefs.silent_compress_format) {
        Some(d) => d,
        None => return false,
    };
    let source = pb;

    if force {
        let preset = omegazip::suggested_preset_for_path(&source);
        let opts = omegazip::CompressOptions {
            chunk_size: None,
            solid: false,
            password: None,
            recovery_parity: 0,
            preset: Some(preset),
            parallel: true,
            progress: None,
        };
        let r = omegazip::compress_advanced_dispatch(&source, &dest, opts);
        match r {
            Ok(_) => {
                #[cfg(target_os = "macos")]
                notify_macos(
                    "OmegaZip: архивация",
                    &format!("Готово: {}", dest.to_string_lossy()),
                );
                app.exit(0);
            }
            Err(e) => {
                #[cfg(target_os = "macos")]
                notify_macos("OmegaZip: ошибка архивации", &e.to_string());
                app.exit(1);
            }
        }
        return true;
    }

    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
    let h = app.clone();
    std::thread::spawn(move || {
        let preset = omegazip::suggested_preset_for_path(&source);
        let opts = omegazip::CompressOptions {
            chunk_size: None,
            solid: false,
            password: None,
            recovery_parity: 0,
            preset: Some(preset),
            parallel: true,
            progress: None,
        };
        let r = omegazip::compress_advanced_dispatch(&source, &dest, opts);
        match r {
            Ok(_) => {
                #[cfg(target_os = "macos")]
                if force {
                    notify_macos(
                        "OmegaZip: архивация",
                        &format!("Готово: {}", dest.to_string_lossy()),
                    );
                }
                h.exit(0);
            }
            Err(e) => {
                if force {
                    #[cfg(target_os = "macos")]
                    notify_macos("OmegaZip: ошибка архивации", &e.to_string());
                    h.exit(1);
                    return;
                }
                let msg = e.to_string();
                let path_str = source.to_string_lossy().into_owned();
                let h2 = h.clone();
                let _ = h.run_on_main_thread(move || {
                    if let Some(w) = h2.get_webview_window("main") {
                        let _ = w.show();
                    }
                    let _ = h2.emit("open-files", vec![path_str.clone()]);
                    let payload = serde_json::json!({
                        "message": msg,
                    });
                    let _ = h2.emit("silent-compress-failed", payload);
                });
            }
        }
    });

    true
}

#[cfg(target_os = "android")]
use tauri::AppHandle;

#[cfg(target_os = "android")]
pub fn silent_extract_enabled(_app: &AppHandle) -> bool {
    false
}

#[cfg(target_os = "android")]
pub fn set_silent_extract_enabled(_app: &AppHandle, _on: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "android")]
pub fn silent_compress_enabled(_app: &AppHandle) -> bool {
    false
}

#[cfg(target_os = "android")]
pub fn set_silent_compress_enabled(_app: &AppHandle, _on: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "android")]
pub fn silent_compress_format_stored(_app: &AppHandle) -> String {
    "oz".to_string()
}

#[cfg(target_os = "android")]
pub fn set_silent_compress_format(_app: &AppHandle, _format: String) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "android")]
pub fn try_start_silent_background(_app: &AppHandle, _paths: &[String]) -> bool {
    false
}

#[cfg(target_os = "android")]
pub fn try_start_silent_background_forced(_app: &AppHandle, _paths: &[String]) -> bool {
    false
}

#[cfg(target_os = "android")]
pub fn try_start_silent_compress_background(_app: &AppHandle, _paths: &[String]) -> bool {
    false
}

#[cfg(target_os = "android")]
pub fn try_start_silent_compress_background_forced(_app: &AppHandle, _paths: &[String]) -> bool {
    false
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod silent_open;

#[cfg(target_os = "macos")]
mod macos_services;

#[cfg(target_os = "macos")]
mod macos_open_files;

fn handle_opened_paths(app: &AppHandle, paths: Vec<String>) {
    if paths.is_empty() {
        return;
    }
    #[cfg(target_os = "macos")]
    {
        SHOW_MAIN_ON_READY.store(false, Ordering::SeqCst);
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.hide();
        }
    }
    if silent_open::try_start_silent_background(app, &paths) {
        return;
    }
    if silent_open::try_start_silent_compress_background(app, &paths) {
        return;
    }
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
    }
    let _ = app.emit("open-files", paths);
}

#[cfg(target_os = "macos")]
fn handle_opened_paths_from_service(app: &AppHandle, paths: Vec<String>) {
    if paths.is_empty() {
        app.exit(0);
        return;
    }
    SHOW_MAIN_ON_READY.store(false, Ordering::SeqCst);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
    if silent_open::try_start_silent_background_forced(app, &paths) {
        return;
    }
    if silent_open::try_start_silent_compress_background_forced(app, &paths) {
        return;
    }
    // Для Finder Services не показываем GUI fallback: сервис должен быть бесшумным.
    app.exit(0);
}

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, DragDropEvent, Emitter, Manager, RunEvent, WebviewEvent, WindowEvent};
#[cfg(not(target_os = "macos"))]
use tauri_plugin_dialog::DialogExt;

#[cfg(target_os = "macos")]
static SHOW_MAIN_ON_READY: AtomicBool = AtomicBool::new(true);

#[cfg(target_os = "windows")]
fn ensure_windows_shell_integration(app: &AppHandle) {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    let resource_dir = match app.path().resource_dir() {
        Ok(p) => p,
        Err(_) => return,
    };

    fn pick_script(base: &Path, name: &str) -> Option<PathBuf> {
        let in_scripts = base.join("scripts").join(name);
        if in_scripts.exists() {
            return Some(in_scripts);
        }
        let flat = base.join(name);
        if flat.exists() {
            return Some(flat);
        }
        None
    }

    let install_context = match pick_script(&resource_dir, "install-context-menu-windows.ps1") {
        Some(p) => p,
        None => return,
    };
    let install_assoc = match pick_script(&resource_dir, "install-oz-file-association-windows.ps1") {
        Some(p) => p,
        None => return,
    };
    let install_context_s = install_context.to_string_lossy().into_owned();
    let install_assoc_s = install_assoc.to_string_lossy().into_owned();
    let exe_s = exe.to_string_lossy().into_owned();

    let _ = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &install_context_s,
            "-OmegaZipExe",
            &exe_s,
        ])
        .spawn();

    let _ = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &install_assoc_s,
            "-OmegaZipApp",
            &exe_s,
        ])
        .spawn();
}

#[tauri::command]
fn compress(source: PathBuf, archive_path: PathBuf) -> Result<u32, String> {
    omegazip::compress_dispatch(&source, &archive_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn compress_advanced(
    app: AppHandle,
    source: PathBuf,
    archive_path: PathBuf,
    password: Option<String>,
    chunked: bool,
    solid: bool,
    recovery: bool,
    preset: Option<String>,
) -> Result<u32, String> {
    let progress = Arc::new(move |p: omegazip::Progress| {
        let _ = app.emit("compress-progress", &p);
    });
    let preset_parsed = preset.and_then(|s| omegazip::Preset::parse_name(&s));
    let opts = omegazip::CompressOptions {
        chunk_size: if chunked {
            Some(omegazip::DEFAULT_CHUNK_SIZE)
        } else {
            None
        },
        solid,
        password,
        recovery_parity: if recovery { 2 } else { 0 },
        preset: preset_parsed,
        parallel: true,
        progress: Some(progress),
        solid_block_size_bytes: None,
        zip_analyzed: false,
    };
    omegazip::compress_advanced_dispatch(&source, &archive_path, opts).map_err(|e| e.to_string())
}

#[tauri::command]
fn decompress(archive_path: PathBuf, dest_dir: PathBuf) -> Result<u32, String> {
    omegazip::decompress_any_to_path(&archive_path, &dest_dir).map_err(|e| e.to_string())
}

#[tauri::command]
fn decompress_with_password(
    app: AppHandle,
    archive_path: PathBuf,
    dest_dir: PathBuf,
    password: Option<String>,
) -> Result<u32, String> {
    let progress = Arc::new(move |p: omegazip::Progress| {
        let _ = app.emit("decompress-progress", &p);
    });
    omegazip::decompress_any_to_path_with_options(
        &archive_path,
        &dest_dir,
        password.as_deref(),
        Some(progress),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn archive_info(archive_path: PathBuf) -> Result<omegazip::ArchiveInfo, String> {
    omegazip::archive_info(&archive_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_archive(
    archive_path: PathBuf,
    password: Option<String>,
) -> Result<Vec<String>, String> {
    omegazip::list_any_archive_with_password(&archive_path, password.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn export_to_zip(
    archive_path: PathBuf,
    zip_path: PathBuf,
    password: Option<String>,
) -> Result<u32, String> {
    omegazip::export_to_zip(&archive_path, &zip_path, password.as_deref())
        .map_err(|e| e.to_string())
}

#[cfg(not(target_os = "macos"))]
fn file_path_to_string(p: tauri_plugin_dialog::FilePath) -> String {
    p.to_string()
}

#[cfg(target_os = "macos")]
fn pick_file_or_folder_rfd() -> Result<Option<String>, String> {
    // Сначала файл (чаще нужен); отмена → выбор папки
    let res = rfd::FileDialog::new()
        .pick_file()
        .or_else(|| rfd::FileDialog::new().pick_folder());
    Ok(res.map(|p| p.to_string_lossy().into_owned()))
}

#[cfg(not(target_os = "macos"))]
fn pick_file_or_folder_impl(app: &AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        app2.dialog().file().pick_folder(move |path| {
            let res = path.map(&file_path_to_string);
            if tx.send(res).is_err() {}
        });
    })
    .map_err(|e| e.to_string())?;
    match rx.recv() {
        Ok(Some(p)) => return Ok(Some(p)),
        Ok(None) => {}
        Err(_) => return Err("диалог отменён".into()),
    }
    let (tx2, rx2) = std::sync::mpsc::sync_channel(1);
    let app3 = app.clone();
    app.run_on_main_thread(move || {
        app3.dialog().file().pick_file(move |path| {
            let _ = tx2.send(path.map(&file_path_to_string));
        });
    })
    .map_err(|e| e.to_string())?;
    rx2.recv().map_err(|_| "диалог отменён".into())
}

#[cfg(target_os = "macos")]
fn pick_file_or_folder_impl(_app: &AppHandle) -> Result<Option<String>, String> {
    pick_file_or_folder_rfd()
}

#[tauri::command]
fn pick_file_or_folder(app: AppHandle) -> Result<Option<String>, String> {
    pick_file_or_folder_impl(&app)
}

#[cfg(target_os = "macos")]
fn pick_save_file_rfd(
    default_name: Option<String>,
    default_directory: Option<String>,
) -> Result<Option<String>, String> {
    let name = default_name.unwrap_or_else(|| "archive.zip".into());
    let mut dlg = if name.ends_with(".zip") {
        rfd::FileDialog::new()
            .add_filter("ZIP", &["zip"])
            .set_file_name(&name)
    } else {
        rfd::FileDialog::new()
            .add_filter("OmegaZip", &["oz"])
            .set_file_name(&name)
    };
    if let Some(dir) = default_directory.filter(|s| !s.is_empty()) {
        dlg = dlg.set_directory(Path::new(&dir));
    }
    Ok(dlg.save_file().map(|p| p.to_string_lossy().into_owned()))
}

#[cfg(not(target_os = "macos"))]
fn pick_save_file_impl(
    app: &AppHandle,
    default_name: Option<String>,
    default_directory: Option<String>,
) -> Result<Option<String>, String> {
    let name = default_name.unwrap_or_else(|| "archive.zip".into());
    let (filter_name, exts): (&str, &[&str]) = if name.ends_with(".zip") {
        ("ZIP", &["zip"])
    } else {
        ("OmegaZip", &["oz"])
    };
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        let mut builder = app2
            .dialog()
            .file()
            .add_filter(filter_name, exts)
            .set_file_name(&name);
        if let Some(ref dir) = default_directory {
            if !dir.is_empty() {
                builder = builder.set_directory(Path::new(dir));
            }
        }
        builder.save_file(move |path| {
            let _ = tx.send(path.map(&file_path_to_string));
        });
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|_| "диалог отменён".into())
}

#[cfg(target_os = "macos")]
fn pick_save_file_impl(
    _app: &AppHandle,
    default_name: Option<String>,
    default_directory: Option<String>,
) -> Result<Option<String>, String> {
    pick_save_file_rfd(default_name, default_directory)
}

#[tauri::command]
fn pick_save_file(
    app: AppHandle,
    default_name: Option<String>,
    default_directory: Option<String>,
) -> Result<Option<String>, String> {
    pick_save_file_impl(&app, default_name, default_directory)
}

#[cfg(target_os = "macos")]
fn pick_folder_rfd() -> Result<Option<String>, String> {
    Ok(rfd::FileDialog::new()
        .pick_folder()
        .map(|p| p.to_string_lossy().into_owned()))
}

#[cfg(not(target_os = "macos"))]
fn pick_folder_impl(app: &AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        app2.dialog().file().pick_folder(move |path| {
            let _ = tx.send(path.map(&file_path_to_string));
        });
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|_| "диалог отменён".into())
}

#[cfg(target_os = "macos")]
fn pick_folder_impl(_app: &AppHandle) -> Result<Option<String>, String> {
    pick_folder_rfd()
}

#[tauri::command]
fn pick_folder(app: AppHandle) -> Result<Option<String>, String> {
    pick_folder_impl(&app)
}

#[cfg(target_os = "macos")]
fn pick_oz_file_rfd() -> Result<Option<String>, String> {
    Ok(rfd::FileDialog::new()
        .add_filter("OmegaZip", &["oz"])
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned()))
}

#[cfg(target_os = "macos")]
fn pick_archive_file_rfd() -> Result<Option<String>, String> {
    Ok(rfd::FileDialog::new()
        .add_filter(
            "Архивы",
            &[
                "oz", "zip", "7z", "rar", "tar", "gz", "tgz", "xz", "txz", "bz2", "tbz2", "tbz",
                "zst", "tzst", "cab", "iso", "wim", "msi", "dmg", "deb", "rpm", "xar", "lzma",
                "jar", "war", "apk", "ipa", "docx", "xlsx", "pptx", "odt", "ods", "cbz", "cbr",
            ],
        )
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned()))
}

#[cfg(not(target_os = "macos"))]
fn pick_archive_file_impl(app: &AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        app2
            .dialog()
            .file()
            .add_filter(
                "Архивы",
                &[
                    "oz", "zip", "7z", "rar", "tar", "gz", "tgz", "xz", "txz", "bz2", "tbz2", "tbz",
                    "zst", "tzst", "cab", "iso", "wim", "msi", "dmg", "deb", "rpm", "xar", "lzma",
                    "jar", "war", "apk", "ipa", "docx", "xlsx", "pptx", "odt", "ods", "cbz", "cbr",
                ],
            )
            .pick_file(move |path| {
                let _ = tx.send(path.map(&file_path_to_string));
            });
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|_| "диалог отменён".into())
}

#[cfg(target_os = "macos")]
fn pick_archive_file_impl(_app: &AppHandle) -> Result<Option<String>, String> {
    pick_archive_file_rfd()
}

#[tauri::command]
fn pick_archive_file(app: AppHandle) -> Result<Option<String>, String> {
    pick_archive_file_impl(&app)
}

#[cfg(not(target_os = "macos"))]
fn pick_oz_file_impl(app: &AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        app2
            .dialog()
            .file()
            .add_filter("OmegaZip", &["oz"])
            .pick_file(move |path| {
                let _ = tx.send(path.map(&file_path_to_string));
            });
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|_| "диалог отменён".into())
}

#[cfg(target_os = "macos")]
fn pick_oz_file_impl(_app: &AppHandle) -> Result<Option<String>, String> {
    pick_oz_file_rfd()
}

#[tauri::command]
fn pick_oz_file(app: AppHandle) -> Result<Option<String>, String> {
    pick_oz_file_impl(&app)
}

// ---- rclone integration (cloud sync; только десктоп) ----
#[cfg(not(target_os = "android"))]
#[tauri::command]
fn rclone_list_remotes() -> Result<Vec<String>, String> {
    let out = Command::new("rclone")
        .args(["listremotes"])
        .output()
        .map_err(|e| format!("rclone не найден или недоступен: {}", e))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("rclone listremotes: {}", stderr.trim()));
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let remotes: Vec<String> = s
        .lines()
        .map(|l| l.trim().trim_end_matches(':').to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(remotes)
}

#[cfg(target_os = "android")]
#[tauri::command]
fn rclone_list_remotes() -> Result<Vec<String>, String> {
    Err("Облако rclone в OmegaZip Android не поддерживается.".into())
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
fn rclone_upload(local_path: PathBuf, remote_spec: String) -> Result<(), String> {
    let local = Path::new(&local_path);
    if !local.exists() {
        return Err("Локальный файл или папка не найдены".into());
    }
    let out = Command::new("rclone")
        .args(["copy", local.to_string_lossy().as_ref(), remote_spec.trim()])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("rclone copy: {}", stderr.trim()));
    }
    Ok(())
}

#[cfg(target_os = "android")]
#[tauri::command]
fn rclone_upload(_local_path: PathBuf, _remote_spec: String) -> Result<(), String> {
    Err("Облако rclone в OmegaZip Android не поддерживается.".into())
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
fn rclone_download(remote_spec: String, local_path: PathBuf) -> Result<(), String> {
    let out = Command::new("rclone")
        .args(["copy", remote_spec.trim(), local_path.to_string_lossy().as_ref()])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("rclone copy: {}", stderr.trim()));
    }
    Ok(())
}

#[cfg(target_os = "android")]
#[tauri::command]
fn rclone_download(_remote_spec: String, _local_path: PathBuf) -> Result<(), String> {
    Err("Облако rclone в OmegaZip Android не поддерживается.".into())
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
fn rclone_available() -> Result<bool, String> {
    Ok(Command::new("rclone").arg("version").output().is_ok_and(|o| o.status.success()))
}

#[cfg(target_os = "android")]
#[tauri::command]
fn rclone_available() -> Result<bool, String> {
    Ok(false)
}

/// Статус 7-Zip / p7zip: путь, инструкции по установке, заметка про RAR.
#[tauri::command]
fn seven_zip_status() -> omegazip::SevenZipStatus {
    omegazip::seven_zip_status()
}

/// Рекомендуемый пресет .oz по расширениям источника (SMART-01).
#[tauri::command]
fn suggest_compress_preset(path: String) -> omegazip::CompressPresetHint {
    let p = PathBuf::from(path);
    omegazip::suggest_compress_preset_hint(&p)
}

/// Пресет .oz с учётом `context_preset` и порога больших папок (как Finder Services).
#[tauri::command]
fn effective_compress_preset(path: String) -> String {
    let p = PathBuf::from(path);
    match omegazip::effective_oz_preset_from_service_context(&p) {
        omegazip::Preset::Fast => "fast".to_string(),
        omegazip::Preset::Balanced => "balanced".to_string(),
        omegazip::Preset::Max => "max".to_string(),
        omegazip::Preset::Ultra => "ultra".to_string(),
    }
}

#[tauri::command]
fn effective_compress_preset_hint(path: String) -> omegazip::CompressPresetHint {
    let p = PathBuf::from(path);
    omegazip::effective_compress_preset_hint(&p)
}

#[tauri::command]
fn get_silent_extract_on_open(app: AppHandle) -> bool {
    silent_open::silent_extract_enabled(&app)
}

#[tauri::command]
fn set_silent_extract_on_open(app: AppHandle, enabled: bool) -> Result<(), String> {
    silent_open::set_silent_extract_enabled(&app, enabled)
}

#[tauri::command]
fn get_silent_compress_on_open(app: AppHandle) -> bool {
    silent_open::silent_compress_enabled(&app)
}

#[tauri::command]
fn set_silent_compress_on_open(app: AppHandle, enabled: bool) -> Result<(), String> {
    silent_open::set_silent_compress_enabled(&app, enabled)
}

#[tauri::command]
fn get_silent_compress_format(app: AppHandle) -> String {
    silent_open::silent_compress_format_stored(&app)
}

#[tauri::command]
fn set_silent_compress_format(app: AppHandle, format: String) -> Result<(), String> {
    silent_open::set_silent_compress_format(&app, format)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            compress,
            compress_advanced,
            decompress,
            decompress_with_password,
            archive_info,
            list_archive,
            export_to_zip,
            pick_file_or_folder,
            pick_save_file,
            pick_folder,
            pick_oz_file,
            pick_archive_file,
            rclone_list_remotes,
            rclone_upload,
            rclone_download,
            rclone_available,
            seven_zip_status,
            suggest_compress_preset,
            effective_compress_preset,
            effective_compress_preset_hint,
            get_silent_extract_on_open,
            set_silent_extract_on_open,
            get_silent_compress_on_open,
            set_silent_compress_on_open,
            get_silent_compress_format,
            set_silent_compress_format,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                macos_open_files::register_app_handle(app.handle().clone());
                // Раньше, чем колбэк RunEvent::Ready — ловим application:openFiles: от NSServices.
                macos_open_files::try_install_open_files_hook();
                // На macOS стартуем скрыто, чтобы окно не всплывало от NSServices.
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building OmegaZip")
        .run(|app_handle, event| {
            #[cfg(target_os = "macos")]
            if let RunEvent::Ready = event {
                macos_open_files::try_install_open_files_hook();
                let ready_paths: Vec<String> = std::env::args()
                    .skip(1)
                    .filter_map(|a| {
                        let p = Path::new(&a);
                        if p.exists() {
                            Some(p.to_string_lossy().into_owned())
                        } else {
                            None
                        }
                    })
                    .collect();
                if !ready_paths.is_empty() {
                    handle_opened_paths_from_service(app_handle, ready_paths);
                    return;
                }
                let consumed = macos_services::try_emit_open_files_from_pasteboard(app_handle);
                if !consumed && SHOW_MAIN_ON_READY.load(Ordering::SeqCst) {
                    if let Some(w) = app_handle.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                return;
            }
            #[cfg(target_os = "macos")]
            if let RunEvent::Resumed = event {
                macos_open_files::try_install_open_files_hook();
                let consumed = macos_services::try_emit_open_files_from_pasteboard(app_handle);
                if !consumed && SHOW_MAIN_ON_READY.load(Ordering::SeqCst) {
                    if let Some(w) = app_handle.get_webview_window("main") {
                        let _ = w.show();
                    }
                }
                return;
            }
            #[cfg(target_os = "macos")]
            if let RunEvent::Opened { urls } = event {
                let paths: Vec<String> = urls
                    .iter()
                    .filter_map(|u| u.to_file_path().ok())
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                handle_opened_paths(app_handle, paths);
                return;
            }
            #[cfg(target_os = "macos")]
            if let RunEvent::Reopen { has_visible_windows, .. } = event {
                let consumed = macos_services::try_emit_open_files_from_pasteboard(app_handle);
                if !consumed && !has_visible_windows && SHOW_MAIN_ON_READY.load(Ordering::SeqCst) {
                    if let Some(w) = app_handle.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                return;
            }
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            if let RunEvent::Ready = event {
                #[cfg(target_os = "windows")]
                ensure_windows_shell_integration(app_handle);
                let paths: Vec<String> = std::env::args()
                    .skip(1)
                    .filter_map(|a| {
                        let p = Path::new(&a);
                        if p.exists() {
                            Some(p.to_string_lossy().into_owned())
                        } else {
                            None
                        }
                    })
                    .collect();
                if paths.is_empty() {
                    if let Some(w) = app_handle.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                } else {
                    handle_opened_paths(app_handle, paths);
                }
                return;
            }
            // macOS + WebviewWindow: сброс файлов часто приходит как WebviewEvent, а не WindowEvent.
            if let RunEvent::WindowEvent {
                event: WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }),
                ..
            } = &event
            {
                let paths: Vec<String> = paths
                    .iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                if !paths.is_empty() {
                    let _ = app_handle.emit("drop-files", &paths);
                }
                return;
            }
            if let RunEvent::WebviewEvent {
                event: WebviewEvent::DragDrop(DragDropEvent::Drop { paths, .. }),
                ..
            } = &event
            {
                let paths: Vec<String> = paths
                    .iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                if !paths.is_empty() {
                    let _ = app_handle.emit("drop-files", &paths);
                }
                return;
            }
            let _ = (app_handle, event);
        });
}

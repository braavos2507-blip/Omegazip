//! macOS: read file URLs from pasteboard when app is activated (e.g. after user
//! clicks "Сжать в OmegaZip" in Finder Services). Full NSServices provider would
//! require native Obj-C; reading general pasteboard on Resumed is a workaround.

use objc2::rc::Retained;
use objc2::msg_send;
use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardTypeFileURL};
use objc2_foundation::MainThreadMarker;

#[allow(dead_code)]
fn read_file_paths_from_pasteboard(pboard: &NSPasteboard) -> Vec<String> {
    let Some(items) = pboard.pasteboardItems() else {
        return vec![];
    };
    let count: usize = unsafe { msg_send![&items, count] };
    let mut paths = Vec::with_capacity(count);
    for i in 0..count {
        let item: Option<Retained<NSPasteboardItem>> =
            unsafe { msg_send![&items, objectAtIndex: i] };
        let Some(item) = item else {
            continue;
        };
        let url_str = item.stringForType(unsafe { NSPasteboardTypeFileURL });
        let Some(url_str) = url_str else {
            continue;
        };
        let s = url_str.to_string();
        let path = if s.starts_with("file://") {
            let path_part = s.trim_start_matches("file://");
            urlencoding::decode(path_part)
                .map(|s| s.into_owned())
                .unwrap_or_else(|_| path_part.to_string())
        } else {
            s
        };
        if !path.is_empty() {
            paths.push(path);
        }
    }
    paths
}

/// Call from main thread when app is activated (e.g. RunEvent::Resumed).
/// If the general pasteboard contains file URLs (e.g. from Finder Services),
/// runs forced silent service flow and returns true.
#[allow(dead_code)]
pub fn try_emit_open_files_from_pasteboard(app_handle: &tauri::AppHandle) -> bool {
    if MainThreadMarker::new().is_none() {
        return false;
    }
    let pboard = NSPasteboard::generalPasteboard();
    let paths = read_file_paths_from_pasteboard(&pboard);
    if paths.is_empty() {
        return false;
    }
    if crate::silent_open::try_start_silent_background_forced(app_handle, &paths) {
        return true;
    }
    if crate::silent_open::try_start_silent_compress_background_forced(app_handle, &paths) {
        return true;
    }
    false
}

/// No-op: full NSServices provider would require native Obj-C. Pasteboard-on-Resumed is used instead.
#[allow(dead_code)]
pub fn register_services_provider(_app_handle: &tauri::AppHandle) {}

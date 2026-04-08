//! macOS: NSServices с `NSMessage` = `openFiles` вызывает у делегата
//! `application:openFiles:`, а не `application:openURLs:` (который обрабатывает Tao/Tauri).
//! Без этого `RunEvent::Opened` не приходит — тихий режим не запускается, остаётся только окно.
//!
//! Tao регистрирует делегат как класс `TaoAppDelegateParent` и **не** реализует
//! `application:openFiles:`. Ставим IMP на этот класс по имени как можно раньше
//! (конец `setup` в Tauri — раньше, чем пользовательский колбэк `RunEvent::Ready`).

use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use objc2::ffi::{class_addMethod, class_getInstanceMethod, method_setImplementation};
use objc2::msg_send;
use objc2::runtime::{AnyClass, AnyObject, Sel};
use objc2::sel;
use objc2_app_kit::{NSApplication, NSApplicationDelegateReply};
use objc2_foundation::{MainThreadMarker, NSArray, NSString};
use tauri::AppHandle;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

pub fn register_app_handle(app: AppHandle) {
    let _ = APP_HANDLE.set(app);
}

fn try_add_open_files_to_class(cls: &AnyClass) -> bool {
    let open_files_sel = sel!(application:openFiles:);
    let types = CString::new("v@:@@").expect("objc types");
    let imp: objc2::runtime::Imp =
        unsafe { std::mem::transmute(application_open_files_imp as *const ()) };

    let added = unsafe {
        class_addMethod(
            cls as *const AnyClass as *mut AnyClass,
            open_files_sel,
            imp,
            types.as_ptr(),
        )
    };
    if added.as_bool() {
        return true;
    }
    let method_ptr = unsafe { class_getInstanceMethod(cls as *const AnyClass, open_files_sel) };
    if method_ptr.is_null() {
        return false;
    }
    unsafe {
        method_setImplementation(method_ptr, imp);
    }
    true
}

/// Вызывать из конца `.setup()` и из `RunEvent::Ready` / `Resumed`, пока не установится.
pub fn try_install_open_files_hook() {
    if HOOK_INSTALLED.load(Ordering::SeqCst) {
        return;
    }
    if MainThreadMarker::new().is_none() {
        return;
    }

    let tao_name = CStr::from_bytes_with_nul(b"TaoAppDelegateParent\0").expect("cstr");
    if let Some(tao_cls) = AnyClass::get(tao_name) {
        if try_add_open_files_to_class(tao_cls) {
            HOOK_INSTALLED.store(true, Ordering::SeqCst);
            return;
        }
    }

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let ns_app = NSApplication::sharedApplication(mtm);
    let Some(delegate) = ns_app.delegate() else {
        return;
    };
    let obj: &AnyObject = delegate.as_ref();
    let cls: *const AnyClass = unsafe { msg_send![obj, class] };
    if cls.is_null() {
        return;
    }
    let Some(cls_ref) = (unsafe { cls.as_ref() }) else {
        return;
    };
    if try_add_open_files_to_class(cls_ref) {
        HOOK_INSTALLED.store(true, Ordering::SeqCst);
    }
}

unsafe extern "C-unwind" fn application_open_files_imp(
    _: *mut AnyObject,
    _: Sel,
    _sender: *mut AnyObject,
    filenames: *mut AnyObject,
) {
    if filenames.is_null() {
        return;
    }
    let Some(app) = APP_HANDLE.get() else {
        return;
    };
    let filenames = &*(filenames as *mut NSArray<NSString>);
    let count = filenames.count();
    let mut paths = Vec::with_capacity(count);
    for i in 0..count {
        let s = filenames.objectAtIndex(i);
        paths.push(s.to_string());
    }
    if paths.is_empty() {
        return;
    }
    crate::handle_opened_paths_from_service(app, paths);
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let ns_app = NSApplication::sharedApplication(mtm);
    ns_app.replyToOpenOrPrint(NSApplicationDelegateReply::Success);
}

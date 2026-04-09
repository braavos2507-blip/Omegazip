# External Integrations

**Analysis Date:** 2026-03-29

## APIs & External Services

**Cloud Storage & Sync:**
- rclone - Remote storage synchronization (S3, Google Drive, OneDrive, B2, local SSH, etc.)
  - SDK/Client: System binary (`rclone` command)
  - Implementation: `src-tauri/src/lib.rs` lines 282-335
  - Commands: `rclone_list_remotes()`, `rclone_upload()`, `rclone_download()`, `rclone_available()`
  - Error handling: Command-based with stderr capture

## Data Storage

**Databases:**
- None detected - Application is file-based (processes and creates archive files)

**File Storage:**
- Local filesystem only
  - Archive format: `.oz` (proprietary OmegaZip format)
  - ZIP export capability via `export_to_zip()` function
  - Directory structure: Files extracted to specified destination folders

**Caching:**
- None detected

## Authentication & Identity

**Auth Provider:**
- None detected - Application is standalone desktop with no remote authentication
- Password-based encryption support (optional, user-provided)
  - Algorithm: ChaCha20-Poly1305 (AEAD)
  - Key derivation: Argon2 (memory-hard, salt-based)
  - Implementation: `src/crypto.rs`

## Monitoring & Observability

**Error Tracking:**
- None detected - Error handling is local, no external error reporting

**Logs:**
- None detected - Application does not have structured logging to external services
- Errors returned as strings to frontend for user display

## CI/CD & Deployment

**Hosting:**
- Distributes as standalone desktop application (Windows, macOS, Linux)
- No cloud-hosted backend

**CI Pipeline:**
- Not detected

**Distribution:**
- Tauri automatic updater not explicitly configured
- Manual distribution via GitHub releases or installers

## Platform Integration Points

**macOS Specific:**
- **Finder Services Integration:** `src-tauri/src/macos_services.rs`
  - Reads file URLs from pasteboard when app is activated (NSPasteboard)
  - File type: `NSPasteboardTypeFileURL`
  - Emits `open-files` event when files detected
  - Workaround for NSServices provider (full implementation would require Obj-C)

- **Pasteboard Monitoring:**
  - Triggered on `RunEvent::Resumed` (app activation)
  - URL decoding for file:// protocol paths via `urlencoding` crate

- **File Dialogs:**
  - macOS: Uses native `rfd` crate (cross-platform, native UI)
  - Windows/Linux: Uses Tauri dialog plugin

**Windows & Linux:**
- Command-line argument handling: Detects `.oz` files passed as arguments
- Drag-and-drop file handling: Supports dropping files into window
- File manager context menu: Scripts provided (`scripts/install-context-menu.sh`)

**Cross-Platform File Handling:**
- Drag-and-drop events: `WindowEvent::DragDrop` emits `drop-files` event
- File picker dialogs: Platform-specific implementations
- File associations: `.oz` file type registered with OS

## Webhooks & Callbacks

**Incoming:**
- None detected

**Outgoing:**
- None detected

## Desktop/OS Integration

**File System Operations:**
- Tauri plugin `tauri-plugin-fs` 2 - Low-level file operations
- Window drag-drop enabled in `tauri.conf.json`

**Clipboard Integration (macOS):**
- Reads from general pasteboard (NSPasteboard)
- Decodes file URLs from Finder Services
- Implementation: `src-tauri/src/macos_services.rs` lines 11-41

**Window Management:**
- Single window configuration: 540x580 pixels
- Drag-drop enabled: `"dragDropEnabled": true`

## Environment Configuration

**Required env vars:**
- None detected - Application is self-contained
- rclone configuration managed separately (rclone has its own config files)

**Optional configuration:**
- rclone remotes: User configures via `rclone config` command
- Archive password: User-provided at compression/decompression time

**Secrets location:**
- None detected - No external secrets or API keys
- Encryption keys derived from user passwords via Argon2

## IPC & Event System

**Frontend-Backend Communication:**
- Tauri IPC protocol (WebSocket-based)
- Commands (frontend → backend):
  - `compress()`, `compress_advanced()` - Start compression
  - `decompress()`, `decompress_with_password()` - Start decompression
  - `archive_info()`, `list_archive()` - Query archive
  - `export_to_zip()` - Convert to standard ZIP
  - `pick_file_or_folder()`, `pick_folder()`, `pick_save_file()`, `pick_oz_file()` - File dialogs
  - `rclone_list_remotes()`, `rclone_upload()`, `rclone_download()`, `rclone_available()` - Cloud sync

- Events (backend → frontend):
  - `compress-progress` - Progress updates during compression
  - `decompress-progress` - Progress updates during decompression
  - `open-files` - Files opened via drag-drop, CLI args, or Finder Services
  - `drop-files` - Files dropped into window

---

*Integration audit: 2026-03-29*

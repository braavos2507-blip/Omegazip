# Technology Stack

**Analysis Date:** 2026-03-29

## Languages

**Primary:**
- Rust (1.93.1) - Core compression engine, archiver library, desktop application backend
- JavaScript/HTML - UI frontend (vanilla, no framework)

**Secondary:**
- Shell (bash) - Build scripts, platform integration, benchmarking

## Runtime

**Environment:**
- Tauri 2.x - Cross-platform desktop runtime (Windows, macOS, Linux)
- Rust edition 2021

**Package Manager:**
- npm - JavaScript dependencies
- Cargo - Rust dependencies
- Lockfile: Cargo.lock present

## Frameworks

**Core:**
- Tauri 2 - Desktop application framework, IPC bridge between frontend and backend
- Tauri Plugin System - Plugin architecture for UI integration

**Plugin Libraries:**
- `tauri-plugin-dialog` 2.6.0 - File/folder picker dialogs
- `tauri-plugin-fs` 2 - File system operations
- `tauri-build` 2 - Build-time configuration

**Platform-Specific:**
- objc2 0.6 - macOS Objective-C interop (clipboard integration)
- objc2-app-kit 0.3 - macOS NSApplication/NSPasteboard APIs
- objc2-foundation 0.3 - macOS Foundation framework bindings
- rfd 0.15 - Rust file dialog (macOS native)

## Key Dependencies

**Critical:**
- `zstd` 0.13 - Zstandard compression codec
- `lz4_flex` 0.11 - LZ4 compression codec
- `xz2` 0.1 - XZ/LZMA compression codec (bindings)
- `zip` 0.6 - ZIP archive format export/import
- `chacha20poly1305` 0.10 - AEAD encryption (ChaCha20-Poly1305)
- `sha2` 0.10 - SHA-256 hashing
- `argon2` 0.5 - Argon2 key derivation
- `reed-solomon-erasure` 6.0 - Reed-Solomon recovery codes

**Infrastructure:**
- `serde` 1, `serde_json` 1 - Serialization (config, archive metadata)
- `rayon` 1.10 - Parallel iteration for multi-threaded compression
- `walkdir` 2 - Recursive directory traversal
- `anyhow` 1, `thiserror` 1 - Error handling
- `rand` 0.8 - Random number generation (encryption)
- `crc32fast` 1.3 - CRC32 checksums
- `zeroize` 1.7 - Secure memory zeroing (password/key cleanup)

**URL/Encoding:**
- `urlencoding` 2 - URL decoding for macOS pasteboard file paths

## Configuration

**Environment:**
- No `.env` file required
- Configuration via Tauri config: `src-tauri/tauri.conf.json`
- Build-time configuration managed by `tauri-build`

**Build:**
- `tauri.conf.json` - Application metadata, window configuration, bundling options
- Development server: `http://localhost:1420` for hot reload
- Frontend distribution: `ui/` directory
- Production: Bundled binaries for Windows/macOS/Linux

**Tauri Configuration Details:**
- Window size: 540x580 pixels (resizable)
- Drag-drop enabled
- File associations: `.oz` files (OmegaZip archives)
- macOS minimum: macOS 10.15
- Icon bundling from `icons/` directory
- Cross-platform: Windows, macOS, Linux via same codebase

## Platform Requirements

**Development:**
- Rust 1.93+ (rustup recommended)
- Node.js 16+ (for Tauri CLI)
- npm or yarn
- macOS: Xcode command line tools (for native compilation)
- Linux: Development headers (build-essential, libssl-dev, etc.)
- Windows: Visual Studio Build Tools or MSVC

**Production:**
- Windows 10+, macOS 10.15+, Linux (glibc-based distributions)
- rclone binary available in PATH (for cloud integration, optional)
- Desktop environment with file manager support

## Compilation & Optimization

**Release Profile:**
- Link-time optimization (LTO) enabled
- Single codegen unit (aggressive optimization)
- File: `Cargo.toml` (root level)

---

*Stack analysis: 2026-03-29*

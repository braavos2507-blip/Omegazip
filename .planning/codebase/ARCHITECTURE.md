# Architecture

**Analysis Date:** 2026-03-29

## Pattern Overview

**Overall:** Modular compression pipeline with desktop GUI frontend (Tauri + HTML/CSS/JS) and cross-platform native backend (Rust). The system implements a 4-stage compression processing pipeline with multiple encoding strategies, deduplication, encryption, and recovery mechanisms.

**Key Characteristics:**
- Layered architecture: UI (Tauri), IPC layer (command invocations), core compression library (libomegazip), and CLI
- Multi-stage compression pipeline: semantic analysis → preprocessing → codec selection → block storage/dedup
- Support for both v1 (simple) and v2 (advanced with chunking, solid, encryption, recovery) archive formats
- Platform-specific UI implementations (macOS uses rfd native dialogs, Windows/Linux use Tauri plugin dialogs)
- Optional external script integration for analysis, preprocessing, and MIME detection

## Layers

**Frontend UI (Tauri/HTML):**
- Purpose: Desktop application interface for file compression/decompression, archive inspection, cloud backup via rclone
- Location: `ui/index.html`, `/Users/busyden/Documents/Projects/OmegaZip/UI/index.html`
- Contains: HTML, inline CSS, embedded JavaScript logic for UI state and user interactions
- Depends on: Tauri v2 API for IPC with backend commands
- Used by: End users; provides drag-drop support, file pickers, progress tracking, archive info display

**Tauri Backend / IPC Layer:**
- Purpose: Bridge between frontend and Rust compression library; manages file dialogs, rclone integration, event emission for progress
- Location: `src-tauri/src/main.rs` (entry point), `src-tauri/src/lib.rs` (Tauri command handlers)
- Contains: Tauri command handlers (compress, decompress, file pickers, rclone operations), event emitters, platform-specific dialog code
- Depends on: omegazip library (compression core), tauri v2, tauri-plugin-dialog, tauri-plugin-fs
- Used by: Frontend UI communicates only through Tauri invoke system

**Core Compression Library (libomegazip):**
- Purpose: Implements the 4-stage compression pipeline and all compression/decompression logic
- Location: `src/lib.rs` (public API), `src/main.rs` (CLI), Rust modules in `src/`
- Contains: Compression pipeline, codecs, deduplication, encryption, recovery, archive format handling
- Depends on: zstd, lz4_flex, xz2, chacha20poly1305, argon2, reed-solomon-erasure, serde, rayon
- Used by: Tauri backend (via Rust FFI), CLI binary

**CLI Binary:**
- Purpose: Command-line interface to compression functions without GUI
- Location: `src/main.rs`
- Contains: Argument parsing, CLI command implementations (compress, decompress, info, list, export-zip, repo operations)
- Depends on: libomegazip core
- Used by: System integration (context menu, shell scripts, automated tools)

## Data Flow

**Compression Flow (compress_to_path_with_options):**

1. **Scanning & File Collection** → Walk source tree, collect file paths and sizes
2. **Semantic Analysis** → For each file: analyze_bytes() detects context (Text/Binary/Realtime/Archive) via magic bytes, entropy, optional script
3. **Preprocessing** → If PDF detected and OMEGAZIP_PREPROCESS_PDF script available: run external preprocessor
4. **Chunking (optional)** → If chunked=true: split file into 64 KiB chunks using chunks() function
5. **Codec Selection** → best_compress() tests candidates (Dense, Balanced, Fast) and picks smallest result
6. **Deduplication** → BlockStore tracks compressed chunks by SHA256 hash; first occurrence is stored, duplicates are dereferenced
7. **Encryption (optional)** → If password provided: derive_key(password, salt) via Argon2id, encrypt_block() each chunk with ChaCha20-Poly1305
8. **Recovery Encoding (optional)** → If recovery=true: group 16 blocks into stripes, encode_stripe() computes 2 parity blocks per stripe (Reed-Solomon)
9. **Solid Stream (optional)** → If solid=true: compress all files into one continuous stream with offsets tracked in manifest
10. **Manifest Creation** → JSON metadata (file list, chunk/solid references, algorithm IDs, recovery info)
11. **Archive Writing** → Write magic (OMEGAZIP\x02) + flags + salt + encrypted/plain manifest + block data + recovery section

**Decompression Flow (decompress_to_path_with_options):**

1. **Archive Reading** → Read magic, flags, salt; parse manifest
2. **Key Derivation (if encrypted)** → derive_key(password, salt) to reconstruct cipher key
3. **Manifest Parsing** → Extract file list, chunk references or solid stream offsets
4. **Block Reconstruction (if recovery)** → If parity blocks present and corruption detected, decode_stripe() recovers lost blocks
5. **Decryption** → If payload encrypted, decrypt_block() each chunk
6. **Decompression** → Decompress each block using codec identified in manifest
7. **File Reconstruction** → For chunked: concatenate chunks; for solid: extract by offset; recreate directory structure
8. **Output** → Write files to destination directory

**State Management:**

- **Progress Emission:** During compress/decompress, Progress struct emitted via app.emit() to frontend at scan/compress/write phases
- **Error Handling:** All operations return Result<T, String>; errors bubble to frontend for user-facing messages
- **Manifest as Source of Truth:** JSON manifest stored in archive contains all reconstruction information; decryption/decompression is deterministic given manifest + key

## Key Abstractions

**Codec Enum:**
- Purpose: Represents compression algorithm choice
- Examples: Codec::Dense (huffman), Codec::Balanced (zstd+lz4), Codec::Fast (lz4), Codec::MaxRatio (xz), Codec::Store (no compression)
- Pattern: Dispatches to codec_backend module; selected dynamically based on DataContext analysis

**DataContext Enum:**
- Purpose: Semantic classification of file content influencing codec selection
- Examples: Text → Codec::Dense, Binary → Codec::Balanced, Realtime → Codec::Fast, Archive → Codec::MaxRatio
- Pattern: Determined in analyze_bytes() via entropy + magic + optional script; drives best_compress() logic

**BlockStore:**
- Purpose: Global deduplication cache for compressed blocks
- Pattern: HashMap<SHA256 hash, (algo_id, compressed_data)> with Bloom filter for rapid negative lookups
- Location: `src/dedup.rs`
- Used in: Pipeline to avoid storing duplicate chunks

**CompressOptions:**
- Purpose: Configuration struct for flexible compression behavior
- Fields: chunk_size, solid, password, recovery_parity, preset, parallel, progress callback
- Pattern: Separates simple compress_to_path() (v1, defaults) from advanced compress_to_path_with_options() (v2)

**ArchiveInfo & ManifestEntry:**
- Purpose: Metadata structures for archive inspection without full decompression
- Pattern: list_archive() and archive_info() parse manifest only, return file paths and stats
- Location: `src/pipeline.rs`

## Entry Points

**Tauri Command: compress_advanced**
- Location: `src-tauri/src/lib.rs` line 19
- Triggers: Frontend button click (Compress Advanced)
- Responsibilities: Validate options, invoke omegazip::compress_to_path_with_options(), emit progress events, return file count or error

**Tauri Command: decompress_with_password**
- Location: `src-tauri/src/lib.rs` line 55
- Triggers: Frontend decompress button
- Responsibilities: Invoke omegazip::decompress_to_path_with_options(), emit progress events, handle password-protected archives

**CLI: main()**
- Location: `src/main.rs` line 12
- Triggers: Terminal invocation with subcommands (compress, decompress, info, list, export-zip, repo)
- Responsibilities: Parse CLI args, dispatch to library functions, format and print results

**Tauri Event Listeners (Frontend):**
- Location: `UI/index.html` (JavaScript inline)
- Listens to: "compress-progress", "decompress-progress", "open-files", "drop-files"
- Responsibilities: Update UI progress bar, handle drag-drop files, populate file input

**macOS-Specific: macos_services.rs**
- Location: `src-tauri/src/macos_services.rs`
- Triggers: App resume, URL open, file drag-drop
- Responsibilities: Emit "open-files" event from clipboard or file URLs; integrates with Finder

## Error Handling

**Strategy:** Rust Result<T, String> propagated through all layers; frontend catches errors and displays messages to user

**Patterns:**
- **Compression errors:** Return Err with context (file not found, codec failure, crypto errors)
- **Decompression errors:** Detect corruption via CRC32; attempt recovery via Reed-Solomon if enabled; return readable error
- **Encryption errors:** Argon2 key derivation, ChaCha20 nonce/tag validation failures
- **File I/O:** Wrapped with anyhow/thiserror; context preserved for debugging
- **Frontend error display:** Catch in Tauri handlers, emit error message, show in UI toast/dialog

## Cross-Cutting Concerns

**Logging:** No structured logging framework; error messages via Result Err variant; debug output via println! in CLI (controlled by environment)

**Validation:**
- Input: File existence check, path validation
- Archive: Magic byte check, manifest JSON parsing, recovery stripe validation
- Crypto: Password length/encoding, salt size (16 bytes), nonce uniqueness (random per block)

**Authentication:** Optional password-based; no user/session management. Password → Argon2id KDF → ChaCha20-Poly1305 AEAD

**Parallelism:**
- Codec selection: rayon parallelizes block compression testing
- Pipeline: Optional parallel=true enables multi-threaded block compression via rayon
- Dedup: Block hash calculation parallelized for large files

**Platform Abstraction:**
- macOS: Uses rfd native file dialogs (objc2 bindings to AppKit)
- Windows/Linux: Tauri plugin dialogs
- File I/O: Standard Rust std::fs, walkdir for traversal

---

*Architecture analysis: 2026-03-29*

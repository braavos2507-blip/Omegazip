# Coding Conventions

**Analysis Date:** 2026-03-29

## Language & Edition

**Primary Language:** Rust 2021 edition

**Secondary Languages:**
- Objective-C (macOS integration only)
- Shell scripts (build/integration scripts)

## Naming Patterns

**Files:**
- Snake_case with `.rs` extension: `pipeline.rs`, `dedup.rs`, `analyzer.rs`
- Module organization: one file per logical module

**Functions:**
- Snake_case for all functions: `analyze_bytes()`, `best_compress()`, `random_salt()`
- Public API functions prefixed with `pub fn`
- Private helper functions use lowercase without pub keyword
- Impl methods follow same snake_case convention

**Variables & Constants:**
- Variables: snake_case: `chunk_size`, `data_ready`, `max_len`
- Constants: UPPERCASE_SNAKE_CASE: `DEFAULT_CHUNK_SIZE`, `BLOOM_K`, `STRIPE_DATA_SHARDS`
- Local struct instances: snake_case: `opts`, `block_ref`, `snapshot_id`

**Types & Structs:**
- Struct names: PascalCase: `BlockStore`, `Progress`, `ProgressPhase`, `Codec`
- Enum variants: PascalCase: `DataContext::Text`, `Preset::Fast`, `Codec::Dense`
- Type aliases: PascalCase (if used)
- Lifetime parameters: single lowercase letters: `'a`
- Generic parameters: PascalCase: `T`

**Modules:**
- Module names: lowercase_snake_case
- Module organization in `lib.rs`: flat list of `pub mod` declarations

## Doc Comments & Documentation

**Module-Level Docs:**
- Use `//!` at file top for module documentation
- Written in Russian (per codebase convention): `//! Кодирование по энтропии: Huffman`
- Describe purpose, algorithm, and key components

**Public Function Docs:**
- Document all public functions with `///` comments
- Include Russian descriptions of parameters and return values
- Examples:
  - From `crypto.rs`: `/// Генерирует соль для KDF.`
  - From `recovery.rs`: `/// По списку блоков (переменной длины) считает parity-блоки для одной полосы.`

**Implementation Comments:**
- Use `//` for inline implementation notes
- Comments placed above relevant code blocks
- Example sections use `// ============== Section Name ==============` format

## Code Style

**Formatting:**
- Standard Rust formatting (implied rustfmt compliance)
- Line wrapping: logical breaks at 100-120 characters
- Indentation: 4 spaces (Rust default)
- Consistent spacing around operators and punctuation

**Linting:**
- Assumes cargo check passes
- No explicit linter config files found
- Standard Rust clippy warnings respected (through build scripts)

**Imports Organization:**
- Order: standard library, external crates, internal modules
- Example from `pipeline.rs`:
  ```rust
  use std::io::Write;
  use std::path::Path;
  use crate::chunked::chunks;
  use crate::codec::{best_compress, decompress, codec_id, codec_from_id, Codec};
  ```
- Grouped by module/category with blank lines between groups

**Module Exports:**
- Use `pub use` in `lib.rs` to re-export public API
- Example from `lib.rs`:
  ```rust
  pub use pipeline::{
      compress_to_path, compress_to_path_with_options,
      decompress_to_path, decompress_to_path_with_password, decompress_to_path_with_options,
      export_to_zip, archive_info, list_archive,
      CompressOptions, Progress, ProgressPhase, Preset, ArchiveInfo,
  };
  ```

## Error Handling

**Strategy:** Multi-layered error handling combining Result types and custom error types

**Patterns:**

1. **Box<dyn std::error::Error + Send + Sync>** - For operations that may fail in multiple ways
   - Used in public API functions like `backup()`, `repo_init()`
   - Simplest propagation pattern
   - Example from `repo.rs`:
     ```rust
     pub fn repo_init(path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
         fs::create_dir_all(path.join("chunks"))?;
         fs::create_dir_all(path.join("snapshots"))?;
         Ok(())
     }
     ```

2. **std::io::Result<T>** - For I/O operations
   - Used when error is primarily I/O: `analyze_file()`, `read_preprocess_result()`
   - Example from `analyzer.rs`:
     ```rust
     pub fn analyze_file(path: &Path) -> std::io::Result<AnalysisResult> {
         let mut f = File::open(path)?;
         // ...
     }
     ```

3. **Result<T, String>** - For Tauri command handlers
   - Used exclusively in `main.rs` Tauri commands
   - Error messages converted to strings for frontend
   - Example from `main.rs`:
     ```rust
     #[tauri::command]
     fn compress(source: PathBuf, archive_path: PathBuf) -> Result<u32, String> {
         omegazip::compress_to_path(&source, &archive_path).map_err(|e| e.to_string())
     }
     ```

4. **Option<T>** - For optional or null-fallback paths
   - Used for utility functions with graceful degradation
   - Example from `analyzer.rs`:
     ```rust
     fn script_mime(path: &Path) -> Option<String> {
         let cmd = std::env::var("OMEGAZIP_MIME_SCRIPT").ok()?;
         // ... returns None if env var not set or script fails
     }
     ```

5. **map_err()** - Error type conversion
   - Converts between error types when needed
   - Example: `.map_err(|e| e.to_string())` in Tauri commands

**Special Cases:**
- `.expect()` used for "should never happen" conditions
  - Example from `crypto.rs`: `.expect("argon2")` on Argon2 hashing
  - Only when error indicates programmer error, not user/data error

## Function Design

**Size Guidelines:**
- Functions typically 15-50 lines
- Larger functions (100+ lines) break down into helpers: `pipeline.rs` at 937 lines uses many internal functions

**Parameters:**
- Ownership-aware: `&Path` for reads, `PathBuf` for ownership
- Optional parameters: wrapped in `Option<T>`: `password: Option<String>`
- Callback functions: wrapped in `Arc<dyn Fn(...) + Send + Sync>` for multithreading
- Example from `pipeline.rs`:
  ```rust
  pub struct CompressOptions {
      pub chunk_size: Option<usize>,
      pub password: Option<String>,
      pub progress: Option<Arc<dyn Fn(Progress) + Send + Sync>>,
  }
  ```

**Return Values:**
- Single values: direct type or `Result<T, E>`
- Multiple values: struct wrapping (e.g., `AnalysisResult`)
- Tuples for related pairs: `(Codec, Vec<u8>)` from `best_compress()`

## Module Design

**Public Exports:**
- Public functions and types explicitly marked `pub`
- Re-exported in `lib.rs` for public API surface
- Example from `dedup.rs`:
  ```rust
  pub struct BlockStore { /* ... */ }
  pub fn new() -> Self { /* ... */ }
  pub fn add_chunk(&mut self, data: &[u8], /* ... */) -> BlockRef { /* ... */ }
  ```

**Private Helpers:**
- No `pub` keyword for internal utilities
- Example from `analyzer.rs`:
  ```rust
  fn entropy(data: &[u8]) -> f64 { /* ... */ }
  fn text_ratio(data: &[u8]) -> f64 { /* ... */ }
  fn script_mime(path: &Path) -> Option<String> { /* ... */ }
  ```

**Struct Fields:**
- Public fields on simple data containers: `struct BlockRef { pub hash: [u8; 32], pub algo: u8 }`
- Private fields with getters for complex logic: `struct BlockStore { blocks: HashMap<>, bloom: Vec<u8> }`

## Serialization & Data Formats

**JSON:**
- Uses `serde` with `#[derive(serde::Serialize, serde::Deserialize)]`
- Conditional serialization: `#[serde(skip_serializing_if = "Option::is_none")]`
- Example from `pipeline.rs`:
  ```rust
  #[derive(serde::Serialize, serde::Deserialize)]
  struct ManifestEntry {
      path: String,
      #[serde(default)]
      algo: u8,
      #[serde(skip_serializing_if = "Option::is_none")]
      chunks: Option<Vec<ChunkRef>>,
  }
  ```

## Path Handling

**Convention:**
- Use `&Path` for references (immutable paths)
- Use `PathBuf` for owned paths (when returning or storing)
- Operations: `path.join()`, `path.exists()`, `path.parent()`, `path.to_string_lossy()`

## Configuration & Constants

**Location:** Top of modules or in `lib.rs`
- Example: `const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;` in `chunked.rs`
- Example: `const BLOOM_BITS: usize = 256 * 1024 * 8;` in `dedup.rs`
- Configuration values in environment variables for external integrations (rclone, OMEGAZIP_MIME_SCRIPT)

## Parallel Processing

**Framework:** Rayon for data parallelization
- Used in compression pipeline when `parallel: true` in `CompressOptions`
- Example from `pipeline.rs`: `.par_iter()` usage in compression loops
- Opt-in: controlled by `CompressOptions { parallel: bool }`

## Unsafe Code

**Usage:** Minimal, only in macOS integration
- File: `src-tauri/src/macos_services.rs`
- Unsafe blocks for Objective-C FFI via `objc2` crate
- Protected with `#[cfg(target_os = "macos")]`

---

*Convention analysis: 2026-03-29*

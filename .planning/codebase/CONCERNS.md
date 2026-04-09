# Codebase Concerns

**Analysis Date:** 2026-03-29

## Critical Error Handling Issues

**Unsafe unwrap() calls in binary parsing:**
- Issue: Multiple `unwrap()` calls in archive parsing code without bounds checking, can panic on malformed archives
- Files: `src/pipeline.rs` (lines 347, 364, 546, 582, 599, 602, 629, 636, 773, 791, 812, 822, 841, 850, 895, 930)
- Impact: A malformed .oz archive with incorrect length fields can cause application crash instead of graceful error message
- Fix approach: Replace all `try_into().unwrap()` and `get().unwrap()` with proper error handling using `?` operator or explicit bounds checks. Validate archive structure before parsing.

**Unsafe slice access in decompression:**
- Issue: Lines 719, 837 use unchecked slice ranges `stream_raw[o..o + l]` and `stream_raw[o..o + l]` without validating offset+length
- Files: `src/pipeline.rs` (lines 719, 837)
- Impact: If solid stream metadata contains invalid offsets (corrupted archive or crafted attack), can cause panic or undefined behavior
- Fix approach: Use `.get()` with proper bounds checking: `stream_raw.get(o..o + l).ok_or("Invalid offset")?`

**Decrypt failure unwrap:**
- Issue: Line 822 uses `.unwrap()` on decryption result: `decrypt_block(k, &raw).unwrap()`
- Files: `src/pipeline.rs` (line 822)
- Impact: Bad ciphertext or corrupted encrypted block will panic instead of reporting error
- Fix approach: Use proper error propagation with `?` instead of unwrap

## Security Concerns

**Password handling in CLI:**
- Issue: Password read from command-line args is visible in process list and shell history
- Files: `src/main.rs` (lines 191, 239, 278)
- Current mitigation: Supports `--password-file` and `OMEGAZIP_PASSWORD` env var as alternatives
- Recommendations: Document that `--password` should only be used in scripts where args are not visible. Add warning message when password is passed via CLI.

**Inadequate validation of archive manifest:**
- Issue: JSON manifest is deserialized but file paths are not validated for directory traversal attacks
- Files: `src/pipeline.rs` (line 793: `serde_json::from_slice(&data[pos..pos + manifest_len])`)
- Impact: Attacker could craft .oz archive with paths like `../../system/file` that overwrites arbitrary locations when extracted
- Fix approach: After deserialization, validate that all entry paths are normalized and do not contain `..` or absolute paths. Use `PathBuf` and `canonicalize()` with jail check.

**Hex parsing fallback to zero hash:**
- Issue: Lines 723, 731, 841, 849 use `hex_to_hash(...).unwrap_or([0u8; 32])`
- Files: `src/pipeline.rs` (lines 723, 731, 841, 849)
- Impact: If a manifest has invalid hex hashes, they silently become zero-hash, potentially retrieving wrong blocks or worse
- Fix approach: Return error instead of silently defaulting to zero-hash. Archive integrity is compromised if hashes are malformed.

**ChaCha20 nonce reuse potential:**
- Issue: Each block encryption uses `OsRng` for nonce, but if RNG fails silently, nonce could be reused
- Files: `src/crypto.rs` (lines 34, 37)
- Impact: Nonce reuse with same key breaks ChaCha20-Poly1305 security completely
- Fix approach: Make `fill_bytes` fail explicitly and propagate error instead of expecting it to always succeed

## Test Coverage Gaps

**Untested malformed archive handling:**
- What's not tested: Archives with truncated headers, invalid JSON manifests, mismatched block hashes
- Files: `src/pipeline.rs` (all parsing functions: `decompress_to_path`, `export_to_zip`, `archive_info`, `list_archive`)
- Risk: Panics on real-world corrupted archives instead of reporting errors
- Priority: High

**Untested recovery stripe reconstruction:**
- What's not tested: Actual recovery from missing/corrupted blocks using Reed-Solomon parity
- Files: `src/pipeline.rs` (lines 673-700 recovery path), `src/recovery.rs` (decode_stripe)
- Risk: Recovery feature may not work when needed; only encode_stripe is exercised
- Priority: High

**Untested encryption/decryption with wrong password:**
- What's not tested: Attempting to decrypt with wrong password (should fail cleanly, not panic)
- Files: `src/pipeline.rs` (decompress path), `src/crypto.rs` (decrypt_block)
- Risk: UX degradation - unclear error message or crash when user enters wrong password
- Priority: Medium

**Untested solid stream mode:**
- What's not tested: Compress/decompress with `--solid` flag, offset/length correctness
- Files: `src/pipeline.rs` (compress_solid, decompress_solid sections)
- Risk: Silent data corruption if offsets calculated incorrectly
- Priority: High

## Fragile Areas

**Binary format parsing (manual byte offset tracking):**
- Files: `src/pipeline.rs` (decompress, archive_info, list_archive, export_to_zip functions)
- Why fragile: Multiple variables tracking position in buffer (`pos`), many manual `[pos..pos+N]` slices prone to off-by-one errors
- Safe modification: Add bounds check helper function before every slice access. Consider using `nom` parser combinator library for format parsing
- Test coverage: Only basic round-trip tests; no malformed archive tests

**Huffman tree building with single-symbol edge case:**
- Files: `src/huffman.rs` (lines 45-52)
- Why fragile: Special case handling for heap with one element creates tree with single child instead of proper root
- Safe modification: Add test for single-symbol data encoding/decoding round-trip
- Test coverage: Unknown

**Bloom filter implementation:**
- Files: `src/dedup.rs` (bloom_pos, bloom_may_contain, bloom_insert)
- Why fragile: Fixed BLOOM_BITS size (256KB) with only k=4 hash functions may have high false positive rate but code assumes it's efficient
- Safe modification: Add configurable bit size and k value; add false positive rate calculation/logging
- Test coverage: Only implicit through dedup tests

## Performance Bottlenecks

**Sequential recovery stripe encoding:**
- Problem: Loop at `src/pipeline.rs` (lines 377-392) encodes stripes one by one instead of in parallel
- Files: `src/pipeline.rs` (compress_solid function)
- Cause: Reed-Solomon encoding is CPU-intensive but not parallelized
- Improvement path: Use rayon to parallelize stripe encoding similar to file compression

**Unnecessary compression trials:**
- Problem: `best_compress()` tries multiple codecs sequentially even when result is already good enough
- Files: `src/codec.rs` (lines 47-68)
- Cause: No early exit or threshold (e.g., if result < 50% original size, don't try others)
- Improvement path: Add configurable early-exit threshold; skip slow codecs (XZ) if faster ones already achieve good ratio

**Full file read for small analysis sample:**
- Problem: Analyzer reads only 64KB for entropy analysis but then processes entire file multiple times
- Files: `src/analyzer.rs` (line 88, reads 64KB sample), but full data passed to preprocessing
- Cause: Sample-based analysis not synchronized with actual compression data
- Improvement path: For small files (<1MB), use full file data for analysis. Cache analysis result across preprocessing and compression stages.

**Parallel compression with potential I/O contention:**
- Problem: `rayon::par_iter()` on file reading may cause excessive disk I/O
- Files: `src/pipeline.rs` (lines 234-246 parallel file read)
- Cause: No buffering or thread pool size limit
- Improvement path: Limit rayon thread count to number of physical cores; use buffered channel between reading and compression

## Scaling Limits

**In-memory block deduplication:**
- Current capacity: All blocks stored in `HashMap<[u8; 32], (u8, Vec<u8>)>` with entire compressed block in memory
- Limit: Archive with many unique chunks > available RAM will OOM
- Scaling path: Implement file-backed block store; spill compressed blocks to temporary files when memory threshold exceeded

**Bloom filter fixed size:**
- Current capacity: 256KB Bloom filter hardcoded, sufficient for ~2-3M unique blocks before saturation
- Limit: Large archives with millions of unique chunks may have high false positive rate
- Scaling path: Make Bloom filter size configurable as percentage of available RAM; monitor actual false positive rate

**Manifest JSON in memory:**
- Current capacity: Entire manifest loaded into memory as Vec<ManifestEntry>
- Limit: Archive with millions of files will use significant memory for manifest alone
- Scaling path: Stream manifest parsing; keep only active file entries in memory

## Dependencies at Risk

**Old MSRV and potential compatibility issues:**
- Risk: `reed-solomon-erasure 6.0` may not be maintained; verify compatibility with current rustc
- Impact: If crate unmaintained, security fixes may not be available
- Migration plan: Monitor crate status; have fallback to different Reed-Solomon implementation if needed

**Command-line dependency on external tools:**
- Risk: `rclone` integration assumes rclone is installed and in PATH
- Impact: Cloud sync features fail silently or with unclear error messages if rclone missing
- Migration plan: Bundle rclone or use Rust-native cloud libraries for core providers (AWS S3, etc)

## Missing Critical Features

**No integrity verification before decompression:**
- Problem: No way to verify archive is not corrupted before attempting full extraction
- Blocks: Users cannot validate archives before processing
- Fix approach: Implement `verify` command that reads all blocks, checks CRCs, and verifies recovery parity without decompressing

**No incremental/resume support:**
- Problem: Compression and decompression cannot be resumed if interrupted
- Blocks: Users cannot compress/decompress very large archives reliably
- Fix approach: Add checkpoint system; save progress state periodically

**No archival metadata preservation:**
- Problem: File permissions, timestamps, ownership are not stored in .oz format
- Blocks: Cannot restore archives with exact file metadata
- Fix approach: Add optional metadata section to format; extend ManifestEntry with timestamps and permissions

**No bulk file operation (add/remove/replace files in existing archive):**
- Problem: Archives are immutable; cannot update single files without re-creating entire archive
- Blocks: Archives cannot be used for incremental backups
- Fix approach: Design v3 format supporting append-only writes; implement add/remove/update operations

## Test Coverage Gaps (Expanded)

**Archive format edge cases:**
- What's not tested: Empty archives, single large file, files with unusual names (unicode, special chars)
- Files: Format tests would need to be added to test suite
- Risk: Format incompatibility or crashes on edge cases
- Priority: Medium

**Cross-platform path handling:**
- What's not tested: Path separator handling on Windows vs Unix, relative/absolute path normalization
- Files: `src/pipeline.rs` (path handling with `strip_prefix`, `canonicalize`)
- Risk: Archives created on one platform may not extract correctly on another
- Priority: Medium

**Concurrent/multi-threaded corruption handling:**
- What's not tested: What happens if archive file is modified while Tauri app is reading it
- Files: `src-tauri/src/lib.rs`, `src/pipeline.rs` (file I/O)
- Risk: Partial reads, data corruption, or hangs
- Priority: Low (race condition mitigation would require OS-level file locking)

---

*Concerns audit: 2026-03-29*

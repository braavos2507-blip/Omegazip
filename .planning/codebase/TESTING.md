# Testing Patterns

**Analysis Date:** 2026-03-29

## Current State

**Testing Status:** Minimal inline testing
- No dedicated test framework configured (Jest, Vitest, pytest, etc.)
- Only one test module found in entire codebase
- No test files in separate directory structure
- No CI/CD test pipeline configured

## Test Framework

**In-Use:**
- Native Rust `#[test]` attribute and `#[cfg(test)]` modules
- Assertions via standard `assert!()`, `assert_eq!()`, `assert_ne!()`

**Location of Single Test:**
- `src/chunked.rs` - lines 16-28

**Current Test Count:** 1 test function

**Run Commands:**
```bash
cargo test                     # Run all tests (./src/ and ./src-tauri/src/)
cargo test --lib            # Run library tests only
cargo test chunked          # Run specific test module
```

## Test File Organization

**Pattern:** Inline tests (not separated)

**Location:** Co-located with implementation
- Test module declared at end of same file as implementation
- Uses `#[cfg(test)]` attribute to exclude from release builds

**Example from `src/chunked.rs`:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunks() {
        let data: Vec<u8> = (0..200).collect();
        let c: Vec<&[u8]> = chunks(&data, 64).collect();
        assert_eq!(c.len(), 4);
        assert_eq!(c[0].len(), 64);
        assert_eq!(c[3].len(), 8);
    }
}
```

## Test Structure

**Naming Convention:**
- Test functions: `test_` prefix: `test_chunks`
- Module: `tests` (standard Rust convention)

**Assertion Pattern:**
- `assert_eq!()` for value comparisons
- `assert!()` for boolean conditions
- Example:
  ```rust
  assert_eq!(c.len(), 4);           // Check count
  assert_eq!(c[0].len(), 64);       // Check first chunk size
  assert_eq!(c[3].len(), 8);        // Check last chunk size
  ```

**Setup:** Minimal - data created directly in test
```rust
let data: Vec<u8> = (0..200).collect();
let c: Vec<&[u8]> = chunks(&data, 64).collect();
```

**Teardown:** None (implicit via scope - Rust handles memory)

## Test Coverage

**Current Coverage:** Extremely sparse (1 test across 2323 lines of code)

**Tested Areas:**
- `src/chunked.rs` - Chunking logic for 200-byte input with 64-byte chunks (1 test)

**Untested Areas (Critical gaps):**
- `src/analyzer.rs` (147 lines) - No tests for entropy calculation, MIME detection, data context analysis
- `src/codec.rs` (88 lines) - No tests for codec selection, compression, decompression
- `src/crypto.rs` (54 lines) - No tests for key derivation, encryption, decryption
- `src/dedup.rs` (107 lines) - No tests for Bloom filter, block deduplication
- `src/huffman.rs` (191 lines) - No tests for Huffman encoding/decoding
- `src/recovery.rs` (79 lines) - No tests for Reed-Solomon encoding/decoding
- `src/pipeline.rs` (937 lines) - No tests for compression pipeline, manifest handling
- `src/repo.rs` (243 lines) - No tests for backup, restore, snapshot operations
- `src/preprocess.rs` (51 lines) - No tests for PDF preprocessing
- `src-tauri/src/main.rs` (304 lines) - No tests for Tauri command handlers
- `src-tauri/src/lib.rs` (414 lines) - No tests for Tauri app lifecycle

## High-Risk Untested Areas

**Compression Pipeline** (`src/pipeline.rs` - 937 lines):
- Main compression/decompression logic never tested
- Manifest serialization/deserialization untested
- Chunk reference handling untested
- Solid archive streaming untested
- Impact: Core functionality is untested; any regression goes undetected

**Cryptography** (`src/crypto.rs` - 54 lines):
- Key derivation (Argon2) untested
- Encryption/decryption (ChaCha20-Poly1305) untested
- Impact: Security-critical code has no automated verification

**Deduplication** (`src/dedup.rs` - 107 lines):
- Block hashing untested
- Bloom filter correctness untested
- False positive rate unverified
- Impact: Core compression efficiency feature unvalidated

**Codec Selection** (`src/codec.rs` - 88 lines):
- Compression algorithm selection untested
- Best-compress heuristic untested
- Impact: Compression quality optimization untested

## Test Patterns (As Would Be Expected)

**Unit Test Structure (recommended):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_describes_behavior() {
        // Arrange: Set up test data
        let input = /* ... */;

        // Act: Call function under test
        let result = function_under_test(input);

        // Assert: Verify result
        assert_eq!(result, expected_value);
    }
}
```

**Error Testing (recommended pattern, not currently used):**
```rust
#[test]
fn test_handles_error_condition() {
    let result = function_that_may_fail();
    assert!(result.is_err());

    // Or check error message:
    match result {
        Err(e) => assert_eq!(e.to_string(), "expected error"),
        Ok(_) => panic!("should have failed"),
    }
}
```

## Dependencies & Tools

**Test Dependencies:**
- None explicitly declared for testing
- Core tests use only standard Rust test framework

**Optional/Recommended Test Tools:**
- `proptest` - for property-based testing (not in use)
- `approx` - for floating-point assertions (could help with entropy tests)
- `tempfile` - for temp file operations in backup/restore tests (not in use)

## Mock & Fixture Requirements

**Command Mocking (for Tauri):**
- Tests for `main.rs` would need to mock:
  - File dialogs (currently uses `tauri_plugin_dialog`)
  - File system operations
  - Tauri `AppHandle` for progress callbacks
  - External tools (rclone)

**Example mock strategy:**
```rust
// Would need to mock for testing rclone_list_remotes()
mod tests {
    #[test]
    fn test_rclone_list_remotes_parsing() {
        let mock_output = "remote1:\nremote2:\n";
        // Parse as if it came from rclone
        let remotes: Vec<String> = mock_output.lines()
            .map(|l| l.trim().trim_end_matches(':').to_string())
            .filter(|l| !l.is_empty())
            .collect();
        assert_eq!(remotes, vec!["remote1", "remote2"]);
    }
}
```

## Integration Testing

**Current Status:** Not configured

**What Should Be Tested:**
- End-to-end compression → decompression cycles
- Password-protected archive operations
- Chunk deduplication with real files
- Solid archive mode with multiple files
- Recovery parity block operations
- Repository backup/restore cycles
- Tauri command handlers with actual file I/O

**Where They Should Live:**
- `tests/` directory at root (not currently created)
- Use same `#[test]` attribute with full file paths

## Performance Testing

**Current Status:** Not configured

**Candidates for Performance Testing:**
- Huffman encoding speed vs compression ratio
- Bloom filter false positive rate
- Chunk deduplication efficiency
- Parallel compression speedup vs single-threaded

## Checklist for Adding Tests

When implementing new features or fixing bugs:

1. **Create test module** in same file: `#[cfg(test)] mod tests { ... }`
2. **Write unit test** with Arrange-Act-Assert pattern
3. **Test both success and failure paths**
4. **Run:** `cargo test` to verify
5. **Check coverage:** Consider using `tarpaulin` or `grcov` if added
6. **Document test intent:** Comments explaining what behavior is verified

## Known Test Gaps & Priorities

**High Priority (Security/Correctness):**
- Crypto key derivation and encryption correctness
- Dedup block hashing consistency
- Compression round-trip (compress then decompress = original)

**Medium Priority (Functionality):**
- Codec selection heuristics
- Pipeline error handling
- Archive format compatibility

**Low Priority (Performance):**
- Compression ratio benchmarks
- Parallel processing efficiency

---

*Testing analysis: 2026-03-29*

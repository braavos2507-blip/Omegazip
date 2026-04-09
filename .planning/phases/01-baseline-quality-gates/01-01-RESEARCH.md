# Phase 1 (v1.0) — RESEARCH: baseline

**Requirements:** CORE-01 — CORE-03, GUI-01 — GUI-02, QA-01  
**Ретроспектива:** работа выполнена до формализации GSD; документ фиксирует состояние на момент закрытия v1.0.

## Состояние

| REQ | Доказательство |
|-----|----------------|
| **CORE-01** | `src/main.rs` — команды `compress`, `decompress`, `export-zip`, `info`, `list`, `deps`; ошибки через `Result` / сообщения. |
| **CORE-02** | `src/compat.rs` — `safe_join`, отказ от `..` и абсолютных путей внутри архива; тест `zip_rejects_path_traversal` в `tests/compat_roundtrip.rs`. |
| **CORE-03** | `cargo test` — юнит-тесты ядра + `tests/compat_roundtrip.rs` (ZIP, tar.gz, zstd). |
| **GUI-01** | Tauri: `pick_*`, `compress` / `compress_advanced`, прогресс `compress-progress` / `decompress-progress` в `ui/`. |
| **GUI-02** | `seven_zip_status`, баннер в UI, `omegazip deps` в CLI. |
| **QA-01** | `.github/workflows/ci.yml` — `cargo test` на ubuntu + macOS; после закрытия v1.0 добавлен `cargo check` в `src-tauri`. |

## Пробелы

- Отдельного каталога планов «01» до этого момента не было — закрывается постфактум.

---
phase: 09-silent-open-extract
plan: 01
status: completed
completed_at: 2026-03-29
one-liner: Тихая распаковка одного архива из ОС без главного окна при успехе; пароль/ошибка → окно и события.
requirements_addressed:
  - GAP-01
  - SHELL-01b
---

# Summary: Plan 09-01

## Что сделано

- **`omegazip::looks_like_supported_archive_path`** (`src/compat.rs`) — список суффиксов синхронизирован с `ui/index.html` (`isExtractArchivePath`); юнит-тест.
- **`src-tauri/src/silent_open.rs`**: чтение/запись `gui-prefs.json` (`silent_extract_on_open`, по умолчанию true), `OMEGAZIP_NO_SILENT_EXTRACT`, фоновая распаковка, `AppHandle::exit(0)` при успехе, при ошибке — `show` + `open-files` + `silent-extract-failed`.
- **`lib.rs`**: `RunEvent::Opened` / `Ready` — сначала `try_start_silent_background`, иначе как раньше; команды `get_silent_extract_on_open` / `set_silent_extract_on_open`.
- **`macos_services`**: pasteboard — тот же приоритет silent.
- **`tauri.conf.json`**: `label: "main"` для `get_webview_window`.
- **UI**: чекбокс, загрузка/сохранение настройки, слушатель `silent-extract-failed`.
- **Документация**: `docs/SILENT-EXTRACT.md`, ссылки в `FILE-ASSOCIATIONS.md`, `README.md`.

## Follow-ups

- Прогресс/OS notification при долгой тихой распаковке.
- Single-instance при втором двойном щелчке.

## Verify

- `cargo test`
- `cd src-tauri && cargo check`

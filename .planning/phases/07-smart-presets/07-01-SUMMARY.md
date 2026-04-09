---
phase: 07-smart-presets
plan: 01
status: completed
completed_at: 2026-03-29
requirements_addressed:
  - SMART-01
  - SMART-02
---

# Summary: Plan 07-01

## What shipped

- **`src/smart_preset.rs`** — эвристики `suggested_preset_for_path` / `suggest_compress_preset_hint`; константы лимита обхода папки; юнит-тесты (текст, медиа, PNG, папка только mp4, папка rs+mp4, hint).
- **`src/lib.rs`** — модуль и реэкспорт API.
- **`src/main.rs`** — `--preset auto` в `compress`.
- **`src-tauri/src/lib.rs`** — команда **`suggest_compress_preset`**.
- **`ui/index.html`**, **`ui-android/index.html`** — опция **«Авто (по типу файлов)»**, `refreshCompressAutoHint`, разрешение пресета при «Сжать».
- **`docs/SMART-PRESETS.md`**, ссылки в **`README.md`** и **`docs/FORMATS.md`**.

## Verification

- `cargo test -p omegazip`
- `cargo check` в `src-tauri`
- Ручной прогон GUI: режим Авто + разные источники.

## Follow-ups

- Расширить таблицу расширений по метрикам из продакшена.
- Опционально: MIME через существующие скрипты анализатора — отдельная итерация.

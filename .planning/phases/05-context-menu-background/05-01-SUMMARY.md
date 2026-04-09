---
phase: 05-context-menu-background
plan: 01
status: completed
completed_at: 2026-03-29
requirements_addressed:
  - SHELL-02
  - SHELL-03
  - PATH-02
  - PATH-03
---

# Summary: Plan 05-01

## What shipped

- **`docs/CONTEXT-MENU.md`** — единая инструкция: цель (сжатие .oz/.zip и распаковка без окна), macOS (`install-context-menu.sh`), Windows (шаблон reg + риски), Linux (фрагмент `Actions=`).
- **`scripts/install-context-menu.sh`** — три сервиса Finder: сжатие в **stem.oz**, сжатие в **stem.zip**, распаковка в **stem_распаковано**; `decompress` для поддерживаемых форматов (не только `.oz`).
- **`scripts/context-menu-windows.reg.example`** — пример ключей `OmegaZipCompressOz` / `OmegaZipCompressZip` с плейсхолдером пути к `omegazip.exe` и `cmd /c` + `%~dpn1`.
- **`src-tauri/src/lib.rs`**, **`ui/index.html`**, **`ui-android/index.html`** — `pick_save_file` принимает опционально **`default_directory`**; «Куда сохранить» и экспорт в ZIP открывают диалог в папке источника (PATH-02 в GUI).
- **`README.md`** — ссылка на CONTEXT-MENU; **`docs/FILE-ASSOCIATIONS.md`** — ссылка на контекстное меню.

## Verification

- `bash -n scripts/install-context-menu.sh`
- `cargo check` в `src-tauri`
- Ручная проверка сервисов macOS и импорта reg на Windows — по `VALIDATION.md`.

## Follow-ups / deferred

- **SHELL-03 (трей, фон):** отложено; CLI из ПКМ покрывает минимальный сценарий «без окна».
- **Windows:** для папок и `Directory\Background` — отдельные ключи реестра или установщик (не в этом плане).
- **Linux:** production-ready обёртка stem вместо однострочного `sed` в документе — по желанию, скрипт macOS остаётся эталоном логики.

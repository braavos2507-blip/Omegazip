---
phase: 04-associations-double-click
plan: 01
status: completed
completed_at: 2026-03-30
requirements_addressed:
  - SHELL-01
  - PATH-01
---

# Summary: Plan 04-01

## What shipped

- **`docs/FILE-ASSOCIATIONS.md`** — инструкции по ассоциациям для Windows, macOS, Linux и ссылка на поведение `open-files`.
- **`src-tauri/tauri.conf.json`** — расширены `fileAssociations` для `zip`, `7z`, `rar` (в дополнение к `oz`).
- **`ui/index.html`**, **`ui-android/index.html`** — при одном архиве в `applyPaths` и при выборе архива вручную подставляется **папка назначения = родительский каталог архива** (PATH-01).
- **`README.md`** — ссылка на документацию ассоциаций.
- **`04-01-RESEARCH.md`** — уточнение по Windows `args()`.

## Verification

- `cargo check` в `src-tauri` (выполнить локально после изменений).
- Ручная проверка двойного щелчка — по чеклисту VALIDATION.md на 2 ОС.

## Follow-ups

- Полноценный контекстный меню / фон (фаза 5).
- Single-instance при открытии второго файла (при необходимости).

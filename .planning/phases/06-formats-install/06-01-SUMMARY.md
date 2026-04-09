---
phase: 06-formats-install
plan: 01
status: completed
completed_at: 2026-03-29
requirements_addressed:
  - FMT-01
  - FMT-02
  - FMT-03
  - INST-01
  - INST-02
---

# Summary: Plan 06-01

## What shipped

- **`docs/FORMATS.md`** — матрица чтение/запись (нативно / 7-Zip / —), ориентир «топ‑5», известные ограничения (FMT-02), ссылки.
- **`docs/INSTALL.md`** — happy path для разработчика и пользователя, стратегия **внешнего** 7-Zip без bundling (FMT-03), ссылки на FORMATS / ассоциации / ПКМ.
- **`README.md`** — раздел переименован в «Форматы и 7-Zip», ссылки на FORMATS и INSTALL, сжатые формулировки.
- **`ui/index.html`**, **`ui-android/index.html`** — баннер при отсутствии 7-Zip без длинного `<pre install_howto>`; ссылка на 7-zip.org (десктоп), короткий текст + `docs/INSTALL.md` / `docs/FORMATS.md`; на Android — пояснение про отсутствие внешнего 7-Zip. Классы `deps-banner`, `deps-warn` сохранены; добавлен стиль `.deps-more`.

## Verification

- `grep -E 'FORMATS|INSTALL' README.md`
- `cargo check` в `src-tauri` (без изменений Rust — регрессия не требуется, при желании запустить локально).

## Follow-ups

- При появлении стабильной страницы релизов — подставить URL в INSTALL.md.
- Bundled 7-Zip (если понадобится) — отдельное решение вне v1.1 по умолчанию.

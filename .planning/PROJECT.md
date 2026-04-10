# OmegaZip

## What This Is

**OmegaZip** — архиватор с собственным форматом **`.oz`** (дедуп по чанкам, solid, шифрование, recovery) плюс совместимость с ZIP/tar/zstd/CAB и (при наличии **7-Zip** в системе) с широким набором внешних форматов. Есть **CLI** (`omegazip`), **GUI на Tauri 2** (Windows, macOS, Linux) и **OmegaZip Android** (Tauri Android, отдельный UI; см. `ANDROID_BUILD.md`).

## Core Value

Надёжное сжатие и распаковка с понятным поведением на десктопе и мобильных ограничениях: нативные форматы без внешних бинарников; опционально 7-Zip там, где ОС это позволяет.

## Requirements

### Validated

- Сжатие/распаковка `.oz`, ZIP, tar-семейство, zstd, CAB в ядре на Rust
- GUI с диалогами, прогрессом, статусом 7-Zip
- CI: `cargo test` (корень) + `cargo check` в `src-tauri` на Ubuntu и macOS; `scripts/test-context-menu-logic.sh` + AST PowerShell (`context-menu-powershell` на Windows)
- **v1.0 закрыт** (2026-03-29): см. `.planning/MILESTONE-V1.0.md`
- **OmegaZip Android**: отдельный `ui-android/`, `ANDROID_BUILD.md`, ограничения без 7-Zip/rclone задокументированы
- **GAP-01 (v1.2):** тихая распаковка — `docs/SILENT-EXTRACT.md`
- **GAP-03 (v1.2):** документ «OmegaZip vs топ-5» — `docs/VERSUS-TOP5.md`
- **GAP-04 (v1.2):** сквозной сценарий установки — `docs/INSTALL.md`
- **GAP-05 (v1.2):** трей не входит в релиз — `docs/GAP05-TRAY-DEFERRED.md`
- **GAP-02 (v1.2):** ПКМ Win/Linux — `omega-context-helper.ps1`, установщики, CI + `npm run test:context-menu`

### Active

- [ ] Стабильность редких edge-case; измеримый контур — [docs/MEASURABLE-QUALITY.md](../docs/MEASURABLE-QUALITY.md) (`measure:baseline-local`, `measure:oz-advantage`)

### Out of Scope

- **Создание RAR** — проприетарный формат WinRAR; только распаковка через 7-Zip на десктопе
- **Полный** конвейер нотаризации в CI без секретов не запускается; подпись и сценарии — [docs/DIST-01-MACOS-SIGNING.md](../docs/DIST-01-MACOS-SIGNING.md)
- Полная замена коммерческим архиваторам по всем нишевым форматам без 7-Zip

## Context

- Стек: Rust (ядро), Tauri 2 (shell), vanilla HTML/JS UI (`ui/`, `ui-android/`).
- Карта кодовой базы: `.planning/codebase/*.md`.
- Журнал правок и тестов: `Правки.md` (корень репозитория).

## Constraints

- **Tech stack:** Rust edition 2021, Tauri 2, без тяжёлого фронтенд-фреймворка в основном UI.
- **Android:** нет встроенного 7-Zip в процессе — отдельные сообщения и возможности форматов.
- **Совместимость:** поведение CLI и GUI должно оставаться предсказуемым при отсутствии 7-Zip.

## Current Milestone: v1.2 — ТЗ: строгая доводка (GAP) — **закрыт**

**Предыдущие майлстоуны:** **v1.0** закрыт ([MILESTONE-V1.0.md](MILESTONE-V1.0.md)); **v1.1** — планы 4–7 сданы; **п.8 ТЗ** в объёме спецификации отсутствует (фаза 8 не ведётся).

**Майлстоун v1.2 по GAP:** **закрыт** (GAP-01…05). Дальнейшие пункты — **v2** (см. [REQUIREMENTS.md](REQUIREMENTS.md): QA-03, DIST-01).

**Target features (v1.2 — выполнено):**
- ~~**GAP-01:**~~ ✅ `docs/SILENT-EXTRACT.md`
- ~~**GAP-02:**~~ ✅ ПКМ Win/Linux, CI, `npm run test:context-menu`; смоук по желанию — [docs/QA-WIN-LINUX-PREP.md](../docs/QA-WIN-LINUX-PREP.md)
- ~~**GAP-03:**~~ ✅ `docs/VERSUS-TOP5.md`
- ~~**GAP-04:**~~ ✅ `docs/INSTALL.md`
- ~~**GAP-05** (опц.):~~ ✅ `docs/GAP05-TRAY-DEFERRED.md`

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Формат `.oz` как основной для продвинутых сценариев | Дедуп, solid, шифрование, recovery | ✓ Good |
| 7-Zip как внешний бинарник для не-нативных форматов | Покрытие RAR/7z/ISO и т.д. без лицензирования RAR | ✓ Good |
| Отдельный `ui-android` и `tauri.android.conf.json` | Имя и ограничения Android без ломки десктопа | ✓ Good (v1.0) |
| v1.1: приоритет интеграции ОС над расширением полноэкранного GUI | ТЗ ориентировано на ПКМ и фон | ✓ Good (фазы 4–7) |
| v1.2: честные статусы ТЗ + GAP вместо переоценки «всё [x]» | Заказчик ожидает дословное соответствие | ✓ Good (GAP-01) |

## Evolution

Документ обновляется на границах фаз (`/gsd:transition`) и майлстоунов. После фазы: валидированные требования → Validated; новые — Active; решения → Key Decisions.

---
*Last updated: 2026-04-09 — v1.2 GAP закрыт; п.8 ТЗ не в объёме — TBD-01 снят.*

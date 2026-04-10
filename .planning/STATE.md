# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-04-09)

**Core value:** Надёжное сжатие/распаковка с нативным ядром и опциональным 7-Zip на десктопе; честные ограничения на Android.  
**Майлстоун v1.0:** **закрыт и заархивирован** — `.planning/MILESTONES.md`, `.planning/milestones/v1.0-*.md`, `.planning/MILESTONE-V1.0.md`.  
**Current focus:** Майлстоун **v1.2** по **GAP** — **закрыт**. **v2:** **QA-03** (CI roundtrip) и **DIST-01** (подпись macOS + GHA `macos-signed-build`, см. `docs/DIST-01-MACOS-SIGNING.md`) — частично закрыты; остаётся настройка секретов у владельца репо и при желании нотаризация в CI.

## Current Position

**Milestone:** v1.2 — все GAP-01…05 закрыты; **GAP-02** подтверждён `npm run test:context-menu` + CI (bash + PowerShell AST).  
Last activity: **2026-04-10** — `measure-oz-repo-corpora`, `profile-compress-heavy-smoke` (B2/C2); ранее — ZIP D6/D9, `archive_hardening`.

Progress: [REQUIREMENTS.md](REQUIREMENTS.md) v1.2.

## Performance Metrics

*Заполняется после выполнения планов.*

## Accumulated Context

### Decisions

- См. таблицу Key Decisions в `PROJECT.md`; v1.1 добавляет приоритет интеграции ОС над расширением полноэкранного GUI.

### Pending Todos

- Опционально перед релизом: смоук ПКМ по [docs/QA-WIN-LINUX-PREP.md](../docs/QA-WIN-LINUX-PREP.md).
- По продукту: расширение **QA-03** (пороги/тяжёлый корпус); **DIST-01** — добавить secrets и при необходимости шаг notarize в GHA — см. [REQUIREMENTS.md](REQUIREMENTS.md) v2, [docs/DIST-01-MACOS-SIGNING.md](../docs/DIST-01-MACOS-SIGNING.md).

### Blockers

- Нет (п.8 ТЗ в спецификации отсутствует).

## Session Continuity

*(пусто)*

---
*STATE.md — 2026-04-09 — v1.2 GAP закрыт; TBD-01 снят (п.8 в ТЗ не было).*

# Майлстоун v1.0 — закрыт (ретроспектива GSD)

**Статус:** **completed** (зафиксировано 2026-03-29)  
**Цель майлстоуна:** базовое ядро, десктопный GUI, линия Android, документация и CI.

## Фазы (план → summary)

| Фаза | Папка | Summary |
|------|--------|---------|
| **1** Baseline & quality gates | [phases/01-baseline-quality-gates](phases/01-baseline-quality-gates/) | [01-01-SUMMARY.md](phases/01-baseline-quality-gates/01-01-SUMMARY.md) |
| **2** Android delivery | [phases/02-android-delivery](phases/02-android-delivery/) | [02-01-SUMMARY.md](phases/02-android-delivery/02-01-SUMMARY.md) |
| **3** Hardening & docs | [phases/03-hardening-docs](phases/03-hardening-docs/) | [03-01-SUMMARY.md](phases/03-hardening-docs/03-01-SUMMARY.md) |

## Требования

- Архив чеклиста v1.0: [milestones/v1.0-REQUIREMENTS.md](milestones/v1.0-REQUIREMENTS.md)
- Живой файл: [REQUIREMENTS.md](REQUIREMENTS.md) (v1.1+)

## Дорожная карта (снимок)

- [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md)

## Журнал майлстоунов

- [MILESTONES.md](MILESTONES.md)

## CI

- [`.github/workflows/ci.yml`](../.github/workflows/ci.yml): `cargo test` (корень) + `cargo check` в `src-tauri` на **ubuntu-latest** и **macos-latest**.

---
*Индекс майлстоуна для навигации; детали — в фазах и SUMMARY.*

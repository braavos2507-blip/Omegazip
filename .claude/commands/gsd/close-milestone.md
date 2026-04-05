---
type: prompt
name: gsd:close-milestone
description: Закрыть завершённый майлстоун — архив в .planning/milestones/, MILESTONES.md, свёртка ROADMAP (GSD)
argument-hint: "[version, напр. v1.0]"
allowed-tools:
  - Read
  - Write
  - Bash
---

## Назначение

Алиас практики **complete-milestone** из [complete-milestone.md](../../get-shit-done/workflows/complete-milestone.md): зафиксировать **отгруженный** майлстоун, не ломая активный.

## OmegaZip — правило

- **v1.0** — закрыт; архивы: `.planning/milestones/v1.0-ROADMAP.md`, `v1.0-REQUIREMENTS.md`, журнал `.planning/MILESTONES.md`.
- **v1.1** — не вызывать полное удаление `REQUIREMENTS.md`, пока открыты **TBD-01** / фаза 8. Для закрытия v1.1 сначала `/gsd:audit-milestone`, затем расширить архив и при необходимости вынести «следующий» REQ в новый файл по [complete-milestone.md](../../get-shit-done/workflows/complete-milestone.md).

## Полный чеклист (reference)

См. `@.claude/commands/gsd/complete-milestone.md` и workflow `get-shit-done/workflows/complete-milestone.md`.

# 10-01 Summary — GAP-02 (ПКМ Win/Linux «из коробки»)

Дата: 2026-04-09  
Статус: **done** (реализация + автотесты + CI).

## Поставка

- **Windows:** `scripts/install-context-menu-windows.ps1`, `scripts/omega-context-helper.ps1` — авто .oz/.zip, stem, пресеты; четыре пункта HKCU.
- **Linux:** `scripts/install-context-menu-linux.sh` — Nautilus + KDE, паритет логики с macOS.
- **Доки:** [docs/CONTEXT-MENU.md](../../../docs/CONTEXT-MENU.md), [docs/QA-WIN-LINUX-PREP.md](../../../docs/QA-WIN-LINUX-PREP.md).
- **Автопроверка:** `npm run test:context-menu` → `scripts/test-context-menu-logic.sh`; в [.github/workflows/ci.yml](../../../.github/workflows/ci.yml) — job `context-menu-powershell` (AST PowerShell на `windows-latest`).

## Ручной смоук (опционально перед релизом)

Чеклист: [docs/QA-WIN-LINUX-PREP.md](../../../docs/QA-WIN-LINUX-PREP.md) — Проводник / Nautilus.

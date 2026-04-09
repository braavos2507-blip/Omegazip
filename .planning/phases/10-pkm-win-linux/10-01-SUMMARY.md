# 10-01 Summary — GAP-02 (ПКМ Win/Linux «из коробки»)

Дата: 2026-04-09  
Статус: implementation complete, manual QA pending.

## Что реализовано

- Windows:
  - `scripts/install-context-menu-windows.ps1`
  - установка в `HKCU\Software\Classes\*\shell` без admin;
  - пункты: `.oz`, `.zip`, `extract`;
  - добавлен `-Uninstall`.
- Linux:
  - `scripts/install-context-menu-linux.sh`
  - установка пользовательских действий для:
    - Nautilus Scripts (`~/.local/share/nautilus/scripts`);
    - KDE Service Menu (`~/.local/share/kio/servicemenus/omegazip.desktop`);
  - добавлен `--uninstall`.
- Документация:
  - `docs/CONTEXT-MENU.md` обновлён с прямыми командами установки/удаления Win/Linux.

## Что осталось для закрытия GAP-02

- Ручной прогон сценариев из `Правки.md`:
  - GUI mini-check;
  - фактическая проверка пунктов контекстного меню на целевых окружениях.
- После ручной валидации:
  - отметить GAP-02 как `done` в `REQUIREMENTS.md`/`ROADMAP.md`;
  - обновить `STATE.md` на следующую фазу.

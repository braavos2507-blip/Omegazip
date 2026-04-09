# Phase 5 — VALIDATION

**Requirements:** SHELL-02, SHELL-03, PATH-02, PATH-03

## Критерии (из ROADMAP)

- [x] Сценарий ПКМ: **сжать в .oz** и **сжать в .zip** без полного окна приложения (macOS — Services/Quick Actions; Win/Linux — по документации и шаблонам).
- [x] Результат сжатия в **папке источника**, имя **stem + .oz / .zip** (скрипт macOS; Win — `%~dpn1` в примере reg).
- [x] SHELL-03: в CONTEXT-MENU задокументировано «CLI без окна»; трей — отложено в SUMMARY.

## Проверки

| Проверка | Ожидание |
|----------|----------|
| `bash -n scripts/install-context-menu.sh` | OK |
| `grep CONTEXT-MENU README.md` | есть ссылка |
| macOS вручную | Три пункта сервисов, имена `stem.oz` / `stem.zip`, распаковка в `stem_распаковано` |
| `cargo check` (src-tauri) | OK после правок `pick_save_file` |

## Автоматические команды (локально)

```bash
bash -n scripts/install-context-menu.sh
grep -q CONTEXT-MENU README.md
cd src-tauri && cargo check
```

---
*Phase 5 — обновлено после выполнения 05-01*

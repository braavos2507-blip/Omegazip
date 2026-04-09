# Phase 6 — VALIDATION

**Requirements:** FMT-01 — FMT-03, INST-01, INST-02

## Критерии (из ROADMAP)

- [x] Матрица форматов (`docs/FORMATS.md`)
- [x] Список известных ограничений / пробелов (FMT-02)
- [x] Стратегия 7-Zip задокументирована (`docs/INSTALL.md`, FMT-03)
- [x] Один happy path установки (INST-01)
- [x] Первый запуск: компактный баннер + путь к документации (INST-02)

## Проверки

| Проверка | Ожидание |
|----------|----------|
| `docs/FORMATS.md`, `docs/INSTALL.md` | Существуют |
| `README.md` | Ссылки FORMATS и INSTALL |
| UI без 7-Zip | Короткий жёлтый баннер, без полного `install_howto` в `<pre>` |

## Команды (локально)

```bash
test -f docs/FORMATS.md && test -f docs/INSTALL.md
grep -E 'FORMATS|INSTALL' README.md
```

---
*Phase 6 — после execute 06-01*

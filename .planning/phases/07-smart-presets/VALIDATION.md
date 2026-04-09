# Phase 7 — VALIDATION

**Requirements:** SMART-01, SMART-02

## Критерии (из ROADMAP)

- [x] Таблица эвристик в коде + документ `docs/SMART-PRESETS.md`
- [x] Тесты на 2–3+ класса файлов (`smart_preset` unit tests)
- [x] Lossless: ядро .oz без изменений; авто только fast/balanced

## Проверки

| Проверка | Ожидание |
|----------|----------|
| `cargo test -p omegazip` | OK |
| `docs/SMART-PRESETS.md` | Есть |
| `grep SMART-PRESETS README.md` | Есть ссылка |
| UI | Пункт «Авто (по типу файлов)» |

---
*Phase 7 — после execute 07-01*

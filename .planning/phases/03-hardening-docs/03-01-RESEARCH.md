# Phase 3 (v1.0) — RESEARCH: hardening & docs

**Requirement:** QA-02  
**Ретроспектива:** README, FORMATS, INSTALL, SMART-PRESETS, Правки.md, множество интеграционных и юнит-тестов.

## Состояние

- **Документация:** корневой README, `docs/*`, `Правки.md` отражают актуальные форматы и UX.
- **Тесты:** `compat_roundtrip`, модули `codec`, `chunked`, `smart_preset` и др.; CI гоняет тесты на двух ОС.
- **Edge-case:** path traversal в ZIP покрыт тестом; остальное — по мере нахождения (v2 QA-03).

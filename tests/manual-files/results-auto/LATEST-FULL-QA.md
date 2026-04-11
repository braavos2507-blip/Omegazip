# Полный локальный QA (автогенерация)

**Время:** 2026-04-11T10:43:06+04:00  
**Лог:** [baselines/full-qa-20260411-104251.log](baselines/full-qa-20260411-104251.log)  
**uname:** Darwin 25.3.0 arm64

## Команда

```bash
bash scripts/run-full-local-qa.sh
# или
npm run measure:everything-local
```

## Результаты (кратко)

- **cargo test -p omegazip** — см. лог (все интеграционные + lib).
- **cargo clippy** — см. лог.
- **Profile-smoke** (~30 MiB дедуп):

```
OmegaZip profile-smoke (python timer) 2026-04-11T10:43:05
input_bytes: 30720000

zip_time_s: 0.0369
zip_bytes: 144202
oz_chunked_balanced_time_s: 0.0770
oz_bytes: 53261
oz_vs_zip_size_pct: 63.1% smaller than zip
```

- **BENCH-WORKFLOW-LATEST:** [BENCH-WORKFLOW-LATEST.md](BENCH-WORKFLOW-LATEST.md)
- **samply:** не в PATH

Полный вывод — только в `baselines/full-qa-*.log` (не дублируется здесь из-за размера).


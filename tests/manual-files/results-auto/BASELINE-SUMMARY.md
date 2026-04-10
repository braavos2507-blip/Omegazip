# Сводка измерений (автопрогон)

**Дата:** 2026-04-10  
**Машина:** Darwin arm64 (см. полный лог `baselines/baseline-20260410-002621.log`)  
**Бинарник:** `target/release/omegazip` (Rust 1.89.0)

## A — Регрессия / baseline

| Пункт | Результат |
|-------|-----------|
| **A1** QA-03 (`npm run test:bench-ci`) | OK — roundtrip `.oz` / `.zip` |
| **A2** `baseline-all-local.sh` | OK — лог `baselines/baseline-20260410-002621.log` |
| **A3** `bench.sh` (2.1 MB смешанный suite) | OK — см. таблицу пресетов ниже |
| **A4** `benchmark-workflow.sh --real-only` | OK — отчёт [BENCH-WORKFLOW-LATEST.md](BENCH-WORKFLOW-LATEST.md) |
| **A5** Эта сводка | Зафиксировано |

### Пресеты `.oz` (bench_suite ~2.1 MB)

| Preset | ARCH MB | Ratio | COMP s | DEC s | COMP MB/s | DEC MB/s |
|--------|---------|-------|--------|-------|-----------|----------|
| fast | 2.0 | 0.956 | 0.03 | 0.019 | 69.9 | 110.3 |
| balanced | 2.01 | 0.958 | 0.033 | 0.017 | 63.5 | 123.3 |
| max | 2.0 | 0.957 | 0.023 | 0.017 | 91.1 | 123.3 |
| ultra | 2.0 | 0.957 | 0.216 | 0.037 | 9.7 | 56.7 |

## B2 / C2 — команды «продолжить план»

- Все подкаталоги `downloads`: `npm run measure:oz-repo-corpora` (опционально `CORPUS_EXTRA=/ваш/путь`).
- Тяжёлый дымовой замер: `npm run measure:profile-smoke` → локально `profile-smoke-last.txt` (см. [PROFILE-SMOKE-README.md](PROFILE-SMOKE-README.md)).

## B — Преимущество `.oz`

| Корпус | zip_bytes | oz_bytes | Вывод |
|--------|-----------|----------|--------|
| Синтетика 100× идентичный payload | 64022 | 17245 | **−73.1%** размера относительно ZIP |
| `hellogitworld-master/` (16 файлов, мало дубликатов) | 3933 | 5510 | ZIP меньше (оверлей `.oz` на мелком коде) |

**Вывод:** дедуп `.oz` раскрывается на **массовых повторах**; на маленьких репозиториях без дубликатов ZIP может быть компактнее — это нормально и честно для позиционирования.

## C — Профилирование

| Пункт | Результат |
|-------|-----------|
| **C1** `profile-compress-local.sh` | Команды выведены (samply / Instruments / perf) |
| **C2** Дымовой замер | `compress --preset balanced` bench_suite → **~0.01 s real** (малый вход) |
| **C3** | Следующий шаг: **samply** на каталоге ≥100 MB или `ultra` на большом тексте — см. `docs/MEASURABLE-QUALITY.md` §C |

## D — Надёжность (авто + тесты)

| Пункт | Результат |
|-------|-----------|
| **D1** Распаковка `archive_zip_hellogitworld.zip` | OK — в архиве **1 файл** (`sample.mp4`), извлечён |
| **D2** Обрезанный ZIP | Покрыто `cargo test` — `zip_truncated_file_errors_cleanly` |
| **D3** Path traversal | Покрыто `zip_rejects_path_traversal` |
| **D4** Мусорный «архив» 1 байт | OK — CLI возвращает ошибку |
| **D5–D9** | Не автоматизированы в этом прогоне — вручную по чеклисту |
| **D10** | `7z` не в PATH на этой машине — делегирование внешних форматов опционально |

`cargo test -p omegazip`: **23 теста** (17 lib + 6 compat), все **ok**.

## E — Подпись / нотаризация

Не выполнялось (нет запуска `./build-app.sh` с Developer ID в этой сессии). См. [DIST-01-MACOS-SIGNING.md](../../../docs/DIST-01-MACOS-SIGNING.md).

## F — Периодичность

Зафиксировано как опорный прогон; повторять перед релизом и после изменений в `pipeline.rs` / `compat.rs`.

---

*Повторить локально: `npm run measure:baseline-local`, `bash scripts/checklist-d-automated.sh`, `bash scripts/benchmark-workflow.sh --real-only --out-report tests/manual-files/results-auto/BENCH-WORKFLOW-LATEST.md ./tests/manual-files/downloads`.*

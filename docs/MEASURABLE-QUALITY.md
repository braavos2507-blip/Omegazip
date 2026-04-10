# Измеримое качество и усиление преимуществ `.oz`

Документ — **рабочий чеклист**: идите сверху вниз, отмечайте даты. Цель — не «средний архиватор», а **измеримый** прогресс там, где OmegaZip сильнее (дедуп, solid, recovery, свой контейнер), плюс **жёсткая** гигиена по надёжности.

## A. Регрессия и baseline (обязательный минимум)

| # | Действие | Команда / артефакт | Статус |
|---|----------|-------------------|--------|
| A1 | Быстрый gate (синтетика, roundtrip) | `npm run test:bench-ci` или `bash scripts/qa03-benchmark-ci.sh` | ☐ |
| A2 | Полный локальный срез | `bash scripts/baseline-all-local.sh` → лог в `tests/manual-files/results-auto/baselines/` | ☐ |
| A3 | Пресеты на смешанном корпусе | `bash scripts/bench.sh` (уже входит в A2) | ☐ |
| A4 | **Ваши** реальные данные | `bash scripts/benchmark-workflow.sh --real-only --out-report docs/BENCH_RESULTS.md "ПУТЬ/к/корпусу"` (macOS: бинарник из `.app` по умолчанию) | ☐ |
| A5 | Зафиксировать цифры в сводке | Обновить таблицу в [BENCH_RESULTS.md](BENCH_RESULTS.md) или `tests/manual-files/results-auto/BASELINE-SUMMARY.md` (создайте при первом прогоне) | ☐ |

**Правило:** перед крупным изменением в ядре сжатия — A2 + A4 (или хотя бы A1 + `measure-oz-advantage`).

## B. Преимущество `.oz` (не гоняемся за «ещё один ZIP»)

| # | Действие | Команда | Что смотреть |
|---|----------|---------|----------------|
| B1 | Дедуп-корпус: ZIP vs chunked `.oz` | `bash scripts/measure-oz-advantage.sh` | `oz_bytes << zip_bytes`, выигрыш по месту; время — ориентир |
| B2 | То же на **ваших** дубликатах (бэкапы, node_modules-копии, фото-копии) | `npm run measure:oz-repo-corpora` — все подкаталоги `tests/manual-files/downloads/*/`; плюс свой путь: `CORPUS_EXTRA=/путь bash scripts/measure-oz-repo-corpora.sh`. Или точечно: `CORPUS_DIR=... bash scripts/measure-oz-advantage.sh` | Рост разрыва в пользу `.oz` при реальных повторах |
| B3 | Зафиксировать в позиционировании | [VERSUS-TOP5.md](VERSUS-TOP5.md) — при необходимости добавить строку «измерено: …» со ссылкой на лог | Честные цифры вместо маркетинга |

## C. Профилирование (где теряются секунды)

| # | Действие | Команда / заметка | Статус |
|---|----------|-------------------|--------|
| C1 | Подсказки под вашу ОС | `bash scripts/profile-compress-local.sh` | ☐ |
| C2 | Один прогон с замером | **Дымовой:** `npm run measure:profile-smoke` → `tests/manual-files/results-auto/profile-smoke-last.txt` (~30 MiB дедуп). Затем при необходимости инструмент из C1 на том же или большем корпусе | ☐ |
| C3 | Записать вывод | 1–3 предложения: узкое место (I/O, хэш, ZSTD, индекс) → в `Правки.md` или issue у себя | ☐ |

## D. Надёжность: «злой» зоопарк (ручной + автотесты)

Автотесты: `cargo test -p omegazip`, в т.ч. `compat_roundtrip` (ZIP/tar/zstd, path traversal, обрезанный ZIP) и **`archive_hardening`** (шифрование `.oz`, обрезка `.oz`, symlink).

Ручной чеклист (добавляйте свои файлы в `tests/manual-files/downloads/`):

| # | Класс входа | Ожидание | Статус |
|---|-------------|----------|--------|
| D1 | Нормальный ZIP из A4 | Распаковка OK | ☐ |
| D2 | Обрезанный / битый ZIP | Ошибка, **без паники** (покрыто тестом `zip_truncated_file_errors_cleanly`) | ☐ |
| D3 | ZIP с `..` в путях | Отказ (тест `zip_rejects_path_traversal`) | ☐ |
| D4 | Пустой / один байт «не архив» | Понятная ошибка | ☐ |
| D5 | `.oz` с неверным паролем | Понятная ошибка | Авто: `cargo test -p omegazip --test archive_hardening`, `npm run measure:checklist-d` (CLI) |
| D6 | Очень длинные пути внутри архива | Глубокая вложенность без `..` распаковывается | Авто: `compat_roundtrip::zip_deep_nested_path_extracts` (32 уровня) |
| D7 | Неполный `.oz` (обрезать хвост вручную) | Ошибка, не зависание | Авто: `archive_hardening::truncated_oz_errors_cleanly` |
| D8 | Симлинки при сжатии папки | По умолчанию **не** включаются в архив (не следуем по ссылке) | Авто (unix): `archive_hardening::symlink_in_source_dir_is_skipped_not_followed` |
| D9 | Два разных файла с одинаковым именем в ZIP | **Последняя запись перезаписывает** предыдущую (как при `File::create`) | Авто: `compat_roundtrip::zip_duplicate_entry_paths_last_content_wins` |
| D10 | Архив только через 7-Zip (RAR/7z) | С 7-Zip в PATH — OK; без — честное сообщение | ☐ |

## E. Локальный релиз (доверие при установке)

| # | Действие | Документ |
|---|----------|----------|
| E1 | Подпись после `./build-app.sh` | [DIST-01-MACOS-SIGNING.md](DIST-01-MACOS-SIGNING.md) |
| E2 | Нотаризация при публичной раздаче | `scripts/macos-notarize-app.sh` + тот же документ |

## F. Периодичность (рекомендация)

- **Перед релизом:** A1 + A2 + B1 + D1–D5 + E1.
- **Раз в спринт / 2 недели:** B2 на реальном корпусе + один пункт C.
- **После изменения `src/pipeline.rs` / `compat.rs`:** A1 + `cargo test` + D2/D3 автоматически в CI.

## KPI-гейт для ниши

Целевые KPI (локальный минимум):

- `oz_vs_zip_size_pct >= 10%` на versioned corpus.
- `time_regression_pct <= 150%` (по smoke: `.oz` относительно ZIP).
- `D-checklist` без критических ошибок.

Проверка:

```bash
npm run measure:kpi-check
```

Строгий релизный гейт (падает при FLAG/BLOCK по E1/E2/Clean-machine + KPI fail):

```bash
CORPUS_EXTRA=/absolute/path/to/real-corpus npm run measure:release-gate-strict
```

Локальный режим разработки (без Apple Developer сертификата):

```bash
CORPUS_EXTRA=/absolute/path/to/real-corpus npm run measure:release-readiness
CORPUS_EXTRA=/absolute/path/to/real-corpus npm run measure:release-gate-local
```

---

## Последний полный автопрогон

- **2026-04-10** — полный локальный контур: [LATEST-FULL-QA.md](../tests/manual-files/results-auto/LATEST-FULL-QA.md), сырой лог: `tests/manual-files/results-auto/baselines/full-qa-20260410-143414.log`.
- Скрипт части D: `bash scripts/checklist-d-automated.sh` (добавлен D10b smoke `.7z` при наличии `7z/7zz` в PATH).
- Release preflight: `npm run measure:release-readiness` → [RELEASE-READINESS.md](RELEASE-READINESS.md).
- Competitive bench: `npm run measure:competitive` → [OZ-ZIP-7Z-LATEST.md](../tests/manual-files/results-auto/OZ-ZIP-7Z-LATEST.md).
- Для .oz в competitive-бенче используется `--preset competitive` (dry-run эвристика), в отчёте есть метрики compress/decompress.

*Связано: [QA-03-BENCHMARKS.md](QA-03-BENCHMARKS.md), [VERSUS-TOP5.md](VERSUS-TOP5.md).*

# Changelog

## 2026-04-11

### security & quality: dependency audit, threat notes, property tests, fuzz seed
- [SECURITY.md](SECURITY.md) — краткий threat model, fuzz/SBOM, как сообщать об уязвимостях.
- GitHub Actions **Security audit** (еженедельно + push в `main`/`master`/`fix/**`): `cargo audit` в корне и `src-tauri`, `npm audit --audit-level=high`.
- `scripts/security-audit.sh`, `npm run audit:deps` — тот же контур локально (нужен `cargo install cargo-audit`).
- Интеграционные property-тесты [tests/property_codecs.rs](../tests/property_codecs.rs) (proptest): roundtrip Store / Balanced / Fast / MaxRatio + smoke `analyze_bytes`.
- Каталог [fuzz/](../fuzz/) + цель `huffman_decode` для `cargo-fuzz` (декодер Huffman + анализатор на случайных байтах).
- Huffman `decode`: счётчик бит `u32` и маска для `len >= 32` — без переполнения `u8` и без `1u32 << len` при `len >= 32`.
- CI **test** workflow: push также на ветки `fix/**`; `bash -n` для `security-audit.sh`.

## 2026-04-10

### feat(bench): competitive ZIP vs .oz vs 7z + KPI/strict gates
- `scripts/measure-competitive-bench.sh` (`npm run measure:competitive`) — автоматический отчёт `tests/manual-files/results-auto/OZ-ZIP-7Z-LATEST.md` по corpus `versioned` и `mixed`.
- `scripts/kpi-check-local.sh` (`npm run measure:kpi-check`) — KPI-гейт: `oz_vs_zip_size_pct >= 10%`, ограничение регресса времени, D-checklist без критических ошибок.
- `scripts/release-gate-strict.sh` (`npm run measure:release-gate-strict`) — строгий релизный gate: FAIL при FLAG/BLOCK в E1/E2/Clean-machine или KPI fail.
- Добавлен `--preset competitive` (dry-run эвристика для .oz) и метрики декомпрессии в competitive-таблицу.
- Исправлена ошибка дедупликации codec-id (могла давать ошибку распаковки `Unknown frame descriptor` на части mixed-наборов).
- Оптимизирована распаковка solid-архивов: stream декодируется один раз, а не на каждый файл.
- На текущем прогоне: `versioned` — `.oz` +9.6% к ZIP; `mixed` — `.oz` +4.9% к ZIP; по скорости компрессии .oz быстрее ZIP/7z.
- Readiness/gates теперь поддерживают режимы:
  - `RELEASE_MODE=local` (E1/E2/clean-smoke = N/A),
  - `RELEASE_MODE=public` (E1/E2/clean-smoke обязательны).

### feat(measure): real internet corpus for B2 (jQuery/Bootstrap versions)
- Добавлен реальный корпус в `Архивы/github-versions` (скачанные публичные ZIP релизы разных версий jQuery и Bootstrap, затем распаковка).
- Прогон `CORPUS_EXTRA=/Users/renat/01Project/OmegaZip/Архивы npm run measure:oz-repo-corpora`: на ~3083 файлах chunked `.oz` меньше ZIP на ~18.1% (`61238454 -> 50177021`).
- Обновлён блок измерений в `docs/VERSUS-TOP5.md`.

### chore(release): local readiness gate + D10b smoke
- Новый preflight `scripts/release-readiness-local.sh` + `npm run measure:release-readiness` (генерирует `docs/RELEASE-READINESS.md` с GO/NO-GO, QA freshness, 7z, signing/notary checks).
- `scripts/checklist-d-automated.sh`: D10b smoke `.7z` compress+decompress автоматически, когда `7z/7zz` доступен.
- Обновлены `docs/DIST-01-MACOS-SIGNING.md` и `docs/MEASURABLE-QUALITY.md` ссылками на release preflight и свежий full-QA лог.

### chore(measure): B2 multi-corpus + C2 heavy smoke
- `scripts/measure-oz-repo-corpora.sh` — прогон `measure-oz-advantage` по `tests/manual-files/downloads/*/` и опционально `CORPUS_EXTRA`.
- `scripts/profile-compress-heavy-smoke.sh` — ~30 MiB дедуп-корпус, время ZIP vs chunked `.oz` → `profile-smoke-last.txt` (в .gitignore); `npm run measure:oz-repo-corpora`, `measure:profile-smoke`.
- [PROFILE-SMOKE-README.md](tests/manual-files/results-auto/PROFILE-SMOKE-README.md), обновлён [MEASURABLE-QUALITY.md](docs/MEASURABLE-QUALITY.md).

### test: ZIP D6/D9 — глубокая вложенность и дубликаты имён
- `compat_roundtrip`: `zip_deep_nested_path_extracts` (32 уровня каталогов), `zip_duplicate_entry_paths_last_content_wins` (политика «последний выигрывает»).
- [FORMATS.md](FORMATS.md): задокументировано поведение при дубликатах путей в ZIP.

### test: archive_hardening — .oz пароль, обрезка, symlink (unix)
- `tests/archive_hardening.rs`: зашифрованный `.oz` без пароля / с неверным паролем, усечённый `.oz`, симлинк в каталоге не попадает в архив.
- `scripts/checklist-d-automated.sh`: D4b (пустой файл), D5 (CLI неверный пароль); `npm run test:archive-hardening`.
- [MEASURABLE-QUALITY.md](MEASURABLE-QUALITY.md): обновлены строки D5/D7/D8.

## 2026-04-09

### chore(measure): автопрогон чеклиста — BASELINE-SUMMARY, checklist-d, OMEGZIP_BIN в benchmark-workflow

### fix(scripts): bench.sh создаёт каталог перед tee raw.tsv

### docs+tooling: измеримое качество и усиление `.oz`
- [docs/MEASURABLE-QUALITY.md](MEASURABLE-QUALITY.md) — чеклист baseline / профилирование / зоопарк / локальный релиз.
- `scripts/measure-oz-advantage.sh` (ZIP vs chunked `.oz` на дедуп-корпусе), `scripts/baseline-all-local.sh`, `scripts/profile-compress-local.sh`; npm: `measure:oz-advantage`, `measure:baseline-local`.
- Тест `zip_truncated_file_errors_cleanly`; логи baseline в `results-auto/baselines/*.log` (в .gitignore).

### feat(macos): DIST-01 — подпись, entitlements, GHA
- `src-tauri/entitlements/macos-release.plist` и ссылка в `tauri.conf.json`; после вставки CLI `build-app.sh` выполняет повторную подпись (`APPLE_SIGNING_IDENTITY` / `MACOS_CODESIGN_IDENTITY`).
- `scripts/macos-import-certificate-ci.sh`, `scripts/macos-notarize-app.sh`; [docs/DIST-01-MACOS-SIGNING.md](DIST-01-MACOS-SIGNING.md); workflow **macos-signed-build** (`workflow_dispatch`, артефакт zip).

### test(ci): QA-03 compress/decompress roundtrip
- `scripts/qa03-benchmark-ci.sh` — синтетический корпус, проверка байт после `.oz` (`--preset auto`) и `.zip`; запуск на ubuntu + macOS в CI.
- `npm run test:bench-ci`; документация: [docs/QA-03-BENCHMARKS.md](QA-03-BENCHMARKS.md).

### chore(planning): TBD-01 снят — п.8 в ТЗ отсутствует
- Требование **TBD-01** и gate «п.8 ТЗ» удалены из активного объёма: в принятой спецификации **пункта 8 нет**; обновлены REQUIREMENTS, ROADMAP, PROJECT, STATE.

### chore(release): v1.2 GAP milestone closed
- GAP-01…05 закрыты (см. `.planning/MILESTONE-V1.2.md`, `.planning/MILESTONES.md`).
- CI: `cargo clippy` для crate `omegazip` (ubuntu + macOS); контекстное меню — bash-тест логики + `bash -n`, Windows — AST PowerShell.

### feat(win+linux): context menu parity with macOS helpers
- `scripts/omega-context-helper.ps1`, `install-context-menu-windows.ps1`; `install-context-menu-linux.sh` — согласованные `pick_ext_auto`, пресеты, крупные папки.
- Документация: `docs/CONTEXT-MENU.md`, `docs/QA-WIN-LINUX-PREP.md`; legacy `context-menu-windows.reg.example` помечен как устаревший.

### docs
- GAP-03: `docs/VERSUS-TOP5.md`; GAP-04: сквозной сценарий в `docs/INSTALL.md`; GAP-05: `docs/GAP05-TRAY-DEFERRED.md`.

## 2026-04-08

### feat(macos): configurable aggressive `.oz` preset (Finder Services)
- `~/.config/omegazip/context_preset` or `OMEGAZIP_CONTEXT_PRESET`: `auto` | `max` | `ultra` (`aggressive` = `max`).
- Default **`OMEGAZIP_AUTO_UPGRADE_FOLDER_MB=200`** when unset; optional file `~/.config/omegazip/auto_upgrade_folder_mb`; `0` disables.
- `install-context-menu.sh` creates `context_preset` with `auto` if missing.
- Examples: [config/omegazip/context_preset.example](config/omegazip/context_preset.example), [config/omegazip/auto_upgrade_folder_mb.example](config/omegazip/auto_upgrade_folder_mb.example).

### feat(cli+macos): `--preset auto` for `.oz` from Services + smarter PDF/EPUB presets
- Finder workflow: `omegazip compress --preset auto` when output is `.oz` (balanced → chunked dedup; fast для медиа/архивов).
- `smart_preset`: PDF/EPUB → **balanced** (раньше PDF/EPUB давали fast без чанков); добавлены `fb2`, `mobi`, `azw`, `azw3` как текстовые.
- `benchmark-workflow.sh`: для `.oz` тот же `--preset auto`.

### fix(macos): default `.oz` in `pick_ext_auto` (blacklist → `.zip`)
- Replaced extension whitelist with **default `.oz`** and a **blacklist** for media, existing archives, fonts, executables, disk images, sqlite DB files.
- PDF, EPUB, DOCX and other text-heavy types now use **`.oz`** from Finder Services.

### docs(macos): release checklist and benchmark CSV
- Restored root `README.md`; added `.gitignore` for `node_modules`, `target`, `dist`, `out`.
- `scripts/benchmark-workflow.sh`: column `extracted_bytes` for decompress rows (sum of extracted file sizes); report table distinguishes compress vs decompress.
- `docs/MACOS-QUICKSTART.md` — установка, сервисы, `/tmp/OmegaZip-workflow.log`, переустановка.
- `docs/CONTEXT-MENU.md`: секция «Диагностика».
- `scripts/install-context-menu.sh`: `base64 -i` для совместимости с macOS.

### fix(macos): stabilize Finder context actions via workflows (`9686603`)
- Switched macOS context actions to Automator workflow services that call `omegazip` CLI directly.
- Added resilient Finder selection fallback in workflows when direct input is not passed.
- Updated `docs/CONTEXT-MENU.md` with the 2-item workflow setup and duplicate-menu cleanup guidance.

### chore(macos): remove temporary service diagnostics (`94457e0`)
- Removed temporary debug tracing and `/tmp` diagnostics from the macOS service/open-file flow in `src-tauri`.
- Kept the final silent handling behavior while cleaning up instrumentation added during investigation.

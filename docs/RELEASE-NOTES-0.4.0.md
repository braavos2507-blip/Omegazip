# OmegaZip 0.4.0 — Release Notes

Дата: 2026-04-15

## Что нового

- 4-ступенчатый пайплайн архивации с пресетами `fast|balanced|max|ultra`.
- `.oz` v2: `chunked dedup`, `solid`, парольное шифрование, recovery (Reed-Solomon + CRC).
- Репозиторий бэкапов: `repo init|backup|list|restore|prune|push|rclone-sync`.
- Экспорт `.oz -> .zip`, просмотр `info/list`, прогресс в GUI.
- Контекстное меню и сценарии «без окна» для быстрых операций.

## Качество и готовность

- Локальный контур релизной готовности: `GO-LOCAL` (см. `docs/RELEASE-READINESS.md`).
- KPI: PASS (см. `tests/manual-files/results-auto/KPI-CHECK-LATEST.md`).
- Полный локальный QA и benchmark-отчеты обновлены:
  - `tests/manual-files/results-auto/LATEST-FULL-QA.md`
  - `tests/manual-files/results-auto/BENCH-WORKFLOW-LATEST.md`

## Артефакты

- `dist/OmegaZip.app`
- `dist/OmegaZip_0.4.0_aarch64.dmg`
- `dist/OmegaZip_0.4.0_app_macOS-arm64.zip`
- `dist/omegazip-cli-macos-arm64`
- checksums: `dist/CHECKSUMS.txt`

## Известные ограничения

- Публичная macOS-дистрибуция требует `Developer ID Application` + notarization.
- Подпись `Apple Development` подходит для локальной разработки/тестов, но не заменяет публичный Developer ID релиз.
- Для расширенных форматов (`.7z`, RAR, ISO, WIM, MSI) нужен установленный `7z/7zz`.

## Что дальше

- Для публичного выпуска пройти `docs/RELEASE-DAY-CHECKLIST.md`:
  - E1: подпись `Developer ID Application`;
  - E2: notarization + staple;
  - clean-machine smoke;
  - public readiness + strict gate.

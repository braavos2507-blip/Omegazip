# Clean-Machine Smoke (release)

Цель: проверить пользовательский путь на машине, где раньше не ставили OmegaZip.

## Подготовка

- Чистый macOS-профиль/VM.
- Нет старых Services/Quick Actions OmegaZip.
- Есть релизный артефакт (`.app` или DMG).

## Чеклист

| Шаг | Действие | Ожидание | Статус |
|---|---|---|---|
| S1 | Установить приложение в `Applications` | Установка без ошибок | ☐ |
| S2 | Первый запуск `.app` | Приложение открывается, GUI отвечает | ☐ |
| S3 | Проверка CLI внутри bundle | `OmegaZip.app/Contents/MacOS/omegazip --help` работает | ☐ |
| S4 | Установить контекстные действия | Скрипт установки проходит без ошибок | ☐ |
| S5 | Сжатие через контекстное меню | Создаётся архив рядом с исходником | ☐ |
| S6 | Распаковка двойным кликом | Архив открывается/извлекается корректно | ☐ |
| S7 | Ассоциации файлов | `.oz` открывается OmegaZip по умолчанию | ☐ |
| S8 | Форматы через 7-Zip (если установлен) | `.7z`/RAR сценарии работают | ☐ |
| S9 | Удаление/переустановка Services | Нет дублей, действия не ломаются | ☐ |

## Лог smoke

- Дата:
- ОС:
- Артефакт:
- Результат: GO / NO-GO
- Замечания:

---

## Фиксация для публичного release gate

Режим `RELEASE_MODE=public` в `scripts/release-readiness-local.sh` требует **PASS** по clean-machine (строка в записи).

1. Скопируйте шаблон и отредактируйте:

   ```bash
   cp tests/manual-files/results-auto/CLEAN-MACHINE-SMOKE-LATEST.md.example \
      tests/manual-files/results-auto/CLEAN-MACHINE-SMOKE-LATEST.md
   ```

   В файле должна быть строка вида `**Status:** PASS` (как в примере).

2. Или автоматически:

   ```bash
   npm run measure:record-clean-machine-smoke -- PASS "OmegaZip_0.4.0_x64.dmg"
   ```

3. Затем: `npm run measure:release-readiness:public` и `npm run measure:release-gate-strict` (после E1/E2 и KPI).


# macOS: быстрый старт (приложение, сервисы, логи)

## 1. Установка

1. Соберите или скопируйте **`OmegaZip.app`** в `/Applications/` (или используйте свой путь; CLI лежит внутри: `Contents/MacOS/omegazip`).
2. Убедитесь, что бинарник исполняемый и запускается из терминала:
   ```bash
   /Applications/OmegaZip.app/Contents/MacOS/omegazip --help
   ```

## 2. Пункты «Сжать / Распаковать» в Finder

Из **корня репозитория**:

```bash
./scripts/install-context-menu.sh
```

Скрипт ставит в `~/Library/Services` два workflow (подробности и ограничения — [CONTEXT-MENU.md](CONTEXT-MENU.md)).

В Finder: **ПКМ → Сервисы** или **Быстрые действия** — выберите нужный пункт. При необходимости включите сервисы в **Системные настройки → Клавиатура → Сочетания клавиш → Сервисы**.

## 3. Лог workflow

Если что-то не срабатывает, смотрите лог (пишет `install-context-menu.sh`):

**`/tmp/OmegaZip-workflow.log`**

Там видны выбранное расширение, пути и вывод `omegazip compress` / `decompress`.

## 4. Чистая переустановка меню

1. Удалите старые `.workflow` OmegaZip из `~/Library/Services/` (или переустановите только скриптом — он перезаписывает ожидаемые имена).
2. Повторите `./scripts/install-context-menu.sh`.
3. При «залипших» пунктах: **`killall Finder`**, при необходимости перезагрузка служб (`pbs -flush` — по ситуации).

## 5. Тихий режим из приложения

Поведение без окна при открытии файлов из системы описано в [SILENT-EXTRACT.md](SILENT-EXTRACT.md) (если файл есть в вашей копии репо).

## 6. Бенчмарк CLI

Скрипт **`scripts/benchmark-workflow.sh`** измеряет сжатие/распаковку через установленный `omegazip`. Результаты по умолчанию в `/tmp/omegazip-full-bench/`; пример отчёта в репозитории: [BENCH_RESULTS.md](BENCH_RESULTS.md).

```bash
./scripts/benchmark-workflow.sh --real-only --out-report docs/BENCH_RESULTS.md "$HOME/Documents/Для тестов"
```

## 7. Локальная обкатка (одна машина, без публикации)

1. Собрать и положить приложение: `./build-app.sh` → при необходимости скопировать `dist/OmegaZip.app` в `/Applications/`.
2. Поставить сервисы: `./scripts/install-context-menu.sh`.
3. Проверить окружение: `./scripts/verify-local-macos.sh` (или передать путь к своему `.app`).
4. В Finder прогнать: текстовый файл → сжать; картинку → сжать; архив → распаковать; смотреть `/tmp/OmegaZip-workflow.log` при сбоях.
5. Замечания фиксировать у себя (например в `Правки.md`), **в удалённый репозиторий не выкладывать**, пока не решишь сам.

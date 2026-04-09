# Контекстное меню и сжатие без окна

Цель: из проводника **сжать в `.oz` или `.zip`** и **распаковать** архивы через CLI OmegaZip **без открытия главного окна** Tauri (SHELL-03: сценарий «CLI-first»). Результат сжатия лежит **в папке источника**, имя файла — **stem + расширение** (PATH-02, PATH-03), как в GUI.

Связанный документ: [FILE-ASSOCIATIONS.md](FILE-ASSOCIATIONS.md) (двойной щелчок по архиву).

## macOS

1. Соберите приложение и CLI (`omegazip` внутри `.app`).
2. Из корня репозитория выполните:
   ```bash
   ./scripts/install-context-menu.sh
   ```
3. Скрипт ставит в `~/Library/Services` два workflow:
   - **Сжать в OmegaZip** → `stem.oz` или `stem.zip` рядом с файлом/папкой (формат выбирается автоматически);
   - **Распаковать в OmegaZip** → папка `stem_распаковано` рядом с архивом (форматы, которые поддерживает `omegazip decompress`).
   - **Правило `pick_ext_auto`:** по умолчанию **`.oz`** (текст, разметка, PDF, EPUB, типичные документы, исходники, неизвестные суффиксы — сильная сторона OmegaZip). **`.zip`** только для явно «не-текстового» набора: готовые архивы (`zip`, `7z`, `rar`, `gz` …), изображения, видео, аудио, шрифты, бинарники/образы дисков и т.д. (см. `pick_ext_auto` в `scripts/install-context-menu.sh`).
   - **Пресет для `.oz` без диалогов:** по умолчанию `compress --preset auto` ([SMART-PRESETS.md](SMART-PRESETS.md)). Усиление без GUI:
     - Файл **`~/.config/omegazip/context_preset`** — одна строка: `auto` | `max` | `ultra` (или `aggressive` = `max`). Пример: [config/omegazip/context_preset.example](../config/omegazip/context_preset.example). Скрипт установки **создаёт** этот файл с `auto`, если его ещё нет.
     - Или переменная **`OMEGAZIP_CONTEXT_PRESET`** (имеет приоритет над файлом; в Automator/Finder обычно не наследуется — удобнее файл).
     - **Большие папки → `max`:** по умолчанию порог **200 MB** (`du -sk`): при `auto` и каталоге не меньше этого размера вызывается **`--preset max`**. Переопределить: переменная **`OMEGAZIP_AUTO_UPGRADE_FOLDER_MB`** или файл **`~/.config/omegazip/auto_upgrade_folder_mb`** (одна строка, число МБ). **`0`** — отключить автоповышение.
     - Для **`.zip`** по-прежнему обычный `compress` (Deflate).
4. После установки в Finder: **ПКМ → Сервисы** (или **Быстрые действия**) — выберите нужный пункт. При необходимости включите пункты в **Системные настройки → Клавиатура → Сочетания клавиш → Сервисы**.
5. Если пункты дублируются (`…workflow` и `(OmegaZip)`), переустановите приложение без инжекта `NSServices` и снова запустите `./scripts/install-context-menu.sh`.

### Диагностика (macOS)

- **Лог workflow:** `/tmp/OmegaZip-workflow.log` — сюда пишутся шаги `compress-auto` / `extract`, выбранное расширение и stderr `omegazip`. Если в Finder «ничего не происходит», откройте этот файл сразу после попытки.
- **Перезапуск Finder:** `killall Finder` — после смены сервисов или если меню «застыло».
- Краткий чеклист: [MACOS-QUICKSTART.md](MACOS-QUICKSTART.md).

## Windows

Для типового пользователя есть готовый скрипт в профиль **текущего пользователя** (`HKCU`, без admin):

1. Подготовьте путь к `omegazip.exe`.
2. Выполните:

   ```powershell
   powershell -ExecutionPolicy Bypass -File .\scripts\install-context-menu-windows.ps1 -OmegaZipExe "C:\Path\To\omegazip.exe"
   ```

3. Скрипт добавит пункты:
   - `Сжать в .oz (OmegaZip)`
   - `Сжать в .zip (OmegaZip)`
   - `Распаковать (OmegaZip)` -> `%~dpn1_распаковано`

4. Удаление:

   ```powershell
   powershell -ExecutionPolicy Bypass -File .\scripts\install-context-menu-windows.ps1 -Uninstall
   ```

Если нужен полностью ручной режим/шаблон `.reg`, остаётся `scripts/context-menu-windows.reg.example`.

Пример вызова вручную из `cmd` (аналог одного из пунктов меню):

```bat
"C:\Path\To\omegazip.exe" compress "%USERPROFILE%\Documents\file.txt" "%USERPROFILE%\Documents\file.oz"
```

## Linux

Для типового пользователя есть установщик в профиль:

```bash
./scripts/install-context-menu-linux.sh --binary /absolute/path/to/omegazip
```

Что ставится:

- **Nautilus Scripts** (`~/.local/share/nautilus/scripts/`):
  - `OmegaZip Compress (Auto)` -> авто `.oz/.zip` (для `.oz` использует `--preset auto`);
  - `OmegaZip Extract Here` -> `${stem}_распаковано`.
- **KDE Service Menu** (`~/.local/share/kio/servicemenus/omegazip.desktop`) с теми же действиями.

Удаление:

```bash
./scripts/install-context-menu-linux.sh --uninstall
```

Если в вашей среде используется другой механизм пунктов меню, можно использовать иллюстративный `.desktop`-подход:

```ini
Actions=CompressOz;CompressZip;Extract;

[Desktop Action CompressOz]
Name=Сжать в .oz (OmegaZip)
Exec=sh -c '/usr/local/bin/omegazip compress "$1" "$(dirname "$1")/$(basename "$1" | sed "s/\\.[^.]*$//").oz"' sh %f

[Desktop Action CompressZip]
Name=Сжать в .zip (OmegaZip)
Exec=sh -c '/usr/local/bin/omegazip compress "$1" "$(dirname "$1")/$(basename "$1" | sed "s/\\.[^.]*$//").zip"' sh %f

[Desktop Action Extract]
Name=Распаковать (OmegaZip)
Exec=sh -c '/usr/local/bin/omegazip decompress "$1" "$(dirname "$1")/$(basename "$1" | sed "s/\\.[^.]*$//")_распаковано"' sh %f
```

Для папок и имён вида `archive.tar.gz` **sed с одним отрезанием суффикса** даёт иной stem, чем OmegaZip GUI; штатный `install-context-menu-linux.sh` уже включает корректировку stem для популярных двойных суффиксов.

## GUI: диалог «Куда сохранить»

В десктоп-приложении при выборе файла сохранения передаётся **опционально каталог по умолчанию** (родитель пути источника или архива при экспорте в ZIP), через команду `pick_save_file` и поля `default_directory` / `defaultDirectory` (macOS: rfd; Windows/Linux: `tauri-plugin-dialog` с `set_directory`).

## Отложено

- **Трей / фоновый режим** (расширенный SHELL-03) — не входит в этот план; при необходимости отдельная фаза.

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
   - **Пресет без вопросов пользователю:** для выхода **`.oz`** workflow вызывает `omegazip compress --preset auto` — подбор **fast/balanced** и **чанков** по эвристикам [SMART-PRESETS.md](SMART-PRESETS.md); для **`.zip`** остаётся обычный `compress` (пресеты к ZIP не применяются).
4. После установки в Finder: **ПКМ → Сервисы** (или **Быстрые действия**) — выберите нужный пункт. При необходимости включите пункты в **Системные настройки → Клавиатура → Сочетания клавиш → Сервисы**.
5. Если пункты дублируются (`…workflow` и `(OmegaZip)`), переустановите приложение без инжекта `NSServices` и снова запустите `./scripts/install-context-menu.sh`.

### Диагностика (macOS)

- **Лог workflow:** `/tmp/OmegaZip-workflow.log` — сюда пишутся шаги `compress-auto` / `extract`, выбранное расширение и stderr `omegazip`. Если в Finder «ничего не происходит», откройте этот файл сразу после попытки.
- **Перезапуск Finder:** `killall Finder` — после смены сервисов или если меню «застыло».
- Краткий чеклист: [MACOS-QUICKSTART.md](MACOS-QUICKSTART.md).

## Windows

Полноценное каскадное меню обычно делают установщиком (MSI) или расширением оболочки. Для ручной настройки:

1. Убедитесь, что **`omegazip.exe`** доступен по стабильному пути (или добавьте каталог с бинарником в `PATH`).
2. Отредактируйте шаблон **`scripts/context-menu-windows.reg.example`**: подставьте реальный путь к `omegazip.exe` вместо `C:\\Path\\To\\omegazip.exe`.
3. Импортируйте `.reg` через реестр (нужны права администратора на запись в `HKEY_CLASSES_ROOT`). Если подписи на кириллице отображаются неверно, сохраните файл в **UTF-16 LE** («Юникод») в Блокноте перед импортом.
4. В значении `command` используется **`cmd /c`** и подстановки вида `"%~dpn1"` — так выходной файл получает то же имя (без расширения) и расширение `.oz` / `.zip` в **той же папке**, что и `%1`.

**Риски:** пути с пробелами и кавычками требуют аккуратного экранирования; политики UAC и антивирус могут блокировать правки реестра; обновление приложения меняет путь к `.exe` — reg нужно обновить.

Пример вызова вручную из `cmd` (аналог одного из пунктов меню):

```bat
"C:\Path\To\omegazip.exe" compress "%USERPROFILE%\Documents\file.txt" "%USERPROFILE%\Documents\file.oz"
```

## Linux

Добавьте в `.desktop`-файл приложения секцию **`Actions=`** и блоки `[Desktop Action …]`, вызывающие `omegazip` с полными путями. Параметр `%f` (один файл) или `%F` (несколько) задаётся в строке `Exec=`.

Иллюстративный фрагмент (подставьте путь к бинарнику; для `%f` с пробелами и для stem как в GUI лучше **`Exec=sh -c '...'`** с обёрткой, см. логику `omegazip_stem` в `scripts/install-context-menu.sh`):

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

Для папок и имён вида `archive.tar.gz` **sed с одним отрезанием суффикса** даёт иной stem, чем OmegaZip GUI; для полного совпадения используйте ту же оболочечную функцию, что в macOS-скрипте.

Установка пункта в контекстное меню файлового менеджера зависит от среды (Nautilus, Dolphin, Thunar и т.д.) — часто достаточно скопировать `.desktop` в `~/.local/share/file-manager/actions/` или использовать настройки «Открыть с помощью».

## GUI: диалог «Куда сохранить»

В десктоп-приложении при выборе файла сохранения передаётся **опционально каталог по умолчанию** (родитель пути источника или архива при экспорте в ZIP), через команду `pick_save_file` и поля `default_directory` / `defaultDirectory` (macOS: rfd; Windows/Linux: `tauri-plugin-dialog` с `set_directory`).

## Отложено

- **Трей / фоновый режим** (расширенный SHELL-03) — не входит в этот план; при необходимости отдельная фаза.

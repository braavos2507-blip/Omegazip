# Подготовка к приёмке ПКМ Windows / Linux (GAP-02)

**Автоматически в CI:** `scripts/test-context-menu-logic.sh` (bash, `pick_ext` / `stem`) + парсинг AST для `omega-context-helper.ps1` и `install-context-menu-windows.ps1` на **windows-latest** (`npm run test:context-menu` локально).

Ниже — **чеклист ручного смоука** перед релизом (Проводник / Nautilus). Основные инструкции: [CONTEXT-MENU.md](CONTEXT-MENU.md) (разделы **Windows** и **Linux**).

## Артефакты (уже в репозитории)

| Платформа | Файл | Назначение |
|-----------|------|------------|
| Windows | `scripts/install-context-menu-windows.ps1` | Установка HKCU, `-Uninstall` |
| Windows | `scripts/omega-context-helper.ps1` | Логика меню (авто .oz/.zip, stem, пресеты) — **обязателен** рядом с установщиком |
| Linux | `scripts/install-context-menu-linux.sh` | Nautilus + KDE, `--uninstall`, `--help` |
| Шаблон | `scripts/context-menu-windows.reg.example` | устар. пример без helper |

## Предварительные условия

- Собранный `omegazip` / `omegazip.exe` на пути, который передаёте в скрипт (или из bundle Tauri).
- Для **Windows**: PowerShell, выполнение скриптов разрешено (см. команды в CONTEXT-MENU).
- Для **Linux**: путь к бинарнику **абсолютный**; для Nautilus — скрипты исполняемые; для KDE — актуальная версия Plasma с `kio` сервис-меню при необходимости.

## Чеклист Windows

1. [ ] Установить пункты: `install-context-menu-windows.ps1` с реальным `-OmegaZipExe`; рядом должен лежать `omega-context-helper.ps1`.
2. [ ] ПКМ: **«Сжать OmegaZip (авто .oz/.zip)»** — на текстовом файле → `.oz`, на `.jpg`/`.zip` → `.zip` (см. `pick_ext_auto` в helper / macOS).
3. [ ] Явные **«Сжать в .oz»** / **«Сжать в .zip»**; распаковка **«Распаковать»**.
4. [ ] Имя папки распаковки для `archive.tar.gz` — `archive_распаковано`, не `archive.tar_распаковано` (regression stem).
5. [ ] `%USERPROFILE%\.config\omegazip\context_preset` (при желании) и большая папка → `--preset max`.
6. [ ] `-Uninstall` удаляет все четыре пункта из меню.
7. [ ] При необходимости — `omegazip deps` и GUI mini-check из `Правки.md`.

## Чеклист Linux

1. [ ] `./scripts/install-context-menu-linux.sh --binary /abs/path/to/omegazip`
2. [ ] **Nautilus**: файлы в `~/.local/share/nautilus/scripts/`, пункты в контекстном меню Scripts.
3. [ ] **KDE** (если есть): `~/.local/share/kio/servicemenus/omegazip.desktop`.
4. [ ] Сжатие/распаковка по сценарию, аналогичному Windows; **pick_ext_auto** как на macOS.
5. [ ] Распаковка `*.tar.gz` → каталог `archive_распаковано` (не `archive.tar_распаковано`).
6. [ ] `--uninstall` убирает установленные файлы.
7. [ ] Пресеты: `~/.config/omegazip/context_preset`, большая папка + `auto` → `max` (см. CONTEXT-MENU).

## После успешной приёмки

- Обновить [`.planning/phases/10-pkm-win-linux/10-01-SUMMARY.md`](../.planning/phases/10-pkm-win-linux/10-01-SUMMARY.md): статус QA.
- Проставить в `REQUIREMENTS.md`/`ROADMAP.md` закрытие **GAP-02** / **SHELL-02b**.

---

*Документ не изменяет поведение продуктов на Windows/Linux — только фиксирует порядок работ.*

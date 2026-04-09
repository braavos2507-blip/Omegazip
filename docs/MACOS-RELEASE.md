# macOS — локальный релизный артефакт

После сборки готовые файлы лежат в каталоге **`dist/`** (каталог в `.gitignore`, в git не попадает):

- **`dist/OmegaZip.app`** — приложение.
- **`dist/OmegaZip_0.4.0_aarch64.dmg`** — установочный DMG (версия берётся из `package.json` / Tauri).

CLI внутри bundle:

- `dist/OmegaZip.app/Contents/MacOS/omegazip`

## Как собрать

Из корня репозитория:

```bash
./build-app.sh
```

Скрипт:

1. собирает release CLI (`cargo build --release -p omegazip`);
2. запускает `npm run tauri build`;
3. копирует `OmegaZip.app` в `dist/`;
4. кладёт `omegazip` внутрь `.app`;
5. снимает quarantine (`xattr -cr`).

## Проверка после сборки

```bash
open dist/OmegaZip.app
dist/OmegaZip.app/Contents/MacOS/omegazip --help
```

Контекстное меню / Services: см. [CONTEXT-MENU.md](CONTEXT-MENU.md).

## Примечание про подпись

Без **codesign / notarization** macOS может показывать предупреждения Gatekeeper при первом запуске. Для публичного распространения нужна отдельная подготовительная процедура (вне этого документа).

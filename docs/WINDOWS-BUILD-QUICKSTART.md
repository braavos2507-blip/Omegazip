# Windows Build Quickstart

Короткая инструкция, чтобы получить готовый `.msi/.exe` на машине с Windows.

## 1) Что установить

- Node.js LTS (вместе с npm): https://nodejs.org/
- Rust toolchain: https://rustup.rs/
- Visual Studio 2022 Build Tools (компонент **Desktop development with C++**)

Проверка в PowerShell:

```powershell
node -v
npm -v
rustc -V
cargo -V
```

## 2) Сборка (одной командой)

Из корня проекта. Скрипт собирает **CLI `omegazip` как sidecar** (нужен для пунктов ПКМ) и затем Tauri.

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-windows.ps1
```

Вручную то же самое: `npm run build` (внутри — `tauri:prepare-sidecar` + `tauri build`).

Если `node_modules` уже установлены, можно быстрее:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-windows.ps1 -SkipNpmInstall
```

## 3) Где искать готовые файлы

Обычно:

- `src-tauri\target\release\bundle\msi\`
- `src-tauri\target\release\bundle\nsis\`

Скрипт сам напечатает полные пути к найденным `.msi/.exe`.

## 4) Типичные ошибки

- `link.exe not found` / ошибки MSVC:
  - не установлены Visual Studio Build Tools с C++ workload.
- `WebView2` runtime issues:
  - на тестовой машине поставьте Microsoft Edge WebView2 Runtime.
- Ошибки npm:
  - удалите `node_modules` и `package-lock.json`, затем `npm ci`.

## 5) Smoke-test на Windows

- Установить `.msi/.exe`
- Один раз запустить GUI (приложение автоматически регистрирует HKCU-контекстное меню и открытие `.oz`)
- Проверить:
  - в проводнике по ПКМ доступны действия OmegaZip (на Windows 11 — через "Показать дополнительные параметры")
  - сжатие файла/папки в `.zip` (default share-safe)
  - сжатие в `.oz` вручную (через путь назначения)
  - распаковку `.zip` и `.oz`

Если авто-регистрация не сработала (редко, из-за ограничений политики PowerShell), выполните вручную:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install-context-menu-windows.ps1 -OmegaZipExe "C:\Program Files\OmegaZip\OmegaZip.exe"
powershell -ExecutionPolicy Bypass -File .\scripts\install-oz-file-association-windows.ps1 -OmegaZipApp "C:\Program Files\OmegaZip\OmegaZip.exe"
```


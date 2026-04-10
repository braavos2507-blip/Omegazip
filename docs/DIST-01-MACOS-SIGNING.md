# DIST-01 — подпись и нотаризация macOS

Цель: воспроизводимая **Developer ID** подпись (hardened runtime) и опционально **notarization** + **staple** для распространения вне App Store.

Официальная база: [macOS Code Signing | Tauri v2](https://v2.tauri.app/distribute/sign/macos/).

## Что уже в репозитории

- **`src-tauri/entitlements/macos-release.plist`** — JIT / память для WebView (типично для Tauri); подключён в `tauri.conf.json` → `bundle.macos.entitlements`.
- **`build-app.sh`** — после копирования `omegazip` в `.app` выполняет **повторную подпись** (`codesign --deep --options runtime`), если задано `APPLE_SIGNING_IDENTITY` или `MACOS_CODESIGN_IDENTITY`. Без этого вставка CLI ломает подпись Tauri.
- **`scripts/macos-import-certificate-ci.sh`** — импорт `.p12` в keychain (CI / локально).
- **`scripts/macos-notarize-app.sh`** — отправка `.app` в Notary Service и `stapler staple`.
- **GitHub Actions**: [`.github/workflows/macos-signed-build.yml`](../.github/workflows/macos-signed-build.yml) — **только `workflow_dispatch`**, нужны secrets (см. ниже).

## Сертификат и учётные записи Apple

1. **Apple Developer Program** (платная подписка).
2. Сертификат типа **Developer ID Application** (не «Apple Development» для TestFlight-only).
3. Экспорт `.p12` из «Связка ключей» и перевод в base64 для `APPLE_CERTIFICATE` (как в документации Tauri).
4. Для нотаризации — либо **App Store Connect API** (Issuer ID, Key ID, файл `.p8`), либо **Apple ID** + **пароль приложения** + при необходимости **Team ID**.

## Локально (Mac с установленным сертификатом)

```bash
security find-identity -v -p codesigning
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
./build-app.sh
```

Preflight-отчёт перед релизом (локально):

```bash
npm run measure:release-readiness
```

Отчёт пишется в `docs/RELEASE-READINESS.md` (go/no-go, подпись, notary credentials, 7z, свежесть QA).

Нотаризация после успешной подписи:

```bash
export APPLE_API_ISSUER="..."
export APPLE_API_KEY="..."   # Key ID
export APPLE_API_KEY_PATH="$HOME/path/AuthKey_xxx.p8"
bash scripts/macos-notarize-app.sh dist/OmegaZip.app
```

## GitHub Actions — secrets

| Secret | Назначение |
|--------|------------|
| `APPLE_CERTIFICATE` | Base64 от `.p12` (Developer ID Application) |
| `APPLE_CERTIFICATE_PASSWORD` | Пароль экспорта `.p12` |
| `KEYCHAIN_PASSWORD` | Произвольный пароль для временного keychain в CI |

Опционально для нотаризации в workflow (если включите шаг вручную):

| Secret | Назначение |
|--------|------------|
| `APPLE_API_ISSUER`, `APPLE_API_KEY` | Issuer ID и Key ID |
| Файл ключа | Сохраните `.p8` как секрет (например base64 в отдельный secret) и распакуйте в job в файл; укажите `APPLE_API_KEY_PATH` |

Либо: `APPLE_ID`, `APPLE_PASSWORD` (пароль приложения), при необходимости `APPLE_TEAM_ID`.

## DMG из `target/.../bundle/dmg`

Сборка Tauri кладёт DMG **до** шага `build-app.sh`, который копирует свежий `.app` в `dist/` и встраивает CLI. **Готовый к распространению подписанный комплект** — это **`dist/OmegaZip.app`** (и при необходимости заново собранный DMG из него). DMG в `src-tauri/target/...` без встроенного CLI не использовать как финальный артефакт.

## См. также

- [MACOS-RELEASE.md](MACOS-RELEASE.md) — локальная сборка без подписи.

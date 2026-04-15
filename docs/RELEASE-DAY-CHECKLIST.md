# Release Day Checklist (macOS public)

Краткий сценарий «собрать -> подписать -> нотаризовать -> проверить -> выпуск».

## 0) Preconditions (один раз)

- Активна подписка Apple Developer Program.
- В связке ключей есть `Developer ID Application: ... (TEAMID)`.
- Есть credentials для notarytool:
  - либо `APPLE_API_ISSUER` + `APPLE_API_KEY` + `APPLE_API_KEY_PATH`,
  - либо `APPLE_ID` + `APPLE_PASSWORD` (+ `APPLE_TEAM_ID` при необходимости).

Проверка:

```bash
security find-identity -v -p codesigning
xcrun -f notarytool
```

## 1) Полный локальный контур качества

```bash
export CORPUS_EXTRA="/absolute/path/to/real-corpus"
npm run measure:everything-local
npm run measure:release-readiness
npm run measure:kpi-check
```

Ожидание:

- `docs/RELEASE-READINESS.md` -> `Overall: GO-LOCAL`
- `tests/manual-files/results-auto/KPI-CHECK-LATEST.md` -> PASS по всем строкам

## 2) Подпись Developer ID (E1)

```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
./build-app.sh
codesign --verify --deep --strict --verbose=2 dist/OmegaZip.app
codesign -dv --verbose=4 dist/OmegaZip.app 2>&1 | rg "Authority=Developer ID Application"
```

Ожидание:

- verify проходит без ошибок
- в metadata есть `Authority=Developer ID Application`

## 3) Нотаризация + staple (E2)

Вариант A: App Store Connect API

```bash
export APPLE_API_ISSUER="..."
export APPLE_API_KEY="..."          # Key ID
export APPLE_API_KEY_PATH="$HOME/path/AuthKey_xxx.p8"
bash scripts/macos-notarize-app.sh dist/OmegaZip.app
```

Вариант B: Apple ID

```bash
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="TEAMID"       # опционально
bash scripts/macos-notarize-app.sh dist/OmegaZip.app
```

Доп. проверка Gatekeeper:

```bash
spctl -a -vvv --type exec dist/OmegaZip.app
```

## 4) Артефакты релиза (zip + checksums)

```bash
cd dist
ditto -c -k --sequesterRsrc --keepParent "OmegaZip.app" "OmegaZip_0.4.0_app_macOS-arm64.zip"
shasum -a 256 "OmegaZip_0.4.0_app_macOS-arm64.zip"
```

Добавьте/обновите строку SHA256 в `dist/CHECKSUMS.txt`.

## 5) Clean-machine smoke

Прогон по `docs/CLEAN-MACHINE-SMOKE.md`, затем фиксация:

```bash
npm run measure:record-clean-machine-smoke -- PASS "OmegaZip_0.4.0_app_macOS-arm64.zip"
```

## 6) Public readiness и строгий gate

```bash
export CORPUS_EXTRA="/absolute/path/to/real-corpus"
npm run measure:release-readiness:public
npm run measure:release-gate-strict
```

Ожидание:

- `docs/RELEASE-READINESS.md` -> `Overall: GO-PUBLIC` или минимум корректный `GO-WITH-FLAGS`
- strict gate завершается `STRICT GATE PASS`

## 7) Минимум перед публикацией

- Приложите:
  - `dist/OmegaZip_...zip`
  - `dist/CHECKSUMS.txt`
  - `docs/RELEASE-READINESS.md`
  - `tests/manual-files/results-auto/KPI-CHECK-LATEST.md`
- В релиз-нотах укажите:
  - что это notarized Developer ID build,
  - известные ограничения,
  - ссылку на установку/быстрый старт.

#!/bin/bash
# Сборка OmegaZip.app + CLI и копирование в dist. Запускать из корня проекта.

set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

echo "Сборка: sidecar omegazip (externalBin) + Tauri bundle..."
unset CI
npm run build

# Bundle лежит в target-dir пакета src-tauri (при CARGO_TARGET_DIR — вне src-tauri/target).
if [[ -n "${CARGO_TARGET_DIR:-}" && -d "$CARGO_TARGET_DIR/release/bundle/macos/OmegaZip.app" ]]; then
  APP_SRC="$CARGO_TARGET_DIR/release/bundle/macos/OmegaZip.app"
else
  APP_SRC="$ROOT/src-tauri/target/release/bundle/macos/OmegaZip.app"
fi
if [ ! -d "$APP_SRC" ]; then
  echo "Ошибка: $APP_SRC не найден. Сборка могла завершиться в другой папке."
  exit 1
fi

mkdir -p dist
rm -rf dist/OmegaZip.app
cp -R "$APP_SRC" dist/

BUNDLE_DMG_DIR="$(cd "$(dirname "$APP_SRC")/../dmg" && pwd)"
DMG_BUILT="$(ls -1t "$BUNDLE_DMG_DIR"/OmegaZip_*.dmg 2>/dev/null | head -n 1 || true)"
if [[ -n "$DMG_BUILT" && -f "$DMG_BUILT" ]]; then
  cp -f "$DMG_BUILT" dist/
  echo "DMG скопирован: dist/$(basename "$DMG_BUILT")"
fi

# Вложить CLI в .app — тогда контекстное меню (Quick Action) и алиас могут вызывать его
CLI_SRC="$ROOT/target/release/omegazip"
if [ -x "$CLI_SRC" ]; then
  cp "$CLI_SRC" dist/OmegaZip.app/Contents/MacOS/omegazip
  echo "CLI скопирован в OmegaZip.app (Contents/MacOS/omegazip)."
else
  echo "Предупреждение: CLI не найден ($CLI_SRC), в .app только GUI."
fi

# Очистить расширенные атрибуты до подписи (иначе codesign может падать на detritus).
xattr -cr dist/OmegaZip.app

# После копирования omegazip подпись Tauri недействительна — переподписать.
SIGN_ID="${APPLE_SIGNING_IDENTITY:-${MACOS_CODESIGN_IDENTITY:-}}"
ENT="$ROOT/src-tauri/entitlements/macos-release.plist"
echo "Удаление старой подписи bundle..."
codesign --remove-signature "dist/OmegaZip.app" 2>/dev/null || true

if [[ -n "$SIGN_ID" && -f "$ENT" ]]; then
  echo "Повторная подпись bundle (Developer ID, hardened runtime)..."
  codesign --force --deep --sign "$SIGN_ID" --options runtime --timestamp \
    --entitlements "$ENT" \
    "dist/OmegaZip.app"
else
  echo "APPLE_SIGNING_IDENTITY не задан — применяю ad-hoc подпись для локальной верификации."
  codesign --force --deep --sign - "dist/OmegaZip.app"
fi

echo "Снятие карантина macOS..."
xattr -cr dist/OmegaZip.app

echo "Готово: dist/OmegaZip.app"
echo "Запуск GUI: open dist/OmegaZip.app"
echo "Сжатие из терминала: dist/OmegaZip.app/Contents/MacOS/omegazip compress <файл_или_папка> [выход.oz]"

#!/bin/bash
# Сборка OmegaZip.app + CLI и копирование в dist. Запускать из корня проекта.

set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

echo "Сборка CLI (omegazip)..."
cargo build --release -p omegazip

echo "Сборка приложения (Tauri)..."
unset CI
npm run tauri build

APP_SRC="src-tauri/target/release/bundle/macos/OmegaZip.app"
if [ ! -d "$APP_SRC" ]; then
  echo "Ошибка: $APP_SRC не найден. Сборка могла завершиться в другой папке."
  exit 1
fi

mkdir -p dist
rm -rf dist/OmegaZip.app
cp -R "$APP_SRC" dist/

# Вложить CLI в .app — тогда контекстное меню (Quick Action) и алиас могут вызывать его
CLI_SRC="$ROOT/target/release/omegazip"
if [ -x "$CLI_SRC" ]; then
  cp "$CLI_SRC" dist/OmegaZip.app/Contents/MacOS/omegazip
  echo "CLI скопирован в OmegaZip.app (Contents/MacOS/omegazip)."
else
  echo "Предупреждение: CLI не найден ($CLI_SRC), в .app только GUI."
fi

echo "Снятие карантина macOS..."
xattr -cr dist/OmegaZip.app

echo "Готово: dist/OmegaZip.app"
echo "Запуск GUI: open dist/OmegaZip.app"
echo "Сжатие из терминала: dist/OmegaZip.app/Contents/MacOS/omegazip compress <файл_или_папка> [выход.oz]"

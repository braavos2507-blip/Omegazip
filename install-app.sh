#!/bin/bash
# Установка OmegaZip.app в /Applications. Запускать после build-app.sh.

set -e
cd "$(dirname "$0")"

APP_SRC="dist/OmegaZip.app"
if [ ! -d "$APP_SRC" ]; then
  echo "Сначала соберите приложение: ./build-app.sh"
  exit 1
fi

echo "Установка OmegaZip в /Applications..."
rm -rf /Applications/OmegaZip.app
cp -R "$APP_SRC" /Applications/
xattr -cr /Applications/OmegaZip.app

echo "Готово. OmegaZip установлен в /Applications/OmegaZip.app"
echo "Запуск: open -a OmegaZip"

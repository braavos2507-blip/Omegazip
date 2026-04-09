#!/usr/bin/env bash
# Быстрое действие Finder: распаковать выбранные .oz в папки «имя_распаковано».
# В Automator: «Запустить сценарий оболочки», «Передать как аргументы», путь к этому скрипту и "$@".

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OZ="/Applications/OmegaZip.app/Contents/MacOS/omegazip"
[[ -x "$OZ" ]] || OZ="$ROOT/dist/OmegaZip.app/Contents/MacOS/omegazip"
[[ -x "$OZ" ]] || { echo "OmegaZip CLI не найден. Соберите: ./build-app.sh"; exit 1; }

for f in "$@"; do
  [[ -e "$f" ]] || continue
  case "$f" in *.oz)
    dir="${f%.oz}_распаковано"
    "$OZ" decompress "$f" "$dir" && echo "Распаковано: $dir"
  ;; esac
done

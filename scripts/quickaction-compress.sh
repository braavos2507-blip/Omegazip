#!/usr/bin/env bash
# Быстрое действие Finder: сжать выбранные файлы/папки в .oz (рядом с каждым).
# В Automator: «Запустить сценарий оболочки», «Передать как аргументы», вызвать этот скрипт с "$@".
# Либо скопировать тело цикла в сценарий Automator (см. docs/ИСПОЛЬЗОВАНИЕ_НА_МАКЕ.md).

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OZ="/Applications/OmegaZip.app/Contents/MacOS/omegazip"
[[ -x "$OZ" ]] || OZ="$ROOT/dist/OmegaZip.app/Contents/MacOS/omegazip"
[[ -x "$OZ" ]] || { echo "OmegaZip CLI не найден. Соберите: ./build-app.sh"; exit 1; }

for f in "$@"; do
  [[ -e "$f" ]] || continue
  out="${f}.oz"
  "$OZ" compress "$f" "$out" && echo "Сжато: $out"
done

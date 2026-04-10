#!/usr/bin/env bash
# B2 (часть): ZIP vs chunked .oz на **каталогах из репозитория** (manual-files/downloads/*/).
# Свой большой корпус: CORPUS_EXTRA=/path/to/dir (один доп. прогон в конце).
#
# Использование: из корня репозитория — bash scripts/measure-oz-repo-corpora.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DOWN="$ROOT/tests/manual-files/downloads"

if [[ ! -d "$DOWN" ]]; then
  echo "Нет каталога: $DOWN" >&2
  exit 1
fi

shopt -s nullglob
corpora=( "$DOWN"/*/ )
shopt -u nullglob

if [[ ${#corpora[@]} -eq 0 ]]; then
  echo "Подкаталогов в $DOWN нет — пропуск."
  exit 0
fi

for c in "${corpora[@]}"; do
  c="${c%/}"
  echo ""
  echo "########################################"
  echo "### Корпус: ${c#$ROOT/}"
  echo "########################################"
  CORPUS_DIR="$c" bash "$ROOT/scripts/measure-oz-advantage.sh"
done

if [[ -n "${CORPUS_EXTRA:-}" ]]; then
  echo ""
  echo "########################################"
  echo "### Корпус (CORPUS_EXTRA): $CORPUS_EXTRA"
  echo "########################################"
  CORPUS_DIR="$CORPUS_EXTRA" bash "$ROOT/scripts/measure-oz-advantage.sh"
fi

echo ""
echo "Готово. Свой путь: CORPUS_EXTRA=/ваш/каталог bash scripts/measure-oz-repo-corpora.sh"

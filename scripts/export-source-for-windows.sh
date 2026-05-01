#!/usr/bin/env bash
# Архив исходников без node_modules/target — удобно перенести на Windows (git не тянет кэши).
# Запуск из корня репозитория: bash scripts/export-source-for-windows.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
OUT="${1:-$ROOT/OmegaZip-source-for-windows.zip}"
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "ERROR: not a git repository" >&2
  exit 1
fi
git archive --format=zip -o "$OUT" HEAD
echo "Created: $OUT"
echo "On Windows: unzip, then npm ci && powershell -ExecutionPolicy Bypass -File .\\scripts\\build-windows.ps1"

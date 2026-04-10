#!/usr/bin/env bash
# Записывает tests/manual-files/results-auto/CLEAN-MACHINE-SMOKE-LATEST.md для public release gate.
# Использование:
#   bash scripts/record-clean-machine-smoke.sh PASS "OmegaZip_0.4.0_x64.dmg"
#   bash scripts/record-clean-machine-smoke.sh FAIL "сборка не запускается на чистой VM"
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$ROOT/tests/manual-files/results-auto"
OUT="$OUT_DIR/CLEAN-MACHINE-SMOKE-LATEST.md"
mkdir -p "$OUT_DIR"

STATUS="${1:?usage: record-clean-machine-smoke.sh PASS|FAIL [artifact_or_notes]}"
NOTES="${2:-}"
now="$(date -Iseconds 2>/dev/null || date)"

{
  echo "# Clean-machine smoke record (auto)"
  echo ""
  echo "**Date:** $now"
  echo "**Status:** $STATUS"
  echo "**Artifact / notes:** $NOTES"
  echo ""
  echo "Чеклист: docs/CLEAN-MACHINE-SMOKE.md"
} >"$OUT"

echo "Written: $OUT"
if [[ "$STATUS" != "PASS" && "$STATUS" != "FAIL" ]]; then
  echo "Предупреждение: для gate нужен статус PASS или FAIL (сейчас: $STATUS)" >&2
fi

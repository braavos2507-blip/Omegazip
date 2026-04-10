#!/usr/bin/env bash
# Локальный «срез» измерений: QA-03 + bench + преимущество .oz.
# Лог пишется в tests/manual-files/results-auto/baselines/

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

STAMP="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="$ROOT/tests/manual-files/results-auto/baselines"
mkdir -p "$OUT_DIR"
LOG="$OUT_DIR/baseline-${STAMP}.log"

{
  echo "OmegaZip baseline-all-local — $STAMP"
  echo "uname: $(uname -a)"
  echo "rustc: $(rustc -V 2>/dev/null || echo n/a)"
  echo ""

  echo "========== qa03-benchmark-ci =========="
  bash "$ROOT/scripts/qa03-benchmark-ci.sh"
  echo ""

  echo "========== bench.sh (presets) =========="
  bash "$ROOT/scripts/bench.sh"
  echo ""

  echo "========== measure-oz-advantage =========="
  bash "$ROOT/scripts/measure-oz-advantage.sh"
  echo ""

  echo "========== done =========="
} 2>&1 | tee "$LOG"

echo ""
echo "Лог: $LOG"

#!/usr/bin/env bash
# Строгий релизный gate: падает при FLAG/BLOCK у E1/E2/smoke + KPI fail.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MODE="${RELEASE_MODE:-public}"

# Важно: не вызывать npm run measure:release-readiness — там жёстко RELEASE_MODE=local.
export CORPUS_EXTRA="${CORPUS_EXTRA:-}"
RELEASE_MODE="$MODE" bash "$ROOT/scripts/release-readiness-local.sh" >/tmp/oz-release-gate-readiness.log 2>&1

npm run measure:kpi-check >/tmp/oz-release-gate-kpi.log 2>&1

MD="$ROOT/docs/RELEASE-READINESS.md"
if [[ ! -f "$MD" ]]; then
  echo "Нет $MD" >&2
  exit 1
fi

e1="$(rg "Signed app ready \\(E1\\)" "$MD" || true)"
e2="$(rg "Notarization preflight \\(E2\\)" "$MD" || true)"
smoke="$(rg "Clean-machine release smoke" "$MD" || true)"

fail=0
if [[ "$MODE" == "local" ]]; then
  # В local-режиме E1/E2/smoke могут быть N/A.
  true
else
  for row in "$e1" "$e2" "$smoke"; do
    if [[ -z "$row" ]]; then
      fail=1
      continue
    fi
    # Таблица: | Gate | PASS | details |
    if [[ ! "$row" =~ \|[[:space:]]*PASS[[:space:]]*\| ]]; then
      fail=1
    fi
  done
fi

if [[ "$fail" -ne 0 ]]; then
  echo "STRICT GATE FAIL: требуется PASS для E1/E2/Clean-machine smoke (режим public)."
  echo "См. $MD"
  exit 1
fi

echo "STRICT GATE PASS"

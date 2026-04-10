#!/usr/bin/env bash
# KPI gate для ниши: размер, скорость, критические ошибки D-checklist.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$ROOT/tests/manual-files/results-auto"
OUT_MD="$OUT_DIR/KPI-CHECK-LATEST.md"
mkdir -p "$OUT_DIR"

KPI_MIN_OZ_ADVANTAGE_PCT="${KPI_MIN_OZ_ADVANTAGE_PCT:-10}"
KPI_MAX_TIME_REGRESSION_PCT="${KPI_MAX_TIME_REGRESSION_PCT:-150}"

cd "$ROOT"

# D-checklist: должен быть без ошибок.
npm run measure:checklist-d >/tmp/oz-kpi-d.log 2>&1 || {
  echo "KPI FAIL: D-checklist вернул ошибку"
  cat /tmp/oz-kpi-d.log
  exit 1
}

PROFILE="$OUT_DIR/profile-smoke-last.txt"
if [[ ! -f "$PROFILE" ]]; then
  echo "Нет $PROFILE — сначала выполните npm run measure:profile-smoke" >&2
  exit 1
fi

zip_t="$(rg "^zip_time_s:" "$PROFILE" | awk '{print $2}')"
oz_t="$(rg "^oz_chunked_balanced_time_s:" "$PROFILE" | awk '{print $2}')"
oz_adv="$(rg "^oz_vs_zip_size_pct:" "$PROFILE" | awk '{print $2}' | tr -d '%')"

if [[ -z "$zip_t" || -z "$oz_t" || -z "$oz_adv" ]]; then
  echo "Не удалось прочитать KPI из $PROFILE" >&2
  exit 1
fi

regression_pct="$(python3 - <<PY
z=float("$zip_t")
o=float("$oz_t")
print((o/z - 1.0) * 100.0 if z>0 else 0.0)
PY
)"

size_ok="$(python3 - <<PY
print("1" if float("$oz_adv") >= float("$KPI_MIN_OZ_ADVANTAGE_PCT") else "0")
PY
)"

time_ok="$(python3 - <<PY
print("1" if float("$regression_pct") <= float("$KPI_MAX_TIME_REGRESSION_PCT") else "0")
PY
)"

cat >"$OUT_MD" <<EOF
# KPI check (local)

- Generated: $(date -Iseconds 2>/dev/null || date)
- Source: \`$PROFILE\`

## Targets

- \`oz_vs_zip_size_pct >= $KPI_MIN_OZ_ADVANTAGE_PCT%\`
- \`time_regression_pct <= $KPI_MAX_TIME_REGRESSION_PCT%\`
- D-checklist: no critical errors

## Results

| Metric | Value | Target | Status |
|---|---:|---:|---|
| oz_vs_zip_size_pct | $oz_adv% | >= $KPI_MIN_OZ_ADVANTAGE_PCT% | $([[ "$size_ok" == "1" ]] && echo PASS || echo FAIL) |
| time_regression_pct | $(printf "%.1f" "$regression_pct")% | <= $KPI_MAX_TIME_REGRESSION_PCT% | $([[ "$time_ok" == "1" ]] && echo PASS || echo FAIL) |
| D-checklist critical errors | 0 | 0 | PASS |
EOF

if [[ "$size_ok" != "1" || "$time_ok" != "1" ]]; then
  echo "KPI FAIL. См. $OUT_MD"
  exit 1
fi

echo "KPI PASS. Report: $OUT_MD"

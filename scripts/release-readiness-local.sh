#!/usr/bin/env bash
# Локальный preflight перед релизом: формирует docs/RELEASE-READINESS.md
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MODE="${RELEASE_MODE:-local}"

OUT="$ROOT/docs/RELEASE-READINESS.md"
RESULTS_DIR="$ROOT/tests/manual-files/results-auto"
LATEST_QA_MD="$RESULTS_DIR/LATEST-FULL-QA.md"
LATEST_LOG="$(ls -1t "$RESULTS_DIR"/baselines/full-qa-*.log 2>/dev/null | head -n 1 || true)"

today="$(date +%F)"
now_iso="$(date -Iseconds 2>/dev/null || date)"

status_qa="BLOCK"
details_qa="Нет $LATEST_QA_MD"
if [[ -f "$LATEST_QA_MD" ]]; then
  if rg -q "$today" "$LATEST_QA_MD"; then
    status_qa="PASS"
    details_qa="Есть свежий полный QA за $today"
  else
    status_qa="FLAG"
    details_qa="Есть LATEST-FULL-QA.md, но не от сегодняшней даты"
  fi
fi

status_corpus="FLAG"
details_corpus="Нужен запуск с CORPUS_EXTRA=/ваш/реальный/корпус"
if [[ -n "${CORPUS_EXTRA:-}" && -d "${CORPUS_EXTRA:-}" ]]; then
  corpus_files="$( (rg --files "$CORPUS_EXTRA" 2>/dev/null || true) | wc -l | tr -d ' ')"
  if [[ "${corpus_files:-0}" -gt 0 ]]; then
    status_corpus="PASS"
    details_corpus="CORPUS_EXTRA задан: $CORPUS_EXTRA (файлов: $corpus_files)"
  else
    status_corpus="FLAG"
    details_corpus="CORPUS_EXTRA задан, но файлов не найдено: $CORPUS_EXTRA"
  fi
fi

SEVEN_ZIP_BIN=""
if command -v 7z >/dev/null 2>&1; then
  SEVEN_ZIP_BIN="$(command -v 7z)"
elif command -v 7zz >/dev/null 2>&1; then
  SEVEN_ZIP_BIN="$(command -v 7zz)"
elif [[ -x "/opt/homebrew/opt/sevenzip/bin/7zz" ]]; then
  SEVEN_ZIP_BIN="/opt/homebrew/opt/sevenzip/bin/7zz"
elif [[ -x "/usr/local/opt/sevenzip/bin/7zz" ]]; then
  SEVEN_ZIP_BIN="/usr/local/opt/sevenzip/bin/7zz"
fi
if [[ -n "$SEVEN_ZIP_BIN" ]]; then
  status_7z="PASS"
  details_7z="Найден: $SEVEN_ZIP_BIN"
else
  status_7z="FLAG"
  details_7z="7z/7zz не найден в PATH (RAR/7z ограничены)"
fi

APP="$ROOT/dist/OmegaZip.app"
if [[ -d "$APP" ]]; then
  if codesign --verify --deep --strict "$APP" >/tmp/oz-sign-verify.log 2>&1; then
    sign_meta="$(codesign -dv --verbose=4 "$APP" 2>&1 || true)"
    if printf '%s\n' "$sign_meta" | rg -q "Authority=Developer ID Application"; then
      status_sign="PASS"
      details_sign="Подпись валидна и содержит Developer ID Application"
    else
      status_sign="FLAG"
      details_sign="Подпись валидна, но не Developer ID (локальная/ad-hoc)"
    fi
  else
    status_sign="FLAG"
    details_sign="dist/OmegaZip.app есть, но verify не проходит (см. /tmp/oz-sign-verify.log)"
  fi
else
  status_sign="BLOCK"
  details_sign="Нет dist/OmegaZip.app (соберите через ./build-app.sh)"
fi

if xcrun -f notarytool >/dev/null 2>&1; then
  has_notarytool=1
else
  has_notarytool=0
fi
if [[ $has_notarytool -eq 1 ]] && {
  [[ -n "${APPLE_API_ISSUER:-}" && -n "${APPLE_API_KEY:-}" && -n "${APPLE_API_KEY_PATH:-}" ]] || \
  [[ -n "${APPLE_ID:-}" && -n "${APPLE_PASSWORD:-}" ]];
}; then
  status_notary="PASS"
  details_notary="Предпосылки нотаризации готовы (notarytool + credentials)"
elif [[ $has_notarytool -eq 1 ]]; then
  status_notary="FLAG"
  details_notary="notarytool есть, но не заданы credentials Apple"
else
  status_notary="BLOCK"
  details_notary="notarytool не найден (Xcode CLT/Xcode не готовы)"
fi

SMOKE_FILE="$RESULTS_DIR/CLEAN-MACHINE-SMOKE-LATEST.md"
status_smoke="FLAG"
details_smoke="Требуется ручной прогон на чистой машине (см. docs/CLEAN-MACHINE-SMOKE.md), затем запись PASS."
if [[ "$MODE" == "local" ]]; then
  status_smoke="N/A"
  details_smoke="Локальная разработка: clean-machine smoke не обязателен."
elif [[ "$MODE" == "public" ]]; then
  if [[ -f "$SMOKE_FILE" ]] && rg -q '\*\*Status:\*\*[[:space:]]*(PASS|GO\b)' "$SMOKE_FILE" 2>/dev/null; then
    status_smoke="PASS"
    details_smoke="Запись: $SMOKE_FILE (найден **Status:** PASS/GO)"
  elif [[ -f "$SMOKE_FILE" ]] && rg -q '^Status:[[:space:]]*PASS' "$SMOKE_FILE" 2>/dev/null; then
    status_smoke="PASS"
    details_smoke="Запись: $SMOKE_FILE"
  else
    status_smoke="BLOCK"
    details_smoke="Нет валидной записи PASS. Пример: tests/manual-files/results-auto/CLEAN-MACHINE-SMOKE-LATEST.md.example → CLEAN-MACHINE-SMOKE-LATEST.md или bash scripts/record-clean-machine-smoke.sh PASS \"артефакт\""
  fi
fi

if [[ "$MODE" == "local" ]]; then
  if [[ "$status_sign" == "FLAG" || "$status_sign" == "BLOCK" ]]; then
    status_sign="N/A"
    details_sign="Локальная разработка: Developer ID подпись не требуется."
  fi
  if [[ "$status_notary" == "FLAG" || "$status_notary" == "BLOCK" ]]; then
    status_notary="N/A"
    details_notary="Локальная разработка: notarization не требуется."
  fi
fi

overall="NO-GO"
if [[ "$MODE" == "local" ]]; then
  if [[ "$status_qa" == "PASS" && "$status_corpus" == "PASS" && "$status_7z" == "PASS" ]]; then
    overall="GO-LOCAL"
  fi
elif [[ "$MODE" == "public" ]]; then
  if [[ "$status_qa" == "PASS" && "$status_corpus" == "PASS" && "$status_7z" == "PASS" \
        && "$status_sign" == "PASS" && "$status_notary" == "PASS" && "$status_smoke" == "PASS" ]]; then
    overall="GO-PUBLIC"
  elif [[ "$status_qa" == "PASS" && "$status_sign" != "BLOCK" && "$status_notary" != "BLOCK" ]]; then
    overall="GO-WITH-FLAGS"
  fi
else
  if [[ "$status_qa" == "PASS" && "$status_sign" != "BLOCK" && "$status_notary" != "BLOCK" ]]; then
    overall="GO-WITH-FLAGS"
  fi
fi

cat >"$OUT" <<EOF
# RELEASE READINESS ($MODE)

**Generated:** $now_iso  
**Mode:** **$MODE**  
**Overall:** **$overall**

## Gate Status

| Gate | Status | Details |
|---|---|---|
| Full local QA | $status_qa | $details_qa |
| Real corpus benchmark (A4/B2) | $status_corpus | $details_corpus |
| 7-Zip dependency (D10) | $status_7z | $details_7z |
| Signed app ready (E1) | $status_sign | $details_sign |
| Notarization preflight (E2) | $status_notary | $details_notary |
| Clean-machine release smoke | $status_smoke | $details_smoke |

## Artifacts

- LATEST QA markdown: \`tests/manual-files/results-auto/LATEST-FULL-QA.md\`
- LATEST QA log: \`${LATEST_LOG:-<not found>}\`
- Bench workflow report: \`tests/manual-files/results-auto/BENCH-WORKFLOW-LATEST.md\`
- Signing/notary guide: [DIST-01-MACOS-SIGNING.md](DIST-01-MACOS-SIGNING.md)
- Clean-machine checklist: [CLEAN-MACHINE-SMOKE.md](CLEAN-MACHINE-SMOKE.md)
- GTM / честные формулировки: [GO-TO-MARKET-CHECKLIST.md](GO-TO-MARKET-CHECKLIST.md)
- CI: полная сборка установщиков — GitHub Actions workflow tauri-bundles (\`.github/workflows/tauri-bundles.yml\`)

## Next Commands

\`\`\`bash
# 1) Полный локальный контур
npm run measure:everything-local

# 2) Реальные корпуса (обязательно перед публичным релизом)
CORPUS_EXTRA=/absolute/path/to/real-corpus npm run measure:oz-repo-corpora

# 3) Проверка D-чеклиста (включая D10b при наличии 7z)
npm run measure:checklist-d

# 4) Сборка + подпись + нотаризация (если есть сертификаты/credentials)
./build-app.sh
bash scripts/macos-notarize-app.sh dist/OmegaZip.app

# 5) Clean-machine smoke → запись PASS (public gate)
bash scripts/record-clean-machine-smoke.sh PASS "dist/OmegaZip.app или DMG"

# 6) KPI и строгий релизный gate (public: E1/E2/smoke = PASS)
CORPUS_EXTRA=/absolute/path/to/real-corpus npm run measure:kpi-check
CORPUS_EXTRA=/absolute/path/to/real-corpus RELEASE_MODE=public bash scripts/release-gate-strict.sh
\`\`\`
EOF

echo "Written: $OUT"

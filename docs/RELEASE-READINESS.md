# RELEASE READINESS (public)

**Generated:** 2026-04-10T22:56:12+04:00  
**Mode:** **public**  
**Overall:** **GO-WITH-FLAGS**

## Gate Status

| Gate | Status | Details |
|---|---|---|
| Full local QA | PASS | Есть свежий полный QA за 2026-04-10 |
| Real corpus benchmark (A4/B2) | PASS | CORPUS_EXTRA задан: /Users/renat/01Project/OmegaZip/Архивы (файлов: 9235) |
| 7-Zip dependency (D10) | PASS | Найден: /opt/homebrew/bin/7zz |
| Signed app ready (E1) | FLAG | Подпись валидна, но не Developer ID (локальная/ad-hoc) |
| Notarization preflight (E2) | FLAG | notarytool есть, но не заданы credentials Apple |
| Clean-machine release smoke | PASS | Запись: /Users/renat/01Project/OmegaZip/tests/manual-files/results-auto/CLEAN-MACHINE-SMOKE-LATEST.md (найден **Status:** PASS/GO) |

## Artifacts

- LATEST QA markdown: `tests/manual-files/results-auto/LATEST-FULL-QA.md`
- LATEST QA log: `/Users/renat/01Project/OmegaZip/tests/manual-files/results-auto/baselines/full-qa-20260410-225541.log`
- Bench workflow report: `tests/manual-files/results-auto/BENCH-WORKFLOW-LATEST.md`
- Signing/notary guide: [DIST-01-MACOS-SIGNING.md](DIST-01-MACOS-SIGNING.md)
- Clean-machine checklist: [CLEAN-MACHINE-SMOKE.md](CLEAN-MACHINE-SMOKE.md)
- GTM / честные формулировки: [GO-TO-MARKET-CHECKLIST.md](GO-TO-MARKET-CHECKLIST.md)
- CI: полная сборка установщиков — GitHub Actions workflow tauri-bundles (`.github/workflows/tauri-bundles.yml`)

## Next Commands

```bash
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
```

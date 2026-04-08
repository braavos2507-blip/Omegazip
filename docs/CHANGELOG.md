# Changelog

## 2026-04-08

### docs(macos): release checklist and benchmark CSV
- Restored root `README.md`; added `.gitignore` for `node_modules`, `target`, `dist`, `out`.
- `scripts/benchmark-workflow.sh`: column `extracted_bytes` for decompress rows (sum of extracted file sizes); report table distinguishes compress vs decompress.
- `docs/MACOS-QUICKSTART.md` — установка, сервисы, `/tmp/OmegaZip-workflow.log`, переустановка.
- `docs/CONTEXT-MENU.md`: секция «Диагностика».
- `scripts/install-context-menu.sh`: `base64 -i` для совместимости с macOS.

### fix(macos): stabilize Finder context actions via workflows (`9686603`)
- Switched macOS context actions to Automator workflow services that call `omegazip` CLI directly.
- Added resilient Finder selection fallback in workflows when direct input is not passed.
- Updated `docs/CONTEXT-MENU.md` with the 2-item workflow setup and duplicate-menu cleanup guidance.

### chore(macos): remove temporary service diagnostics (`94457e0`)
- Removed temporary debug tracing and `/tmp` diagnostics from the macOS service/open-file flow in `src-tauri`.
- Kept the final silent handling behavior while cleaning up instrumentation added during investigation.

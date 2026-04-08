# Changelog

## 2026-04-08

### fix(macos): stabilize Finder context actions via workflows (`9686603`)
- Switched macOS context actions to Automator workflow services that call `omegazip` CLI directly.
- Added resilient Finder selection fallback in workflows when direct input is not passed.
- Updated `docs/CONTEXT-MENU.md` with the 2-item workflow setup and duplicate-menu cleanup guidance.

### chore(macos): remove temporary service diagnostics (`94457e0`)
- Removed temporary debug tracing and `/tmp` diagnostics from the macOS service/open-file flow in `src-tauri`.
- Kept the final silent handling behavior while cleaning up instrumentation added during investigation.

# Changelog

## 2026-04-08

### feat(macos): configurable aggressive `.oz` preset (Finder Services)
- `~/.config/omegazip/context_preset` or `OMEGAZIP_CONTEXT_PRESET`: `auto` | `max` | `ultra` (`aggressive` = `max`).
- Optional `OMEGAZIP_AUTO_UPGRADE_FOLDER_MB`: when preset is `auto`, folders over threshold use `--preset max`.
- Example: [config/omegazip/context_preset.example](config/omegazip/context_preset.example).

### feat(cli+macos): `--preset auto` for `.oz` from Services + smarter PDF/EPUB presets
- Finder workflow: `omegazip compress --preset auto` when output is `.oz` (balanced → chunked dedup; fast для медиа/архивов).
- `smart_preset`: PDF/EPUB → **balanced** (раньше PDF/EPUB давали fast без чанков); добавлены `fb2`, `mobi`, `azw`, `azw3` как текстовые.
- `benchmark-workflow.sh`: для `.oz` тот же `--preset auto`.

### fix(macos): default `.oz` in `pick_ext_auto` (blacklist → `.zip`)
- Replaced extension whitelist with **default `.oz`** and a **blacklist** for media, existing archives, fonts, executables, disk images, sqlite DB files.
- PDF, EPUB, DOCX and other text-heavy types now use **`.oz`** from Finder Services.

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

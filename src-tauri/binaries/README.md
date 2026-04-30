# Tauri `externalBin` (omegazip CLI)

The Windows Explorer context menu calls the **`omegazip` CLI** (`compress` / `decompress`), not the GUI `OmegaZip.exe`. Tauri bundles this binary as a **sidecar** next to the main executable.

## Local / CI

Before `tauri build`, generate the correctly named file:

```bash
npm run tauri:prepare-sidecar
```

This runs `cargo build --release -p omegazip` and copies the binary to:

`omegazip-<rustc host-tuple>[.exe]`

Files in this directory are gitignored; do not commit them.

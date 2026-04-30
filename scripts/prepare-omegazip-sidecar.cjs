// Builds the omegazip CLI and copies it to src-tauri/binaries/ with the Tauri externalBin naming scheme.
// Run from repo root: node scripts/prepare-omegazip-sidecar.cjs

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const root = path.join(__dirname, "..");
const ext = process.platform === "win32" ? ".exe" : "";
const triple = execSync("rustc --print host-tuple", { encoding: "utf8" }).trim();
if (!triple) {
  console.error("Failed to read rustc --print host-tuple");
  process.exit(1);
}

execSync("cargo build --release -p omegazip", { cwd: root, stdio: "inherit" });

const src = path.join(root, "target", "release", "omegazip" + ext);
if (!fs.existsSync(src)) {
  console.error("Expected binary not found:", src);
  process.exit(1);
}

const destDir = path.join(root, "src-tauri", "binaries");
fs.mkdirSync(destDir, { recursive: true });
const dest = path.join(destDir, `omegazip-${triple}${ext}`);
fs.copyFileSync(src, dest);
try {
  fs.chmodSync(dest, 0o755);
} catch (_) {
  // Windows may ignore chmod
}
console.log("Sidecar ready:", dest);

#!/usr/bin/env bash
# Локальный аудит зависимостей (Rust root + src-tauri + npm). См. docs/SECURITY.md
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v cargo-audit >/dev/null 2>&1; then
  echo "Установите cargo-audit: cargo install cargo-audit --locked" >&2
  exit 1
fi

echo "=== cargo audit (omegazip, корень) ==="
cargo audit

echo "=== cargo audit (src-tauri) ==="
( cd "$ROOT/src-tauri" && cargo audit )

if [[ ! -f "$ROOT/package-lock.json" ]]; then
  echo "Нет package-lock.json — пропуск npm audit" >&2
  exit 0
fi

echo "=== npm audit (high+) ==="
cd "$ROOT"
npm audit --audit-level=high

echo "OK: security-audit.sh завершён"

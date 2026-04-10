#!/usr/bin/env bash
# Импорт Developer ID (.p12) во временный keychain для CI или локальной имитации.
# Переменные: APPLE_CERTIFICATE (base64 .p12), APPLE_CERTIFICATE_PASSWORD, KEYCHAIN_PASSWORD.
# Результат: пишет APPLE_SIGNING_IDENTITY в GITHUB_ENV или печатает export.
#
# См. docs/DIST-01-MACOS-SIGNING.md

set -euo pipefail

: "${APPLE_CERTIFICATE:?Set APPLE_CERTIFICATE (base64 .p12)}"
: "${APPLE_CERTIFICATE_PASSWORD:?}"
: "${KEYCHAIN_PASSWORD:?}"

TMP_P12="$(mktemp -t omegazip-cert).p12"
cleanup() { rm -f "$TMP_P12"; }
trap cleanup EXIT

echo "$APPLE_CERTIFICATE" | base64 --decode >"$TMP_P12"

KEYCHAIN="${KEYCHAIN_NAME:-omegazip-build.keychain}"
security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
security default-keychain -s "$KEYCHAIN"
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
security set-keychain-settings -t 3600 -u "$KEYCHAIN"
security import "$TMP_P12" -k "$KEYCHAIN" -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN"

IDENT="$(security find-identity -v -p codesigning "$KEYCHAIN" 2>/dev/null | sed -n 's/.*"\(Developer ID Application:[^"]*\)".*/\1/p' | head -n 1)"

if [[ -z "$IDENT" ]]; then
  echo "macos-import-certificate-ci: не найдена личность «Developer ID Application»." >&2
  security find-identity -v -p codesigning "$KEYCHAIN" >&2 || true
  exit 1
fi

if [[ -n "${GITHUB_ENV:-}" ]]; then
  echo "APPLE_SIGNING_IDENTITY=$IDENT" >>"$GITHUB_ENV"
  echo "macos-import-certificate-ci: APPLE_SIGNING_IDENTITY записан в GITHUB_ENV."
else
  echo "export APPLE_SIGNING_IDENTITY=\"$IDENT\""
  echo "# Добавьте в окружение перед ./build-app.sh"
fi

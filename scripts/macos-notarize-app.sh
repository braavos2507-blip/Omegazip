#!/usr/bin/env bash
# Нотаризация уже подписанного .app через notarytool и stapler.
# Требуется один из наборов:
#   • App Store Connect API: APPLE_API_ISSUER, APPLE_API_KEY (Key ID), APPLE_API_KEY_PATH (.p8)
#   • Apple ID: APPLE_ID, APPLE_PASSWORD (пароль приложения), APPLE_TEAM_ID
#
# Usage: bash scripts/macos-notarize-app.sh [path/to/App.app]
# По умолчанию: dist/OmegaZip.app
#
# См. docs/DIST-01-MACOS-SIGNING.md

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="${1:-$ROOT/dist/OmegaZip.app}"

if [[ ! -d "$APP" ]]; then
  echo "Не найден bundle: $APP" >&2
  exit 1
fi

WORKDIR="$(mktemp -d)"
cleanup() { rm -rf "$WORKDIR"; }
trap cleanup EXIT

NAME="$(basename "$APP" .app)"
ZIP="$WORKDIR/${NAME}.zip"
ditto -c -k --keepParent "$APP" "$ZIP"

echo "Отправка в Apple Notary Service: $ZIP"

if [[ -n "${APPLE_API_KEY_PATH:-}" && -n "${APPLE_API_KEY:-}" && -n "${APPLE_API_ISSUER:-}" ]]; then
  xcrun notarytool submit "$ZIP" \
    --key "$APPLE_API_KEY_PATH" \
    --key-id "$APPLE_API_KEY" \
    --issuer "$APPLE_API_ISSUER" \
    --wait
elif [[ -n "${APPLE_ID:-}" && -n "${APPLE_PASSWORD:-}" ]]; then
  TEAM=()
  [[ -n "${APPLE_TEAM_ID:-}" ]] && TEAM=(--team-id "$APPLE_TEAM_ID")
  xcrun notarytool submit "$ZIP" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_PASSWORD" \
    "${TEAM[@]}" \
    --wait
else
  echo "Задайте переменные API (APPLE_API_ISSUER, APPLE_API_KEY, APPLE_API_KEY_PATH) или Apple ID (APPLE_ID, APPLE_PASSWORD[, APPLE_TEAM_ID])." >&2
  exit 1
fi

echo "Stapler: $APP"
xcrun stapler staple "$APP"
echo "Готово: notarize + staple для $APP"

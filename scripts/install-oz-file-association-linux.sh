#!/usr/bin/env bash
set -euo pipefail

# Пользовательская ассоциация *.oz с приложением (без root): MIME + .desktop + xdg-mime.
#
# Установка:
#   ./scripts/install-oz-file-association-linux.sh --app /abs/path/to/omegazip-app
# Удаление:
#   ./scripts/install-oz-file-association-linux.sh --uninstall

APP_ARG=""
UNINSTALL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app)
      APP_ARG="${2:-}"
      shift 2
      ;;
    --uninstall)
      UNINSTALL=1
      shift
      ;;
    --help|-h)
      echo "Usage: $0 --app /path/to/gui-binary"
      echo "       $0 --uninstall"
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      exit 2
      ;;
  esac
done

DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
MIME_PKG_DIR="$DATA/mime/packages"
APPS_DIR="$DATA/applications"
MIME_XML="$MIME_PKG_DIR/omegazip-oz.xml"
DESKTOP="$APPS_DIR/omegazip-oz-open.desktop"
MIME_TYPE="application/x-omegazip"

if [[ "$UNINSTALL" == "1" ]]; then
  rm -f "$MIME_XML" "$DESKTOP"
  if command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database "$DATA/mime" 2>/dev/null || true
  fi
  echo "Removed user MIME package and desktop entry."
  exit 0
fi

if [[ -z "$APP_ARG" || ! -x "$APP_ARG" ]]; then
  echo "Pass --app /absolute/path/to/gui/binary (executable)." >&2
  exit 1
fi

if command -v readlink >/dev/null 2>&1 && readlink -f / >/dev/null 2>&1; then
  APP_ABS="$(readlink -f "$APP_ARG")"
else
  APP_ABS="$(cd "$(dirname "$APP_ARG")" && pwd)/$(basename "$APP_ARG")"
fi

mkdir -p "$MIME_PKG_DIR" "$APPS_DIR"

cat > "$MIME_XML" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="$MIME_TYPE">
    <glob pattern="*.oz"/>
    <comment>OmegaZip archive</comment>
  </mime-type>
</mime-info>
EOF

cat > "$DESKTOP" <<EOF
[Desktop Entry]
Type=Application
Name=OmegaZip
Exec="$APP_ABS" %F
MimeType=$MIME_TYPE;
NoDisplay=true
EOF

if command -v update-mime-database >/dev/null 2>&1; then
  update-mime-database "$DATA/mime" 2>/dev/null || true
fi

if command -v xdg-mime >/dev/null 2>&1; then
  xdg-mime default "$(basename "$DESKTOP")" "$MIME_TYPE" 2>/dev/null || \
    echo "Note: run: xdg-mime default $(basename "$DESKTOP") $MIME_TYPE" >&2
else
  echo "Install xdg-utils for xdg-mime, or set default app for $MIME_TYPE in your file manager." >&2
fi

echo "Installed: $MIME_XML + $DESKTOP"
echo "Default handler: $APP_ABS"

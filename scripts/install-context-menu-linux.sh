#!/usr/bin/env bash
set -euo pipefail

# Установка пользовательских пунктов контекстного меню OmegaZip в Linux без root:
# - Nautilus Scripts (GNOME Files)
# - KDE Service Menu (Dolphin/KIO)
#
# Usage:
#   ./scripts/install-context-menu-linux.sh [--binary /path/to/omegazip]
#   ./scripts/install-context-menu-linux.sh --uninstall

BIN_ARG=""
UNINSTALL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --binary)
      BIN_ARG="${2:-}"
      shift 2
      ;;
    --uninstall)
      UNINSTALL=1
      shift
      ;;
    *)
      echo "Unknown arg: $1" >&2
      echo "Usage: $0 [--binary /path/to/omegazip] [--uninstall]" >&2
      exit 2
      ;;
  esac
done

NAUTILUS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/nautilus/scripts"
KIO_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/kio/servicemenus"
KIO_FILE="$KIO_DIR/omegazip.desktop"

if [[ "$UNINSTALL" == "1" ]]; then
  rm -f "$NAUTILUS_DIR/OmegaZip Compress (Auto)" \
        "$NAUTILUS_DIR/OmegaZip Extract Here" \
        "$KIO_FILE"
  echo "Removed OmegaZip context entries from user profile."
  exit 0
fi

resolve_bin() {
  if [[ -n "$BIN_ARG" ]]; then
    printf '%s' "$BIN_ARG"
    return
  fi
  if command -v omegazip >/dev/null 2>&1; then
    command -v omegazip
    return
  fi
  if [[ -x "./target/release/omegazip" ]]; then
    printf '%s' "$(pwd)/target/release/omegazip"
    return
  fi
  if [[ -x "./target/debug/omegazip" ]]; then
    printf '%s' "$(pwd)/target/debug/omegazip"
    return
  fi
  echo ""
}

OZ="$(resolve_bin)"
if [[ -z "$OZ" || ! -x "$OZ" ]]; then
  echo "Cannot find executable omegazip. Pass --binary /absolute/path/to/omegazip" >&2
  exit 1
fi

mkdir -p "$NAUTILUS_DIR" "$KIO_DIR"

cat > "$NAUTILUS_DIR/OmegaZip Compress (Auto)" <<'EOS'
#!/usr/bin/env bash
set -euo pipefail

OZ_BIN="__OMEGAZIP_BIN__"

pick_ext_auto() {
  local f="$1"
  local ext="${f##*.}"
  ext="$(printf '%s' "$ext" | tr '[:upper:]' '[:lower:]')"
  case "$ext" in
    zip|7z|rar|gz|tgz|bz2|xz|zst|jpg|jpeg|png|gif|webp|mp3|mp4|mkv|mov|avi|pdf)
      printf '%s' "zip"
      ;;
    *)
      printf '%s' "oz"
      ;;
  esac
}

omegazip_stem() {
  local b
  b="$(basename "$1")"
  local l
  l="$(printf '%s' "$b" | tr '[:upper:]' '[:lower:]')"
  if [[ "$l" == *.tar.gz ]]; then
    printf '%s' "${b%.[Gg][Zz]}"
    return
  fi
  if [[ "$l" == *.tar.bz2 ]]; then
    printf '%s' "${b%.[Bb][Zz]2}"
    return
  fi
  if [[ "$l" == *.tar.xz ]]; then
    printf '%s' "${b%.[Xx][Zz]}"
    return
  fi
  printf '%s' "${b%.*}"
}

for src in "$@"; do
  [[ -e "$src" ]] || continue
  d="$(dirname "$src")"
  stem="$(omegazip_stem "$src")"
  [[ -z "$stem" ]] && stem="archive"
  ext="$(pick_ext_auto "$src")"
  out="$d/$stem.$ext"
  if [[ "$ext" == "oz" ]]; then
    "$OZ_BIN" compress --preset auto "$src" "$out"
  else
    "$OZ_BIN" compress "$src" "$out"
  fi
done
EOS

cat > "$NAUTILUS_DIR/OmegaZip Extract Here" <<'EOS'
#!/usr/bin/env bash
set -euo pipefail
OZ_BIN="__OMEGAZIP_BIN__"
for src in "$@"; do
  [[ -e "$src" ]] || continue
  d="$(dirname "$src")"
  b="$(basename "$src")"
  stem="${b%.*}"
  out="$d/${stem}_распаковано"
  "$OZ_BIN" decompress "$src" "$out"
done
EOS

chmod +x "$NAUTILUS_DIR/OmegaZip Compress (Auto)" "$NAUTILUS_DIR/OmegaZip Extract Here"

OZ_ESCAPED="$(printf '%s\n' "$OZ" | sed 's/[\/&]/\\&/g')"
sed -i "s/__OMEGAZIP_BIN__/$OZ_ESCAPED/g" "$NAUTILUS_DIR/OmegaZip Compress (Auto)" "$NAUTILUS_DIR/OmegaZip Extract Here"

cat > "$KIO_FILE" <<EOF
[Desktop Entry]
Type=Service
X-KDE-ServiceTypes=KonqPopupMenu/Plugin
MimeType=all/allfiles;inode/directory;
Actions=OmegaZipCompressAuto;OmegaZipExtractHere;
X-KDE-Submenu=OmegaZip
Icon=application-x-archive

[Desktop Action OmegaZipCompressAuto]
Name=Compress (auto .oz/.zip)
Icon=folder-compress
Exec=sh -c '"$NAUTILUS_DIR/OmegaZip Compress (Auto)" %F'

[Desktop Action OmegaZipExtractHere]
Name=Extract Here
Icon=archive-extract
Exec=sh -c '"$NAUTILUS_DIR/OmegaZip Extract Here" %F'
EOF

echo "Installed:"
echo "  Nautilus scripts:"
echo "    $NAUTILUS_DIR/OmegaZip Compress (Auto)"
echo "    $NAUTILUS_DIR/OmegaZip Extract Here"
echo "  KDE service menu:"
echo "    $KIO_FILE"
echo ""
echo "If menu is not visible immediately, restart file manager."

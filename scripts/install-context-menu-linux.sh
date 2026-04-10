#!/usr/bin/env bash
set -euo pipefail

# Установка пользовательских пунктов контекстного меню OmegaZip в Linux без root:
# - Nautilus Scripts (GNOME Files)
# - KDE Service Menu (Dolphin/KIO)
#
# Логика pick_ext_auto / stem / пресеты — в паритете с scripts/install-context-menu.sh (macOS).
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
    --help|-h)
      echo "Usage: $0 [--binary /abs/path/to/omegazip] [--uninstall]"
      exit 0
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

# Синхронизировать с scripts/install-context-menu.sh (pick_ext_auto) и omega-context-helper.ps1 ($ZipLikeExt)
pick_ext_auto() {
  local f="$1" base ext
  if [[ -d "$f" ]]; then
    printf '%s' "oz"
    return 0
  fi
  base="${f##*/}"
  ext="${base##*.}"
  ext="$(printf '%s' "$ext" | tr '[:upper:]' '[:lower:]')"
  if [[ "$ext" =~ ^(zip|7z|rar|tar|gz|tgz|bz2|xz|zst|lz4|lzma|cab|ar|cpio|xpi|crx|jar|war|ear|apk|ipa|msix|jpg|jpeg|pjpeg|png|gif|bmp|webp|tif|tiff|heic|heif|avif|ico|jxl|psd|dds|exr|dng|cr2|nef|orf|srw|svgz|mp4|m4v|mkv|avi|mov|webm|mpeg|mpg|m2v|wmv|flv|3gp|ogv|ts|mts|m2ts|vob|asf|f4v|mp3|flac|wav|aac|m4a|m4b|ogg|opus|wma|aiff|aif|mpc|wv|ape|caf|woff|woff2|otf|ttf|eot|exe|dll|dylib|so|bin|com|msi|pyc|pyo|o|a|lib|class|dex|pak|nib|wasm|pdb|dmg|iso|img|vmdk|vdi|qcow2|hdd|vhd|sparseimage|sqlite|db-shm|db-wal)$ ]]; then
    printf '%s' "zip"
    return 0
  fi
  printf '%s' "oz"
}

omegazip_stem() {
  local f="$1" seg l n
  seg="${f##*/}"
  seg="${seg%/}"
  if [[ -d "$f" ]]; then
    printf '%s' "$seg"
    return 0
  fi
  l="$(printf '%s' "$seg" | tr '[:upper:]' '[:lower:]')"
  case "$l" in
    *.tar.gz)  n=${#seg}; printf '%s' "${seg:0:$((n - 7))}"; return ;;
    *.tar.bz2) n=${#seg}; printf '%s' "${seg:0:$((n - 8))}"; return ;;
    *.tar.xz)  n=${#seg}; printf '%s' "${seg:0:$((n - 7))}"; return ;;
    *.tar.zst) n=${#seg}; printf '%s' "${seg:0:$((n - 8))}"; return ;;
    *.tgz)     n=${#seg}; printf '%s' "${seg:0:$((n - 4))}"; return ;;
    *.tbz2)    n=${#seg}; printf '%s' "${seg:0:$((n - 5))}"; return ;;
    *.txz)     n=${#seg}; printf '%s' "${seg:0:$((n - 4))}"; return ;;
    *.tzst)    n=${#seg}; printf '%s' "${seg:0:$((n - 5))}"; return ;;
  esac
  if [[ "$seg" == .* ]]; then
    printf '%s' "$seg"
  elif [[ "$seg" == *.* ]]; then
    printf '%s' "${seg%.*}"
  else
    printf '%s' "$seg"
  fi
}

load_oz_context_preset() {
  if [[ -z "${OMEGAZIP_CONTEXT_PRESET:-}" ]]; then
    local cf="${XDG_CONFIG_HOME:-$HOME/.config}/omegazip/context_preset"
    if [[ -f "$cf" ]]; then
      OMEGAZIP_CONTEXT_PRESET="$(grep -v '^[[:space:]]*#' "$cf" | grep -v '^[[:space:]]*$' | head -1 | tr -d '\r\n' | awk '{print $1}')"
    fi
  fi
  if [[ -z "${OMEGAZIP_CONTEXT_PRESET:-}" ]]; then
    OMEGAZIP_CONTEXT_PRESET="auto"
  fi
}

read_auto_upgrade_mb() {
  if [[ -n "${OMEGAZIP_AUTO_UPGRADE_FOLDER_MB:-}" ]]; then
    printf '%s' "${OMEGAZIP_AUTO_UPGRADE_FOLDER_MB}"
    return
  fi
  local cf="${XDG_CONFIG_HOME:-$HOME/.config}/omegazip/auto_upgrade_folder_mb"
  if [[ -f "$cf" ]]; then
    local v
    v="$(grep -v '^[[:space:]]*#' "$cf" | grep -v '^[[:space:]]*$' | head -1 | tr -d '\r\n' | awk '{print $1}')"
    if [[ "$v" =~ ^[0-9]+$ ]]; then
      printf '%s' "$v"
      return
    fi
  fi
  printf '%s' "200"
}

bump_preset_for_large_folder() {
  local path="$1" base="$2" mb kb
  mb="$(read_auto_upgrade_mb)"
  if [[ "$mb" == "0" ]]; then
    printf '%s' "$base"
    return
  fi
  if [[ "$base" != "auto" || ! -d "$path" ]]; then
    printf '%s' "$base"
    return
  fi
  kb="$(du -sk "$path" 2>/dev/null | awk '{print $1}')"
  if [[ -z "$kb" || ! "$kb" =~ ^[0-9]+$ ]]; then
    printf '%s' "$base"
    return
  fi
  if [[ "$kb" -ge $((mb * 1024)) ]]; then
    printf '%s' "max"
    return
  fi
  printf '%s' "$base"
}

run_compress_oz() {
  local src="$1" out="$2" eff="$3"
  case "$eff" in
    max|aggressive) "$OZ_BIN" compress --preset max "$src" "$out" ;;
    ultra)          "$OZ_BIN" compress --preset ultra "$src" "$out" ;;
    fast)           "$OZ_BIN" compress --preset fast "$src" "$out" ;;
    balanced)       "$OZ_BIN" compress --preset balanced "$src" "$out" ;;
    *)              "$OZ_BIN" compress --preset auto "$src" "$out" ;;
  esac
}

for src in "$@"; do
  [[ -e "$src" ]] || continue
  d="$(dirname "$src")"
  stem="$(omegazip_stem "$src")"
  [[ -z "$stem" ]] && stem="archive"
  ext="$(pick_ext_auto "$src")"
  out="$d/$stem.$ext"
  if [[ "$ext" == "oz" ]]; then
    load_oz_context_preset
    eff="$(bump_preset_for_large_folder "$src" "${OMEGAZIP_CONTEXT_PRESET}")"
    run_compress_oz "$src" "$out" "$eff"
  else
    "$OZ_BIN" compress "$src" "$out"
  fi
done
EOS

cat > "$NAUTILUS_DIR/OmegaZip Extract Here" <<'EOS'
#!/usr/bin/env bash
set -euo pipefail
OZ_BIN="__OMEGAZIP_BIN__"

omegazip_stem() {
  local f="$1" seg l n
  seg="${f##*/}"
  seg="${seg%/}"
  if [[ -d "$f" ]]; then
    printf '%s' "$seg"
    return 0
  fi
  l="$(printf '%s' "$seg" | tr '[:upper:]' '[:lower:]')"
  case "$l" in
    *.tar.gz)  n=${#seg}; printf '%s' "${seg:0:$((n - 7))}"; return ;;
    *.tar.bz2) n=${#seg}; printf '%s' "${seg:0:$((n - 8))}"; return ;;
    *.tar.xz)  n=${#seg}; printf '%s' "${seg:0:$((n - 7))}"; return ;;
    *.tar.zst) n=${#seg}; printf '%s' "${seg:0:$((n - 8))}"; return ;;
    *.tgz)     n=${#seg}; printf '%s' "${seg:0:$((n - 4))}"; return ;;
    *.tbz2)    n=${#seg}; printf '%s' "${seg:0:$((n - 5))}"; return ;;
    *.txz)     n=${#seg}; printf '%s' "${seg:0:$((n - 4))}"; return ;;
    *.tzst)    n=${#seg}; printf '%s' "${seg:0:$((n - 5))}"; return ;;
  esac
  if [[ "$seg" == .* ]]; then
    printf '%s' "$seg"
  elif [[ "$seg" == *.* ]]; then
    printf '%s' "${seg%.*}"
  else
    printf '%s' "$seg"
  fi
}

for src in "$@"; do
  [[ -e "$src" ]] || continue
  [[ -d "$src" ]] && continue
  d="$(dirname "$src")"
  stem="$(omegazip_stem "$src")"
  [[ -z "$stem" ]] && stem="archive"
  out="$d/${stem}_распаковано"
  "$OZ_BIN" decompress "$src" "$out"
done
EOS

chmod +x "$NAUTILUS_DIR/OmegaZip Compress (Auto)" "$NAUTILUS_DIR/OmegaZip Extract Here"

OZ_ESCAPED="$(printf '%s\n' "$OZ" | sed 's/[\/&]/\\&/g')"
sed -i "s/__OMEGAZIP_BIN__/$OZ_ESCAPED/g" "$NAUTILUS_DIR/OmegaZip Compress (Auto)" "$NAUTILUS_DIR/OmegaZip Extract Here"

# shellcheck disable=SC2016
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

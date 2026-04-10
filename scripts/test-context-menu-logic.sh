#!/usr/bin/env bash
# Автотесты логики ПКМ (bash): pick_ext_auto, omegazip_stem.
# Должны совпадать с телом скриптов в install-context-menu-linux.sh (и по смыслу с omega-context-helper.ps1).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT/.."

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "ok: $*"; }

# --- копия логики из install-context-menu-linux.sh (Compress Auto) ---
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

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/subdir"
[[ "$(pick_ext_auto "$TMP/subdir")" == "oz" ]] || fail "directory -> oz"
pass "pick_ext_auto directory"

touch "$TMP/a.jpg"
[[ "$(pick_ext_auto "$TMP/a.jpg")" == "zip" ]] || fail "jpg -> zip"
pass "pick_ext_auto jpg"

touch "$TMP/readme.txt"
[[ "$(pick_ext_auto "$TMP/readme.txt")" == "oz" ]] || fail "txt -> oz"
pass "pick_ext_auto txt"

touch "$TMP/archive.tar.gz"
[[ "$(omegazip_stem "$TMP/archive.tar.gz")" == "archive" ]] || fail "stem tar.gz"
pass "stem .tar.gz"

touch "$TMP/x.TGZ"
[[ "$(omegazip_stem "$TMP/x.TGZ")" == "x" ]] || fail "stem .TGZ"
pass "stem .tgz case"

touch "$TMP/plain.oz"
[[ "$(omegazip_stem "$TMP/plain.oz")" == "plain" ]] || fail "stem .oz"
pass "stem simple ext"

mkdir -p "$TMP/foldername"
[[ "$(omegazip_stem "$TMP/foldername")" == "foldername" ]] || fail "stem dir"
pass "stem directory"

echo ""
echo "All context-menu logic tests passed."

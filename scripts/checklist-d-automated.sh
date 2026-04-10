#!/usr/bin/env bash
# Автоматизируемые пункты раздела D (MEASURABLE-QUALITY): CLI + наличие 7z.
# D2/D3/D5/D7/D8 (часть) + symlink: `cargo test -p omegazip --test archive_hardening`.
# D6/D9 (ZIP): `cargo test -p omegazip --test compat_roundtrip`.
# Вручную при необходимости: пути около лимита ОС (PATH_MAX), экзотические ZIP.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/release/omegazip}"
FAIL=0

if [[ ! -x "$BIN" ]]; then
  echo "D-*: нет $BIN — cargo build --release -p omegazip"
  exit 1
fi

echo "=== D1: распаковка нормального ZIP из manual-files ==="
ZIP="$ROOT/tests/manual-files/downloads/archive_zip_hellogitworld.zip"
OUT="$ROOT/target/d-check-d1"
rm -rf "$OUT"
mkdir -p "$OUT"
if "$BIN" decompress "$ZIP" "$OUT" >/tmp/d1.log 2>&1; then
  N="$(find "$OUT" -type f | wc -l | tr -d ' ')"
  echo "D1 OK (файлов извлечено: $N)"
else
  echo "D1 FAIL"
  cat /tmp/d1.log
  FAIL=1
fi

echo "=== D4: не-архив (1 байт) → ожидаем ошибку ==="
echo X >"$ROOT/target/d4-garbage.bin"
if "$BIN" decompress "$ROOT/target/d4-garbage.bin" "$ROOT/target/d4-out" >/tmp/d4.log 2>&1; then
  echo "D4 FAIL (ожидалась ошибка)"
  FAIL=1
else
  echo "D4 OK (вернулась ошибка)"
fi

echo "=== D4b: пустой файл → ожидаем ошибку ==="
: >"$ROOT/target/d4b-empty"
if "$BIN" decompress "$ROOT/target/d4b-empty" "$ROOT/target/d4b-out" >/tmp/d4b.log 2>&1; then
  echo "D4b FAIL (ожидалась ошибка)"
  FAIL=1
else
  echo "D4b OK"
fi

echo "=== D5: .oz с паролем — неверный пароль (CLI) ==="
D5="$ROOT/target/d5-pw"
rm -rf "$D5"
mkdir -p "$D5/in" "$D5/out"
echo "payload" >"$D5/in/x.txt"
"$BIN" compress --password "bench-real" "$D5/in/x.txt" "$D5/enc.oz"
if "$BIN" decompress --password "bench-wrong" "$D5/enc.oz" "$D5/out" >/tmp/d5.log 2>&1; then
  echo "D5 FAIL (неверный пароль не должен распаковывать)"
  FAIL=1
else
  echo "D5 OK"
fi

echo "=== D10: 7-Zip в PATH ==="
SEVEN_ZIP_BIN=""
if command -v 7z >/dev/null 2>&1; then
  SEVEN_ZIP_BIN="$(command -v 7z)"
elif command -v 7zz >/dev/null 2>&1; then
  SEVEN_ZIP_BIN="$(command -v 7zz)"
elif [[ -x "/opt/homebrew/opt/sevenzip/bin/7zz" ]]; then
  SEVEN_ZIP_BIN="/opt/homebrew/opt/sevenzip/bin/7zz"
elif [[ -x "/usr/local/opt/sevenzip/bin/7zz" ]]; then
  SEVEN_ZIP_BIN="/usr/local/opt/sevenzip/bin/7zz"
fi

if [[ -n "$SEVEN_ZIP_BIN" ]]; then
  echo "D10: найден $SEVEN_ZIP_BIN — делегирование возможно"
  echo "=== D10b: smoke .7z roundtrip через делегирование ==="
  D10="$ROOT/target/d10-7z"
  rm -rf "$D10"
  mkdir -p "$D10/in" "$D10/out"
  echo "7z-smoke" >"$D10/in/a.txt"
  if "$BIN" compress "$D10/in/a.txt" "$D10/p.7z" >/tmp/d10c.log 2>&1 && \
     "$BIN" decompress "$D10/p.7z" "$D10/out" >/tmp/d10d.log 2>&1 && \
     [[ "$(cat "$D10/out/a.txt" 2>/dev/null || true)" == "7z-smoke" ]]; then
    echo "D10b OK (.7z compress+decompress)"
  else
    echo "D10b FAIL (.7z smoke)"
    [[ -f /tmp/d10c.log ]] && cat /tmp/d10c.log
    [[ -f /tmp/d10d.log ]] && cat /tmp/d10d.log
    FAIL=1
  fi
else
  echo "D10: 7z не в PATH — RAR/7z только при установке (не ошибка прогона)"
fi

exit "$FAIL"

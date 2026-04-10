#!/usr/bin/env bash
# Измеряемый контраст: обычный ZIP vs .oz с --chunked на корпусе с массовыми дубликатами.
# Сильная сторона OmegaZip — дедуп по чанкам; ZIP сжимает каждый файл отдельно.
#
# Использование:
#   bash scripts/measure-oz-advantage.sh
#   CORPUS_DIR=/path/to/tree bash scripts/measure-oz-advantage.sh
#
# Требуется: python3, собранный omegazip (release).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIN="${BIN:-$ROOT/target/release/omegazip}"
if [[ ! -x "$BIN" ]]; then
  echo "Сборка omegazip (release)..."
  cargo build --release -p omegazip -q
  BIN="$ROOT/target/release/omegazip"
fi

WORK="$ROOT/target/measure_oz_advantage"
rm -rf "$WORK"
mkdir -p "$WORK"

if [[ -n "${CORPUS_DIR:-}" ]]; then
  CORPUS="$CORPUS_DIR"
  if [[ ! -d "$CORPUS" ]]; then
    echo "CORPUS_DIR не каталог: $CORPUS" >&2
    exit 1
  fi
  echo "Корпус (внешний): $CORPUS"
else
  CORPUS="$WORK/corpus"
  mkdir -p "$CORPUS"
  echo "Генерация синтетического корпуса (100 одинаковых «тяжёлых» файлов)..."
  export CORPUS
  python3 <<'PY'
import os
corpus = os.environ["CORPUS"]
payload = bytes(range(256)) * 180 + b"DEDUP_PATTERN" * 800
for i in range(100):
    with open(os.path.join(corpus, f"f_{i:04d}.bin"), "wb") as f:
        f.write(payload)
PY
fi

ZIP_OUT="$WORK/out.zip"
OZ_OUT="$WORK/out.oz"
rm -f "$ZIP_OUT" "$OZ_OUT"

measure_one() {
  local label="$1"
  shift
  python3 -c "
import subprocess, sys, time, os
label = sys.argv[1]
args = sys.argv[2:]
t0 = time.perf_counter()
r = subprocess.run(args)
dt = time.perf_counter() - t0
if r.returncode != 0:
    sys.exit(r.returncode)
path = args[-1]
sz = os.path.getsize(path)
print(f'{label}: bytes={sz} time_s={dt:.4f}')
" "$label" "$@"
}

echo "=== ZIP (без chunked) ==="
measure_one zip "$BIN" compress "$CORPUS" "$ZIP_OUT"
echo "=== .oz --chunked --preset balanced ==="
measure_one oz "$BIN" compress --chunked --preset balanced "$CORPUS" "$OZ_OUT"

python3 <<PY
import os
z = os.path.getsize("$ZIP_OUT")
o = os.path.getsize("$OZ_OUT")
if z <= 0:
    raise SystemExit("bad zip size")
print()
print("--- Сводка (.oz vs ZIP на этом корпусе) ---")
print(f"  zip_bytes: {z}")
print(f"  oz_bytes:  {o}")
if o < z:
    win = (z - o) / z * 100.0
    print(f"  .oz меньше ZIP на {win:.1f}% (от размера ZIP)")
else:
    print(f"  на этом корпусе .oz не меньше ZIP (см. типы данных и пресеты)")
PY

echo ""
echo "Подсказка: на реальных дубликатах: CORPUS_DIR=/путь/к/дереву bash scripts/measure-oz-advantage.sh"

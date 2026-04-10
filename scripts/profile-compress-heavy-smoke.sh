#!/usr/bin/env bash
# C2: дымовой «тяжёлый» прогон сжатия (~30 MiB, сильный дедуп) — время и размеры.
# Результат: tests/manual-files/results-auto/profile-smoke-last.txt
#
# Использование: bash scripts/profile-compress-heavy-smoke.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIN="${BIN:-$ROOT/target/release/omegazip}"
if [[ ! -x "$BIN" ]]; then
  cargo build --release -p omegazip -q
  BIN="$ROOT/target/release/omegazip"
fi

CORPUS="$ROOT/target/profile_heavy_corpus"
OUT_DIR="$ROOT/target/profile_heavy_out"
REPORT="$ROOT/tests/manual-files/results-auto/profile-smoke-last.txt"
mkdir -p "$(dirname "$REPORT")"
rm -rf "$CORPUS" "$OUT_DIR"
mkdir -p "$CORPUS"

echo "Генерация корпуса (~30 MiB, 60× одинаковый блок 512 KiB)..."
export CORPUS
python3 <<'PY'
import os
corpus = os.environ["CORPUS"]
block = bytes(range(256)) * 2000
assert len(block) == 512000
for i in range(60):
    with open(os.path.join(corpus, f"b_{i:04d}.dat"), "wb") as f:
        f.write(block)
PY

BYTES="$(python3 -c "import pathlib; p=pathlib.Path('$CORPUS'); print(sum(f.stat().st_size for f in p.iterdir() if f.is_file()))")"

python3 <<PY >"$REPORT"
import os, subprocess, time, pathlib, datetime

def run(args):
    t0 = time.perf_counter()
    r = subprocess.run(
        args,
        cwd="$ROOT",
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    dt = time.perf_counter() - t0
    assert r.returncode == 0, args
    return dt

bin_ = "$BIN"
corpus = "$CORPUS"
out_dir = pathlib.Path("$OUT_DIR")
out_dir.mkdir(parents=True, exist_ok=True)

lines = []
lines.append(f"OmegaZip profile-smoke (python timer) {datetime.datetime.now().isoformat(timespec='seconds')}")
lines.append(f"input_bytes: $BYTES")
lines.append("")

t_zip = run([bin_, "compress", corpus, str(out_dir / "heavy.zip")])
sz_zip = (out_dir / "heavy.zip").stat().st_size
lines.append(f"zip_time_s: {t_zip:.4f}")
lines.append(f"zip_bytes: {sz_zip}")

t_oz = run([bin_, "compress", "--chunked", "--preset", "balanced", corpus, str(out_dir / "heavy.oz")])
sz_oz = (out_dir / "heavy.oz").stat().st_size
lines.append(f"oz_chunked_balanced_time_s: {t_oz:.4f}")
lines.append(f"oz_bytes: {sz_oz}")

if sz_zip > 0:
    lines.append(f"oz_vs_zip_size_pct: {(1 - sz_oz / sz_zip) * 100:.1f}% smaller than zip")

print("\n".join(lines))
PY

echo ""
echo "Отчёт: $REPORT"
cat "$REPORT"

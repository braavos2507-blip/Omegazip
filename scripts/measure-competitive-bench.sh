#!/usr/bin/env bash
# Сравнение ZIP vs .oz vs 7z на двух реальных корпусах.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$ROOT/tests/manual-files/results-auto"
OUT_MD="$OUT_DIR/OZ-ZIP-7Z-LATEST.md"
mkdir -p "$OUT_DIR"

BIN="${OMEGAZIP_BIN:-$ROOT/target/release/omegazip}"
if [[ ! -x "$BIN" ]]; then
  echo "Нет бинарника: $BIN. Выполните: cargo build --release -p omegazip" >&2
  exit 1
fi

SEVEN_BIN="$(command -v 7zz || command -v 7z || true)"
if [[ -z "$SEVEN_BIN" ]]; then
  for p in /opt/homebrew/opt/sevenzip/bin/7zz /usr/local/opt/sevenzip/bin/7zz; do
    if [[ -x "$p" ]]; then
      SEVEN_BIN="$p"
      break
    fi
  done
fi
if [[ -z "$SEVEN_BIN" ]]; then
  echo "7z/7zz не найден в PATH" >&2
  exit 1
fi

python3 - <<'PY'
import json
import os
import pathlib
import shutil
import subprocess
import tempfile
import time

root = pathlib.Path("/Users/renat/01Project/OmegaZip")
out_md = root / "tests/manual-files/results-auto/OZ-ZIP-7Z-LATEST.md"
bin_oz = os.environ.get("OMEGAZIP_BIN", str(root / "target/release/omegazip"))
seven = shutil.which("7zz") or shutil.which("7z") or "/opt/homebrew/opt/sevenzip/bin/7zz"

corpora = [
    ("versioned", root / "Архивы/github-versions"),
    ("mixed", root / "Архивы/mixed-files"),
]

def file_count(path: pathlib.Path) -> int:
    return sum(1 for p in path.rglob("*") if p.is_file())

def total_bytes(path: pathlib.Path) -> int:
    return sum(p.stat().st_size for p in path.rglob("*") if p.is_file())

def run_timed(cmd, cwd=None):
    t0 = time.perf_counter()
    proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    dt = time.perf_counter() - t0
    if proc.returncode != 0:
        raise RuntimeError(f"Command failed: {' '.join(cmd)}\n{proc.stdout}\n{proc.stderr}")
    return dt

def run_timed_soft(cmd, cwd=None):
    t0 = time.perf_counter()
    proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    dt = time.perf_counter() - t0
    ok = proc.returncode == 0
    msg = ""
    if not ok:
        msg = (proc.stderr or proc.stdout or "").strip().splitlines()[-1:] or [f"exit {proc.returncode}"]
        msg = msg[0]
    return dt, ok, msg

rows = []
with tempfile.TemporaryDirectory(prefix="oz-zip-7z-") as td:
    td = pathlib.Path(td)
    for name, corpus in corpora:
        if not corpus.exists():
            continue
        files = file_count(corpus)
        if files == 0:
            continue
        in_bytes = total_bytes(corpus)
        zip_out = td / f"{name}.zip"
        oz_out = td / f"{name}.oz"
        seven_out = td / f"{name}.7z"
        if zip_out.exists():
            zip_out.unlink()
        if oz_out.exists():
            oz_out.unlink()
        if seven_out.exists():
            seven_out.unlink()

        t_zip = run_timed([bin_oz, "compress", str(corpus), str(zip_out)])
        t_oz = run_timed([bin_oz, "compress", "--preset", "competitive", str(corpus), str(oz_out)])
        t_7z = run_timed([seven, "a", "-t7z", "-mx=5", "-y", str(seven_out), "."], cwd=str(corpus))

        ext_zip = td / f"{name}_zip_ext"
        ext_oz = td / f"{name}_oz_ext"
        ext_7z = td / f"{name}_7z_ext"
        for ext in (ext_zip, ext_oz, ext_7z):
            if ext.exists():
                shutil.rmtree(ext)
            ext.mkdir(parents=True, exist_ok=True)
        t_d_zip, ok_d_zip, err_d_zip = run_timed_soft([bin_oz, "decompress", str(zip_out), str(ext_zip)])
        t_d_oz, ok_d_oz, err_d_oz = run_timed_soft([bin_oz, "decompress", str(oz_out), str(ext_oz)])
        t_d_7z, ok_d_7z, err_d_7z = run_timed_soft([bin_oz, "decompress", str(seven_out), str(ext_7z)])

        b_zip = zip_out.stat().st_size
        b_oz = oz_out.stat().st_size
        b_7z = seven_out.stat().st_size
        oz_vs_zip = (1.0 - (b_oz / b_zip)) * 100.0
        oz_vs_7z = (1.0 - (b_oz / b_7z)) * 100.0

        rows.append({
            "name": name,
            "files": files,
            "in_mb": in_bytes / (1024 * 1024),
            "zip_mb": b_zip / (1024 * 1024),
            "oz_mb": b_oz / (1024 * 1024),
            "seven_mb": b_7z / (1024 * 1024),
            "oz_vs_zip": oz_vs_zip,
            "oz_vs_7z": oz_vs_7z,
            "t_zip": t_zip,
            "t_oz": t_oz,
            "t_7z": t_7z,
            "t_d_zip": t_d_zip,
            "t_d_oz": t_d_oz,
            "t_d_7z": t_d_7z,
            "ok_d_zip": ok_d_zip,
            "ok_d_oz": ok_d_oz,
            "ok_d_7z": ok_d_7z,
            "err_d_zip": err_d_zip,
            "err_d_oz": err_d_oz,
            "err_d_7z": err_d_7z,
        })

lines = []
lines.append("# Competitive bench: ZIP vs .oz vs 7z")
lines.append("")
lines.append(f"- Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}")
lines.append(f"- OmegaZip binary: `{bin_oz}`")
lines.append(f"- 7z binary: `{seven}`")
lines.append("")
lines.append("| Corpus | Files | Input MB | ZIP MB | .oz MB | 7z MB | .oz vs ZIP | .oz vs 7z | ZIP c s | .oz c s | 7z c s | ZIP d s | .oz d s | 7z d s |")
lines.append("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
for r in rows:
    d_zip = f"{r['t_d_zip']:.3f}" if r["ok_d_zip"] else "fail"
    d_oz = f"{r['t_d_oz']:.3f}" if r["ok_d_oz"] else "fail"
    d_7z = f"{r['t_d_7z']:.3f}" if r["ok_d_7z"] else "fail"
    lines.append(
        f"| {r['name']} | {r['files']} | {r['in_mb']:.2f} | {r['zip_mb']:.2f} | {r['oz_mb']:.2f} | {r['seven_mb']:.2f} | "
        f"{r['oz_vs_zip']:+.1f}% | {r['oz_vs_7z']:+.1f}% | {r['t_zip']:.3f} | {r['t_oz']:.3f} | {r['t_7z']:.3f} | "
        f"{d_zip} | {d_oz} | {d_7z} |"
    )
    if not r["ok_d_zip"]:
        lines.append(f"  - note({r['name']} zip d): {r['err_d_zip']}")
    if not r["ok_d_oz"]:
        lines.append(f"  - note({r['name']} oz d): {r['err_d_oz']}")
    if not r["ok_d_7z"]:
        lines.append(f"  - note({r['name']} 7z d): {r['err_d_7z']}")
lines.append("")
lines.append("`+.oz vs ZIP` означает, что .oz меньше. Отрицательное значение — .oz больше.")
lines.append("`c` = compress, `d` = decompress.")

out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"Written: {out_md}")
PY


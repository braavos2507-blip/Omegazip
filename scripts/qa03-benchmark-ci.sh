#!/usr/bin/env bash
# QA-03: компактный регрессионный прогон сжатие/распаковка для CI (Linux + macOS).
# Проверяет успешный roundtrip и идентичность данных (diff), без порогов по времени
# (время нестабильно между раннерами).
#
# Использование: из корня репозитория — bash scripts/qa03-benchmark-ci.sh
# Переменная BIN: путь к omegazip (по умолчанию target/release/omegazip).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIN="${BIN:-$ROOT/target/release/omegazip}"
if [[ ! -x "$BIN" ]]; then
  echo "Building omegazip (release)..."
  cargo build --release -p omegazip -q
  BIN="$ROOT/target/release/omegazip"
fi

WORK="$ROOT/target/qa03_ci_work"
rm -rf "$WORK"
mkdir -p "$WORK/in" "$WORK/out"

echo "QA-03 CI benchmark — binary: $BIN"

prepare_corpus() {
  # Текст, хорошо сжимается → .oz + --preset auto
  python3 -c "print('omega zip bench line\\n' * 5000)" >"$WORK/in/text.txt"
  python3 -c "import json; print(json.dumps([{'id': i, 'tag': 'x'} for i in range(3000)]))" >"$WORK/in/data.json"
  mkdir -p "$WORK/in/mixed"
  for i in 1 2 3 4 5 6 7 8; do
    printf 'payload %d abcdefghijklmnop\n' "$i" >"$WORK/in/mixed/f_${i}.txt"
  done
  # Бинарник → по логике контекстного меню уйдёт в .zip (без preset auto для zip)
  python3 -c "import random; random.seed(42); open('$WORK/in/r.bin','wb').write(bytes(random.randint(0,255) for _ in range(12000)))"
}

verify_tree_equal() {
  local left="$1"
  local right="$2"
  if ! diff -qr "$left" "$right" >&2; then
    echo "QA-03 FAIL: contents differ: $left vs $right" >&2
    exit 1
  fi
}

run_oz_roundtrip() {
  local name="$1"
  local inp="$2"
  local arc="$WORK/out/${name}.oz"
  local ext="$WORK/out/${name}_ext"
  rm -rf "$arc" "$ext"
  mkdir -p "$ext"
  echo "  [oz] $name <- $inp"
  "$BIN" compress --preset auto "$inp" "$arc"
  "$BIN" decompress "$arc" "$ext"
  if [[ -f "$inp" ]]; then
    local base
    base="$(basename "$inp")"
    if [[ ! -f "$ext/$base" ]]; then
      echo "QA-03 FAIL: expected $ext/$base" >&2
      exit 1
    fi
    cmp -s "$inp" "$ext/$base" || {
      echo "QA-03 FAIL: $name bytes differ" >&2
      exit 1
    }
  else
    # каталог: содержимое архива распаковывается в ext без обёртки имени каталога
    verify_tree_equal "$inp" "$ext"
  fi
}

run_zip_roundtrip() {
  local name="$1"
  local inp="$2"
  local arc="$WORK/out/${name}.zip"
  local ext="$WORK/out/${name}_ext"
  rm -rf "$arc" "$ext"
  mkdir -p "$ext"
  echo "  [zip] $name <- $inp"
  "$BIN" compress "$inp" "$arc"
  "$BIN" decompress "$arc" "$ext"
  local base
  base="$(basename "$inp")"
  [[ -f "$ext/$base" ]] || {
    echo "QA-03 FAIL: expected $ext/$base" >&2
    exit 1
  }
  cmp -s "$inp" "$ext/$base" || {
    echo "QA-03 FAIL: $name bytes differ" >&2
    exit 1
  }
}

prepare_corpus

echo "Running roundtrip cases..."
run_oz_roundtrip "t_text" "$WORK/in/text.txt"
run_oz_roundtrip "t_json" "$WORK/in/data.json"
run_oz_roundtrip "t_mixed" "$WORK/in/mixed"
run_zip_roundtrip "t_bin" "$WORK/in/r.bin"

echo "QA-03 CI benchmark OK ($(uname -s))"

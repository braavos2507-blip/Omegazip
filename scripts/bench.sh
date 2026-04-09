#!/usr/bin/env bash
# OmegaZip benchmark: создаёт тестовые файлы, гоняет пресеты, выводит таблицу скорости и сжатия.
# Запуск: из корня репо — ./scripts/bench.sh   или   bash scripts/bench.sh

set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# Бинарник: либо явный BIN, либо target/release/omegazip, либо через cargo run
BIN="${BIN:-}"
if [[ -z "$BIN" && -x "$ROOT/target/release/omegazip" ]]; then
  BIN="$ROOT/target/release/omegazip"
fi
if [[ -z "$BIN" ]]; then
  echo "Building omegazip (release)..."
  cargo build --release -p omegazip
  BIN="$ROOT/target/release/omegazip"
fi
# Если бинарника нет (например, workspace target в другом месте) — запускаем через cargo
RUN_CMD=()
if [[ -x "$BIN" ]]; then
  RUN_CMD=("$BIN")
else
  RUN_CMD=(cargo run --release -p omegazip --quiet --)
  echo "Using: cargo run --release -p omegazip -- ..."
fi
BENCH_DIR="$ROOT/target/bench_suite"
RESULTS_DIR="$ROOT/target/bench_results"
ARCHIVE_DIR="$ROOT/target/bench_extract"

# Тестовый набор: текст, JSON, бинарь, смешанная папка
prepare_suite() {
  rm -rf "$BENCH_DIR" "$RESULTS_DIR" "$ARCHIVE_DIR"
  mkdir -p "$BENCH_DIR"
  # Текст ~500 KB
  for i in {1..200}; do
    echo "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat." >> "$BENCH_DIR/lorem.txt"
  done
  # JSON
  echo '{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}' > "$BENCH_DIR/small.json"
  for i in {1..800}; do cat "$BENCH_DIR/small.json" >> "$BENCH_DIR/medium.json"; done
  # Бинарь (псевдо-случайный)
  dd if=/dev/urandom of="$BENCH_DIR/random.bin" bs=64k count=32 2>/dev/null || dd if=/dev/random of="$BENCH_DIR/random.bin" bs=64k count=32 2>/dev/null
  # Ещё один текст (кодоподобный)
  head -n 500 "$ROOT/README.md" 2>/dev/null > "$BENCH_DIR/readme_sample.txt" || true
  # Итоговый размер
  INPUT_BYTES=$(find "$BENCH_DIR" -type f -print0 | xargs -0 stat -f%z 2>/dev/null | awk '{s+=$1}END{print s+0}' || find "$BENCH_DIR" -type f -exec stat -c%s {} + 2>/dev/null | awk '{s+=$1}END{print s+0}')
  [[ -z "$INPUT_BYTES" || "$INPUT_BYTES" -eq 0 ]] && INPUT_BYTES=$(find "$BENCH_DIR" -type f -exec cat {} + | wc -c)
  echo "Bench suite: $BENCH_DIR ($(( INPUT_BYTES / 1024 )) KB)"
}

# Замер времени (секунды с дробной частью)
time_cmd() {
  local start end
  start=$(python3 -c 'import time; print(time.time())')
  "$@" >/dev/null 2>&1
  end=$(python3 -c 'import time; print(time.time())')
  python3 -c "print(round($end - $start, 3))"
}

# Размер файла в байтах (macOS / Linux)
file_size() {
  if [[ -f "$1" ]]; then
    stat -f%z "$1" 2>/dev/null || stat -c%s "$1" 2>/dev/null
  else
    echo 0
  fi
}

run_bench() {
  local preset="$1"
  local archive="$RESULTS_DIR/out_${preset}.oz"
  local comp_time dec_time out_bytes ratio comp_mbs dec_mbs
  mkdir -p "$RESULTS_DIR"

  comp_time=$(time_cmd "${RUN_CMD[@]}" compress --preset "$preset" "$BENCH_DIR" "$archive")
  out_bytes=$(file_size "$archive")
  [[ "$INPUT_BYTES" -gt 0 ]] && ratio=$(python3 -c "print(round($out_bytes / $INPUT_BYTES, 3))") || ratio="?"
  comp_mbs=$(python3 -c "print(round($INPUT_BYTES / 1024 / 1024 / max(0.001, float('$comp_time')), 2))" 2>/dev/null || echo "—")

  rm -rf "$ARCHIVE_DIR"
  dec_time=$(time_cmd "${RUN_CMD[@]}" decompress "$archive" "$ARCHIVE_DIR")
  dec_mbs=$(python3 -c "print(round($INPUT_BYTES / 1024 / 1024 / max(0.001, float('$dec_time')), 2))" 2>/dev/null || echo "—")

  echo "$preset	$INPUT_BYTES	$out_bytes	$ratio	$comp_time	$dec_time	$comp_mbs	$dec_mbs"
}

# Печать таблицы
print_table() {
  local input_mb
  input_mb=$(python3 -c "print(round($INPUT_BYTES / 1024 / 1024, 2))")
  echo ""
  echo "=============================================="
  echo "  OmegaZip benchmark — основные форматы/пресеты"
  echo "  Вход: $BENCH_DIR (текст, JSON, бинарь) — ${input_mb} MB"
  echo "=============================================="
  printf "%-8s | %8s | %6s | %10s | %10s | %10s | %10s\n" \
    "PRESET" "ARCH MB" "RATIO" "COMP s" "DEC s" "COMP MB/s" "DEC MB/s"
  echo "----------+----------+--------+------------+------------+------------+------------"
  while IFS=$'\t' read -r preset _ out_bytes ratio comp_time dec_time comp_mbs dec_mbs; do
    local arch_mb
    arch_mb=$(python3 -c "print(round($out_bytes / 1024 / 1024, 2))")
    printf "%-8s | %8s | %6s | %10s | %10s | %10s | %10s\n" \
      "$preset" "$arch_mb" "$ratio" "$comp_time" "$dec_time" "$comp_mbs" "$dec_mbs"
  done
  echo "=============================================="
  echo "  RATIO = размер архива / размер входа (меньше = лучше сжатие)"
  echo "=============================================="
}

# --- main ---
prepare_suite
echo "Running presets: fast, balanced, max, ultra..."
{
  run_bench fast
  run_bench balanced
  run_bench max
  run_bench ultra
} | tee "$RESULTS_DIR/raw.tsv"
echo ""
print_table < "$RESULTS_DIR/raw.tsv"

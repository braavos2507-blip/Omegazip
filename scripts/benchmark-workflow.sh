#!/usr/bin/env bash
# Full benchmark for OmegaZip workflow-like behavior on macOS.
# - Uses real files from a test folder (if present)
# - Generates additional synthetic corpus for diverse extensions
# - Measures compress/decompress time and size ratio
#
# Usage:
#   ./scripts/benchmark-workflow.sh [OPTIONS] [TEST_DIR]
#
# Options:
#   --real-only          Skip synthetic corpus; only files copied from TEST_DIR (see below)
#   --out-report PATH    Write REPORT.md to PATH (default: WORK/REPORT.md under work dir)
#
# Outputs:
#   /tmp/omegazip-full-bench/results.csv
#   REPORT.md (default: /tmp/omegazip-full-bench/REPORT.md, or --out-report)

set -euo pipefail

TEST_DIR="${TEST_DIR:-/Users/renat/Documents/Project/Для тестов}"
REAL_ONLY=0
OUT_REPORT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --real-only)
      REAL_ONLY=1
      shift
      ;;
    --out-report)
      OUT_REPORT="${2:?}"
      shift 2
      ;;
    -*)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
    *)
      TEST_DIR="$1"
      shift
      ;;
  esac
done

BIN="/Applications/OmegaZip.app/Contents/MacOS/omegazip"
WORK="/tmp/omegazip-full-bench"
IN_DIR="$WORK/inputs"
OUT_DIR="$WORK/outputs"

if [[ ! -x "$BIN" ]]; then
  echo "ERROR: omegazip binary not found at: $BIN" >&2
  exit 1
fi

rm -rf "$WORK"
mkdir -p "$IN_DIR" "$OUT_DIR"

pick_ext_auto() {
  local f="$1"
  local base ext
  if [[ -d "$f" ]]; then
    printf '%s' "oz"
    return 0
  fi
  base="${f##*/}"
  ext="${base##*.}"
  ext="$(printf '%s' "$ext" | tr '[:upper:]' '[:lower:]')"
  if [[ "$ext" == "txt" || "$ext" == "md" || "$ext" == "markdown" || "$ext" == "epub" || "$ext" == "csv" || "$ext" == "json" || "$ext" == "xml" || "$ext" == "yaml" || "$ext" == "yml" || "$ext" == "toml" || "$ext" == "ini" || "$ext" == "log" || "$ext" == "sql" || "$ext" == "rs" || "$ext" == "js" || "$ext" == "ts" || "$ext" == "tsx" || "$ext" == "jsx" || "$ext" == "html" || "$ext" == "css" || "$ext" == "java" || "$ext" == "kt" || "$ext" == "go" || "$ext" == "py" || "$ext" == "c" || "$ext" == "cpp" || "$ext" == "h" || "$ext" == "hpp" ]]; then
    printf '%s' "oz"
  else
    printf '%s' "zip"
  fi
}

bytes_of() {
  stat -f "%z" "$1"
}

# Sum of file sizes under a directory (for decompress output).
dir_bytes_sum() {
  local d="$1"
  local sum=0 b
  while IFS= read -r -d '' f; do
    b="$(stat -f '%z' "$f" 2>/dev/null || echo 0)"
    sum=$((sum + b))
  done < <(find "$d" -type f -print0 2>/dev/null)
  echo "$sum"
}

copy_first_by_ext() {
  local ext="$1"
  local dst="$2"
  local src
  src="$(ls -1 "$TEST_DIR"/*."$ext" "$TEST_DIR"/*/*."$ext" 2>/dev/null | head -n 1 || true)"
  if [[ -n "${src:-}" && -f "$src" ]]; then
    cp "$src" "$dst"
    return 0
  fi
  return 1
}

echo "Preparing real samples from: $TEST_DIR"
copy_first_by_ext "pdf" "$IN_DIR/real_sample_pdf.pdf" || true
copy_first_by_ext "zip" "$IN_DIR/real_sample_zip.zip" || true
copy_first_by_ext "jpg" "$IN_DIR/real_sample_jpg.jpg" || true
copy_first_by_ext "png" "$IN_DIR/real_sample_png.png" || true
copy_first_by_ext "mp4" "$IN_DIR/real_sample_mp4.mp4" || true
copy_first_by_ext "docx" "$IN_DIR/real_sample_docx.docx" || true

if [[ "$REAL_ONLY" -eq 0 ]]; then
  echo "Generating synthetic corpus..."
  # Text-ish corpus (highly compressible)
  : > "$IN_DIR/synth_text.txt"
  i=1
  while [[ $i -le 400000 ]]; do
    echo "OmegaZip benchmark line with repeated text 1234567890 abcdefghijklmnopqrstuvwxyz" >> "$IN_DIR/synth_text.txt"
    i=$((i + 1))
  done

  # JSON-ish corpus (compressible structured data)
  {
    echo "["
    i=1
    while [[ $i -le 120000 ]]; do
      echo "{\"id\":$i,\"name\":\"user_$i\",\"city\":\"Moscow\",\"tags\":[\"bench\",\"omega\",\"zip\"],\"active\":true},"
      i=$((i + 1))
    done
    echo "{\"id\":0,\"name\":\"end\",\"city\":\"X\",\"tags\":[],\"active\":false}"
    echo "]"
  } > "$IN_DIR/synth_data.json"

  # CSV-ish corpus
  {
    echo "id,name,city,score"
    i=1
    while [[ $i -le 300000 ]]; do
      echo "$i,user_$i,City_$((i % 100)),$((i % 1000))"
      i=$((i + 1))
    done
  } > "$IN_DIR/synth_table.csv"

  # Binary random corpus (poorly compressible)
  dd if=/dev/urandom of="$IN_DIR/synth_random.bin" bs=1m count=40 status=none

  # Folder with many small files
  mkdir -p "$IN_DIR/synth_folder"
  i=1
  while [[ $i -le 2000 ]]; do
    printf 'file=%d\npayload=%s\n' "$i" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" > "$IN_DIR/synth_folder/file_$i.txt"
    i=$((i + 1))
  done
else
  echo "Skipping synthetic corpus (--real-only)."
fi

shopt -s nullglob
_inputs=( "$IN_DIR"/* )
shopt -u nullglob
if [[ ${#_inputs[@]} -eq 0 ]]; then
  echo "ERROR: no input files under $IN_DIR (check TEST_DIR=$TEST_DIR)" >&2
  exit 1
fi

RESULTS="$WORK/results.csv"
echo "case,input_path,input_bytes,chosen_archive,output_bytes,ratio,time_real_s,time_user_s,time_sys_s,extracted_bytes,status" > "$RESULTS"

run_compress_case() {
  local case_name="$1"
  local input_path="$2"
  local in_bytes out_ext out_path time_file out_file status

  in_bytes="$(bytes_of "$input_path")"
  out_ext="$(pick_ext_auto "$input_path")"
  out_path="$OUT_DIR/${case_name}.${out_ext}"
  time_file="$OUT_DIR/${case_name}.compress.time"
  out_file="$OUT_DIR/${case_name}.compress.out"

  status="ok"
  if ! /usr/bin/time -p "$BIN" compress "$input_path" "$out_path" >"$out_file" 2>"$time_file"; then
    status="compress_failed"
    echo "${case_name},${input_path},${in_bytes},${out_path},0,0,0,0,0,0,${status}" >> "$RESULTS"
    return
  fi

  local out_bytes real_s user_s sys_s ratio
  out_bytes="$(bytes_of "$out_path")"
  real_s="$(awk '/^real/{print $2}' "$time_file")"
  user_s="$(awk '/^user/{print $2}' "$time_file")"
  sys_s="$(awk '/^sys/{print $2}' "$time_file")"
  ratio="$(awk -v i="$in_bytes" -v o="$out_bytes" 'BEGIN{ if(i==0){print "0"} else {printf "%.4f", o/i} }')"

  echo "${case_name},${input_path},${in_bytes},${out_path},${out_bytes},${ratio},${real_s},${user_s},${sys_s},0,${status}" >> "$RESULTS"

  # Decompress benchmark for produced archive
  local dcase ddir dtime dout dstatus ext_bytes
  dcase="${case_name}_decompress"
  ddir="$OUT_DIR/${dcase}"
  dtime="$OUT_DIR/${dcase}.time"
  dout="$OUT_DIR/${dcase}.out"
  dstatus="ok"
  mkdir -p "$ddir"
  if ! /usr/bin/time -p "$BIN" decompress "$out_path" "$ddir" >"$dout" 2>"$dtime"; then
    dstatus="decompress_failed"
    echo "${dcase},${out_path},${out_bytes},${ddir},0,0,0,0,0,0,${dstatus}" >> "$RESULTS"
    return
  fi
  local dreal duser dsys
  dreal="$(awk '/^real/{print $2}' "$dtime")"
  duser="$(awk '/^user/{print $2}' "$dtime")"
  dsys="$(awk '/^sys/{print $2}' "$dtime")"
  ext_bytes="$(dir_bytes_sum "$ddir")"
  echo "${dcase},${out_path},${out_bytes},${ddir},0,0,${dreal},${duser},${dsys},${ext_bytes},${dstatus}" >> "$RESULTS"
}

echo "Running benchmark cases..."
for f in "$IN_DIR"/*; do
  [[ -e "$f" ]] || continue
  if [[ -d "$f" ]]; then
    case_name="$(basename "$f")_dir"
  else
    base="$(basename "$f")"
    stem="${base%.*}"
    ext="${base##*.}"
    case_name="${stem}_${ext}"
  fi
  run_compress_case "$case_name" "$f"
done

REPORT="${OUT_REPORT:-$WORK/REPORT.md}"
MODE_LABEL="full (synthetic + real samples)"
if [[ "$REAL_ONLY" -eq 1 ]]; then
  MODE_LABEL="real-only (samples from TEST_DIR)"
fi
{
  echo "# OmegaZip workflow benchmark"
  echo
  echo "- Generated: $(date +%Y-%m-%d)"
  echo "- Binary: \`$BIN\`"
  echo "- Mode: **$MODE_LABEL**"
  echo "- Test dir source: \`$TEST_DIR\`"
  echo "- Work dir: \`$WORK\`"
  echo
  echo "## Format selection (install-context-menu \`pick_ext_auto\`)"
  echo
  echo "| Input | Archive |"
  echo "|---|---|"
  echo "| Folder | \`.oz\` |"
  echo "| \`txt\`, \`md\`, \`epub\`, \`csv\`, \`json\`, code sources (\`rs\`, \`js\`, \`ts\`, …), markup/config (\`xml\`, \`yaml\`, …) | \`.oz\` |"
  echo "| Other extensions (images, video, \`pdf\`, \`zip\`, \`docx\`, …) | \`.zip\` |"
  echo
  echo "## Results"
  echo
  echo "Compress: **Input** = исходный файл/папка; **Archive** = размер архива; **Ratio** = архив / вход."
  echo
  echo "Decompress: **Archive** = размер файла архива; **Extracted** = сумма размеров извлечённых файлов; **Ratio** = extracted / archive."
  echo
  echo "| Case | Input MB | Archive MB | Extracted MB | Ratio | Real(s) | User(s) | Sys(s) | Status |"
  echo "|---|---:|---:|---:|---:|---:|---:|---:|---|"
  awk -F, 'NR>1 {
    is_dec = ($1 ~ /_decompress$/);
    in_mb = $3 / 1048576.0;
    arch_mb = $5 / 1048576.0;
    ext_mb = $10 / 1048576.0;
    status = $11;
    if (is_dec) {
      ratio = ($3 > 0 ? sprintf("%.4f", $10 / $3) : "—");
      printf("| %s | %.4f | — | %.4f | %s | %s | %s | %s | %s |\n", $1, in_mb, ext_mb, ratio, $7, $8, $9, status);
    } else {
      printf("| %s | %.4f | %.4f | — | %s | %s | %s | %s | %s |\n", $1, in_mb, arch_mb, $6, $7, $8, $9, status);
    }
  }' "$RESULTS"
  echo
  echo "CSV: \`$RESULTS\`"
} > "$REPORT"

echo
echo "Benchmark complete."
echo "Report: $REPORT"
echo "CSV:    $RESULTS"

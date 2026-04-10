#!/usr/bin/env bash
# Подсказки по профилированию сжатия (локально, без облака).
# Реальный замер выполняйте вручную — ниже готовые шаблоны команд.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/omegazip"
CORPUS="${CORPUS:-$ROOT/target/bench_suite}"

echo "=== OmegaZip — профилирование compress (локально) ==="
echo ""
echo "1) Соберите release и подготовьте вход (или используйте target/bench_suite после bench.sh):"
echo "   cd $ROOT && cargo build --release -p omegazip"
echo "   bash scripts/bench.sh   # создаёт $ROOT/target/bench_suite"
echo ""
echo "2) macOS — Sample с samply (установка: cargo install samply):"
echo "   samply record $BIN compress --preset balanced $CORPUS /tmp/profile.oz"
echo ""
echo "3) macOS — Instruments (GUI):"
echo "   open -a Instruments"
echo "   Target: $BIN  Arguments: compress --preset balanced <вход> <выход.oz>"
echo ""
echo "4) Linux — perf (нужен perf и debug symbols или frame pointers):"
echo "   perf record -g -- $BIN compress --preset balanced $CORPUS /tmp/profile.oz"
echo "   perf report"
echo ""
echo "5) Узкие места чаще всего: I/O диска, ZSTD/LZ4, хэширование чанков, упаковка манифеста."
echo "   Фиксируйте 1 сценарий за раз (огромная папка | один большой файл | много мелких файлов)."
echo ""
echo "Полный чеклист: docs/MEASURABLE-QUALITY.md (раздел C)"

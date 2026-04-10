#!/usr/bin/env bash
# Полный локальный контур без ручных шагов: тесты, бенчи, корпуса, profile-smoke,
# checklist D, benchmark-workflow, опционально clippy. Пишет:
#   tests/manual-files/results-auto/LATEST-FULL-QA.md
#   tests/manual-files/results-auto/baselines/full-qa-*.log
#
# Не делает: подпись .app, notary, samply (только отмечает наличие в PATH).
# Внешний корпус: CORPUS_EXTRA=/путь (доп. прогон в measure-oz-repo-corpora).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

STAMP="$(date +%Y%m%d-%H%M%S)"
BASE="$ROOT/tests/manual-files/results-auto"
LOG="$BASE/baselines/full-qa-${STAMP}.log"
MD="$BASE/LATEST-FULL-QA.md"
mkdir -p "$BASE/baselines"

BIN="$ROOT/target/release/omegazip"
if [[ ! -x "$BIN" ]]; then
  cargo build --release -p omegazip -q
fi

exec > >(tee "$LOG") 2>&1

echo "=== OmegaZip full local QA — $STAMP ==="
echo "uname: $(uname -a)"
echo "rustc: $(rustc -V 2>/dev/null || echo n/a)"
echo ""

echo "========== cargo test -p omegazip =========="
cargo test -p omegazip

echo ""
echo "========== cargo clippy -p omegazip =========="
cargo clippy -p omegazip --all-targets -q

echo ""
echo "========== qa03-benchmark-ci =========="
bash "$ROOT/scripts/qa03-benchmark-ci.sh"

echo ""
echo "========== bench.sh =========="
bash "$ROOT/scripts/bench.sh"

echo ""
echo "========== measure-oz-advantage (synthetic) =========="
bash "$ROOT/scripts/measure-oz-advantage.sh"

echo ""
echo "========== measure-oz-repo-corpora =========="
bash "$ROOT/scripts/measure-oz-repo-corpora.sh"

echo ""
echo "========== profile-compress-heavy-smoke =========="
bash "$ROOT/scripts/profile-compress-heavy-smoke.sh"
echo "--- profile-smoke-last.txt ---"
cat "$BASE/profile-smoke-last.txt"

echo ""
echo "========== checklist-d-automated =========="
bash "$ROOT/scripts/checklist-d-automated.sh"

echo ""
echo "========== benchmark-workflow --real-only =========="
if [[ "$(uname -s)" == "Darwin" ]]; then
  bash "$ROOT/scripts/benchmark-workflow.sh" --real-only --out-report "$BASE/BENCH-WORKFLOW-LATEST.md" "$ROOT/tests/manual-files/downloads"
else
  echo "SKIP: benchmark-workflow.sh ориентирован на macOS (stat -f); на Linux пропуск."
fi

echo ""
echo "========== tooling =========="
if command -v samply >/dev/null 2>&1; then
  echo "samply: установлен (запуск вручную: scripts/profile-compress-local.sh)"
else
  echo "samply: не в PATH (cargo install samply — опционально)"
fi

echo ""
echo "=== full local QA OK — log: $LOG ==="

# Краткий markdown для git (без полного лога — он в baselines/)
PROFILE_LINES=""
if [[ -f "$BASE/profile-smoke-last.txt" ]]; then
  PROFILE_LINES="$(cat "$BASE/profile-smoke-last.txt")"
fi

cat >"$MD" <<EOF
# Полный локальный QA (автогенерация)

**Время:** $(date -Iseconds 2>/dev/null || date)  
**Лог:** [baselines/full-qa-${STAMP}.log](baselines/full-qa-${STAMP}.log)  
**uname:** $(uname -srmo 2>/dev/null || uname -a)

## Команда

\`\`\`bash
bash scripts/run-full-local-qa.sh
# или
npm run measure:everything-local
\`\`\`

## Результаты (кратко)

- **cargo test -p omegazip** — см. лог (все интеграционные + lib).
- **cargo clippy** — см. лог.
- **Profile-smoke** (~30 MiB дедуп):

\`\`\`
${PROFILE_LINES}
\`\`\`

- **BENCH-WORKFLOW-LATEST:** [BENCH-WORKFLOW-LATEST.md](BENCH-WORKFLOW-LATEST.md)
- **samply:** $(command -v samply >/dev/null 2>&1 && echo "в PATH" || echo "не в PATH")

Полный вывод — только в \`baselines/full-qa-*.log\` (не дублируется здесь из-за размера).

EOF

echo ""
echo "Markdown: $MD"

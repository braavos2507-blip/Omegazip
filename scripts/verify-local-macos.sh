#!/usr/bin/env bash
# Быстрая проверка: .app, CLI, workflow в Services, лог. Только для локальной машины.
# Использование:
#   ./scripts/verify-local-macos.sh [путь/к/OmegaZip.app]
# По умолчанию: /Applications/OmegaZip.app, иначе ./dist/OmegaZip.app если есть.

set -euo pipefail

APP="${1:-}"
if [[ -z "$APP" ]]; then
  if [[ -d "/Applications/OmegaZip.app" ]]; then
    APP="/Applications/OmegaZip.app"
  elif [[ -d "./dist/OmegaZip.app" ]]; then
    APP="./dist/OmegaZip.app"
  else
    echo "Укажите путь к OmegaZip.app или соберите в dist/ (build-app.sh)." >&2
    exit 1
  fi
fi

BIN="$APP/Contents/MacOS/omegazip"
echo "=== OmegaZip — локальная проверка (macOS) ==="
echo "Приложение: $APP"
echo

if [[ ! -d "$APP" ]]; then
  echo "FAIL: каталог .app не найден"
  exit 1
fi
echo "OK: .app на месте"

if [[ ! -x "$BIN" ]]; then
  echo "FAIL: нет исполняемого omegazip: $BIN"
  exit 1
fi
echo "OK: CLI omegazip"

if ! "$BIN" --help >/dev/null 2>&1; then
  echo "FAIL: omegazip --help"
  exit 1
fi
echo "OK: omegazip отвечает на --help"

SERVICES="${HOME}/Library/Services"
echo
echo "Папка сервисов: $SERVICES"
if [[ -d "$SERVICES" ]] && ls -1 "$SERVICES" 2>/dev/null | grep -qi omega; then
  echo "OK: найдены workflow с «omega» в имени:"
  ls -1 "$SERVICES" | grep -i omega || true
else
  echo "WARN: не видно workflow OmegaZip — выполните: ./scripts/install-context-menu.sh"
fi

LOG="/tmp/OmegaZip-workflow.log"
echo
if [[ -f "$LOG" ]]; then
  echo "Лог $LOG (последние 8 строк):"
  tail -n 8 "$LOG"
else
  echo "Лог $LOG ещё не создавался (нормально до первого запуска из Finder)."
fi

echo
echo "=== готово ==="
echo "Дальше вручную: Finder → ПКМ на тестовом файле → Сервисы → Сжать/Распаковать."

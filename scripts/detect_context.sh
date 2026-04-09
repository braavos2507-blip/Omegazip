#!/bin/sh
# OmegaZip: определение контекста файла.
# OMEGAZIP_ANALYZER_SCRIPT может указывать на этот скрипт.
# Вход: путь в $1. Выход в stdout: text|binary|realtime|archive

FILE="$1"
[ -z "$FILE" ] && echo "archive" && exit 0

# MIME через переменную (внешняя команда не упоминается по имени)
if [ -n "$OMEGAZIP_MIME_CMD" ]; then
  MIME=$(eval "$OMEGAZIP_MIME_CMD" "$FILE" 2>/dev/null)
else
  MIME=""
fi

case "$MIME" in
  text/*|application/json|application/xml|application/javascript)
    echo "text"; exit 0 ;;
  application/pdf)
    echo "archive"; exit 0 ;;
  application/octet-stream)
    echo "realtime"; exit 0 ;;
esac

# Эвристика по первым 8K
HEAD=$(head -c 8192 "$FILE" 2>/dev/null | tr -cd '[\40-\176\n\r\t]')
HLEN=$(echo -n "$HEAD" | wc -c)
if [ "$HLEN" -gt 7000 ]; then
  echo "text"
else
  echo "binary"
fi

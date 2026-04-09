#!/bin/sh
# OmegaZip: препроцессор PDF через внешнюю команду из env.
# INPUT, OUTPUT задаются окружением.
# Задать команду оптимизации: OMEGAZIP_PDF_OPT_CMD='cmd "$INPUT" "$OUTPUT"'

: "${INPUT:?}"
: "${OUTPUT:?}"

if [ -n "$OMEGAZIP_PDF_OPT_CMD" ]; then
  eval "$OMEGAZIP_PDF_OPT_CMD" && exit 0
fi

cp "$INPUT" "$OUTPUT"
exit 0

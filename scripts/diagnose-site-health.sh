#!/usr/bin/env bash
# Быстрая проверка omegazip.ru (origin bucket vs CDN vs SSL).
set -euo pipefail

DOMAIN="${OZ_SITE_DOMAIN:-omegazip.ru}"
WWW="www.${DOMAIN}"
ORIGIN="${OZ_SITE_ORIGIN:-omegazip-ru-site-ec177c5f.website.yandexcloud.net}"

echo "== DNS =="
dig +short "$WWW" CNAME A 2>/dev/null || true
dig +short "$DOMAIN" A 2>/dev/null || true

echo ""
echo "== Origin bucket (должен быть HTTP 200) =="
curl -sS -o /dev/null -w "http://%s/ -> %{http_code}\n" "$ORIGIN" "$ORIGIN"

echo ""
echo "== CDN www (ожидается 200; сейчас часто 404) =="
curl -sS -o /dev/null -w "http://%s/ -> %{http_code}\n" "$WWW" "http://$WWW/" || echo "http failed"
curl -k -sS -o /dev/null -w "https://%s/ -> %{http_code}\n" "$WWW" "https://$WWW/" || echo "https failed"

echo ""
echo "== TLS certificate (должен содержать $WWW или $DOMAIN) =="
if command -v openssl >/dev/null 2>&1; then
  openssl s_client -connect "${WWW}:443" -servername "$WWW" </dev/null 2>/dev/null \
    | openssl x509 -noout -subject -ext subjectAltName 2>/dev/null || echo "openssl check failed"
else
  echo "openssl not installed"
fi

echo ""
echo "Если origin=200, а www=404 — чинить CDN в Yandex Cloud (origin group / Host header / purge)."
echo "Если cert=*.yccdn.cloud.yandex.net — перепривязать Certificate Manager к CDN-ресурсу."

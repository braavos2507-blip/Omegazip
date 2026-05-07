#!/usr/bin/env bash
set -euo pipefail

# Deploy static site to Yandex Object Storage via AWS CLI (S3 API).
# Required env:
#   YC_SITE_BUCKET=omegazip-ru-site-...
# Optional env:
#   YC_SITE_DIR=site
#   YC_STORAGE_ENDPOINT=https://storage.yandexcloud.net

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SITE_DIR="${YC_SITE_DIR:-$ROOT/site}"
BUCKET="${YC_SITE_BUCKET:-}"
ENDPOINT="${YC_STORAGE_ENDPOINT:-https://storage.yandexcloud.net}"

if [[ -z "$BUCKET" ]]; then
  echo "ERROR: set YC_SITE_BUCKET env var"
  exit 1
fi

if [[ ! -d "$SITE_DIR" ]]; then
  echo "ERROR: site dir not found: $SITE_DIR"
  exit 1
fi

if ! command -v aws >/dev/null 2>&1; then
  echo "ERROR: aws CLI is required (brew install awscli / apt install awscli)"
  exit 1
fi

echo "Deploying site from: $SITE_DIR"
echo "Bucket: s3://$BUCKET"
echo "Endpoint: $ENDPOINT"

aws s3 sync "$SITE_DIR/" "s3://$BUCKET/" \
  --endpoint-url "$ENDPOINT" \
  --delete \
  --cache-control "public,max-age=300"

echo "Done. Verify:"
echo "  https://www.omegazip.ru"


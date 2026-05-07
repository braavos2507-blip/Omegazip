terraform {
  required_version = ">= 1.6.0"
  required_providers {
    yandex = {
      source  = "yandex-cloud/yandex"
      version = ">= 0.125.0"
    }
  }
}

provider "yandex" {
  cloud_id  = var.cloud_id
  folder_id = var.folder_id

  # auth: either token or service_account_key_file
  token                    = var.yc_token
  service_account_key_file = var.service_account_key_file
}

locals {
  site_domain      = var.domain
  site_domain_www  = "www.${var.domain}"
  site_bucket_name = var.site_bucket_name
  dl_bucket_name   = var.downloads_bucket_name
}

# -----------------------------
# Object Storage buckets
# -----------------------------

resource "yandex_storage_bucket" "site" {
  bucket = local.site_bucket_name

  # Static website hosting endpoint (needed for direct website mode).
  website {
    index_document = "index.html"
    error_document = "404.html"
  }

  # Public read (static website). If you want private + signed URLs, change later.
  anonymous_access_flags {
    read = true
    list = false
  }

  versioning {
    enabled = false
  }
}

resource "yandex_storage_bucket" "downloads" {
  count  = var.enable_downloads_bucket ? 1 : 0
  bucket = local.dl_bucket_name

  anonymous_access_flags {
    read = true
    list = false
  }

  versioning {
    enabled = false
  }
}

# -----------------------------
# Certificate (managed)
# -----------------------------

resource "yandex_cm_certificate" "site" {
  name = "omegazip-ru-cert"

  domains = [
    local.site_domain,
    local.site_domain_www,
  ]

  managed {
    challenge_type = "DNS_CNAME"
  }
}

# -----------------------------
# CDN (fronting the website)
# -----------------------------

resource "yandex_cdn_origin_group" "site" {
  name = "omegazip-site-origin-group"

  origin {
    source = yandex_storage_bucket.site.bucket_domain_name
  }
}

resource "yandex_cdn_resource" "site" {
  cname               = local.site_domain_www
  active              = true
  origin_protocol     = "https"
  secondary_hostnames = [local.site_domain]

  origin_group_id = yandex_cdn_origin_group.site.id

  ssl_certificate {
    type = "certificate_manager"
    certificate_manager_id = yandex_cm_certificate.site.id
  }

  options {
    redirect_http_to_https = true
  }
}

# -----------------------------
# DNS (optional: manage zone in YC)
# -----------------------------

resource "yandex_dns_zone" "zone" {
  count = var.manage_dns_in_yc ? 1 : 0
  name  = "omegazip-ru-zone"
  zone  = "${var.domain}."
  public = true
}

# Challenge record(s) for Certificate Manager (DNS_CNAME)
resource "yandex_dns_recordset" "cm_challenges" {
  count  = var.manage_dns_in_yc ? length(yandex_cm_certificate.site.challenges) : 0
  zone_id = yandex_dns_zone.zone[0].id

  name = yandex_cm_certificate.site.challenges[count.index].dns_name
  type = yandex_cm_certificate.site.challenges[count.index].dns_type
  ttl  = 300
  data = [yandex_cm_certificate.site.challenges[count.index].dns_value]
}

resource "yandex_dns_recordset" "www_cname" {
  count   = var.manage_dns_in_yc ? 1 : 0
  zone_id = yandex_dns_zone.zone[0].id
  name    = "www"
  type    = "CNAME"
  ttl     = 300
  data    = [yandex_cdn_resource.site.domain_name]
}


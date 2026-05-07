output "site_bucket_domain" {
  value = yandex_storage_bucket.site.bucket_domain_name
}

output "cdn_cname" {
  value = yandex_cdn_resource.site.cname
}

output "cdn_provider_host" {
  # YC CDN обычно выдаёт host вида <cname>.cdn.yandex.net; но точное значение зависит от ресурса.
  # Пользуемся тем, что возвращает провайдер.
  value = yandex_cdn_resource.site.domain_name
}

output "certificate_challenges" {
  value = [
    for ch in yandex_cm_certificate.site.challenges : {
      dns_name  = ch.dns_name
      dns_type  = ch.dns_type
      dns_value = ch.dns_value
    }
  ]
}

output "dns_zone_name_servers" {
  value       = var.manage_dns_in_yc ? yandex_dns_zone.zone[0].name_servers : []
  description = "If manage_dns_in_yc=true: set these NS at registrar for delegation"
}


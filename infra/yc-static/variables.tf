variable "cloud_id" {
  type        = string
  description = "Yandex Cloud ID"
}

variable "folder_id" {
  type        = string
  description = "Yandex Cloud Folder ID"
}

variable "yc_token" {
  type        = string
  description = "OAuth/IAM token for YC (optional if service_account_key_file is set)"
  default     = null
  sensitive   = true
}

variable "service_account_key_file" {
  type        = string
  description = "Path to service account authorized key JSON (recommended). Optional if yc_token is set."
  default     = null
}

variable "domain" {
  type        = string
  description = "Root domain, e.g. omegazip.ru"
}

variable "site_bucket_name" {
  type        = string
  description = "Bucket name for static site (must be globally unique)"
}

variable "enable_downloads_bucket" {
  type        = bool
  description = "Whether to create a separate downloads bucket"
  default     = true
}

variable "downloads_bucket_name" {
  type        = string
  description = "Bucket name for downloads (must be globally unique)"
  default     = null
}

variable "manage_dns_in_yc" {
  type        = bool
  description = "If true, create DNS zone and records in Yandex Cloud DNS"
  default     = true
}


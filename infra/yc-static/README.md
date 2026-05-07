# YC static site (OmegaZip)

Terraform-конфигурация для:
- Object Storage бакета со статическим сайтом
- (опционально) бакета для дистрибутивов
- CDN
- Certificate Manager (TLS)
- Cloud DNS (зона и записи) — опционально

## Быстрый старт

1) Поставить YC CLI и Terraform.
2) Получить `folder_id` и токен/SA key.
3) Заполнить `terraform.tfvars`.
4) `terraform init && terraform apply`.

См. `docs/DEPLOY-YC-STATIC.md`.


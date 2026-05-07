# Деплой статического сайта в Yandex Cloud (omegazip.ru)

Цель: **лендинг + страница скачивания + донаты** без бэкенда.

Архитектура:
- **Object Storage**: бакет со статическим сайтом (`index.html` и ассеты)
- **CDN**: раздача через CDN-домен
- **Certificate Manager**: TLS-сертификат для `omegazip.ru` и `www.omegazip.ru`
- **Cloud DNS**: зона `omegazip.ru.` (или внешняя DNS у регистратора — тогда просто записи)

> Рекомендуется: сайт хранить в одном бакете, **инсталляторы** (DMG/MSI/AppImage/ZIP) — в отдельном бакете/префиксе.

## Что нужно от тебя (1 раз)

1) **YC Folder ID** (папка, где создаём ресурсы).  
2) Как будем управлять DNS:
   - **Вариант A (рекомендую): Cloud DNS в Yandex** — ты меняешь NS у регистратора на NS из YC.
   - **Вариант B: DNS у регистратора** — ты добавляешь записи, которые я дам (CNAME/A/TXT).
3) Доступ для Terraform:
   - либо **OAuth token** (`yc iam create-token`)
   - либо **Service Account key JSON** (лучше для повторяемости).
4) Поддомены:
   - подтверждаем: `omegazip.ru` и `www.omegazip.ru` (редирект на основной домен).
5) Где будут храниться файлы для скачивания:
   - **в YC Object Storage** (рекомендую для скорости/CDN),
   - или **GitHub Releases**, а сайт только ссылается.

## Важный нюанс по DNS (apex-домен)

`www.omegazip.ru` легко направляется на CDN через `CNAME`.  
Для корня `omegazip.ru` обычно нельзя ставить `CNAME` (ограничение DNS apex), поэтому:
- либо используем DNS-провайдера с `ALIAS/ANAME/flattening` на CDN host,
- либо делаем основной домен `www.omegazip.ru`, а `omegazip.ru` — редирект у регистратора/доп. сервиса.

## Что я подготовлю в репозитории

- `infra/yc-static/` — Terraform: бакеты, CDN, сертификат, DNS-зона/записи
- `site/` — минимальный статический сайт (лендинг + download + donate)
- `scripts/yc-deploy-site.sh` — сборка/публикация (sync в бакет + инвалидация CDN)

## Запуск (после того как ты дашь данные)

Дальше будет пошагово:
1) `terraform init`
2) `terraform apply`
3) делегирование DNS (или ручные записи у регистратора)
4) загрузка сайта в бакет
5) проверка HTTPS и редиректа `www`

## Что уже готово в репозитории

- Terraform: `infra/yc-static/`
- Пример сайта: `site/index.html`, `site/404.html`
- Скрипт публикации: `scripts/yc-deploy-site.sh`

## Команды для тебя (чтобы прислать мне входные данные)

1) Узнать cloud/folder:

```bash
yc resource-manager cloud list
yc resource-manager folder list --cloud-id <CLOUD_ID>
```

2) Получить токен (если не через SA key):

```bash
yc iam create-token
```

3) Если через сервисный аккаунт (предпочтительно):
- создай SA с правами на Object Storage, CDN, Certificate Manager, DNS;
- скачай authorized key json и пришли путь к файлу.

## После `terraform apply`

Публикация сайта:

```bash
export YC_SITE_BUCKET=<site_bucket_name>
bash scripts/yc-deploy-site.sh
```



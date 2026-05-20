# План запуска сайта omegazip.ru

## Цель

Запустить лендинг OmegaZip: преимущества, результаты тестов, скачивание, донаты.

## Этапы

### Этап 1 — контент (готово в репозитории)

- [x] Главная: `site/index.html`
- [x] Донат: `site/donate.html`
- [x] Стили: `site/styles.css`
- [ ] Подставить реальные ссылки скачивания (Windows/macOS/Linux)
- [ ] Подставить реальную ссылку оплаты доната

### Этап 2 — хостинг (Yandex Cloud, статика)

1. Создать Object Storage bucket для сайта
2. Подключить CDN + HTTPS (Certificate Manager)
3. Настроить DNS `omegazip.ru` и `www.omegazip.ru`
4. Деплой: `bash scripts/yc-deploy-site.sh`

Подробно: [DEPLOY-YC-STATIC.md](DEPLOY-YC-STATIC.md)

### Этап 3 — донаты

Варианты:
- Boosty / DonationAlerts (быстрый старт)
- YooKassa hosted page (гибче, нужна регистрация ИП/самозанятость по правилам сервиса)

Обновить:
- `site/donate.html` — `DONATE_URL`
- `ui/index.html` — ссылка в плашке приложения (`https://omegazip.ru/donate`)

### Этап 4 — релизные артефакты

- Выложить `.msi` / `.dmg` / Linux bundle
- Обновить кнопки «Скачать» на сайте
- Проверить clean-machine сценарий на Windows/macOS

### Этап 5 — soft launch

- Открыть сайт
- Собрать обратную связь (форма/email/Telegram)
- Итерации по UX и стабильности

## Что нужно от владельца проекта

1. **Yandex Cloud**: `cloud_id`, `folder_id`, токен или SA key
2. **DNS**: делегировать NS в YC или дать доступ к DNS у регистратора
3. **Ссылка доната**: URL платёжной страницы
4. **Ссылки на скачивание**: GitHub Releases или bucket с инсталляторами
5. **Контакт**: email/Telegram для страницы поддержки (опционально)

## Донат в приложении

Реализовано в `ui/index.html`:
- мягкая плашка (не агрессивная)
- показ не чаще 1 раза в 7 дней
- после «Поддержать» / «Я уже поддержал» — пауза 60 дней
- ссылка: `https://omegazip.ru/donate`

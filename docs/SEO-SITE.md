# SEO на omegazip.ru

## В репозитории (`site/`)

- `robots.txt`, `sitemap.xml`
- `omegazipindexnow2026.txt` — ключ IndexNow
- Страницы: `/`, `faq.html`, `oz.html`, `about.html`, `donate.html`, `changelog.html`, `privacy.html`, `open-7z.html`, `oz-vs-zip.html`, `macos-archiver.html`, `windows-context-menu.html`, `backup-folder.html`, `best-free-archiver.html`, `zip-to-oz.html`, `encrypt-archive.html`, `linux-deb-archiver.html`
- `checksums.json`, редирект apex→www: CDN + redirect-бакет (301), запасной JS в `site-config.js`
- На каждой: canonical, Open Graph, Twitter Card (главная), JSON-LD
- `favicon.ico`, `apple-touch-icon.png`, `og-image.png`

## После деплоя

```bash
bash scripts/yc-deploy-site.sh   # включает IndexNow ping
# или
npm run site:indexnow
```

## Вручную (один раз после NS)

1. [Google Search Console](https://search.google.com/search-console) — `https://www.omegazip.ru`, sitemap: `https://www.omegazip.ru/sitemap.xml`
2. [Яндекс.Вебмастер](https://webmaster.yandex.ru/) — то же
3. Запросить индексацию главной

## Не в коде

- Яндекс.Метрика / GA: счётчик подключается в `site/config.json` полем `analyticsMetrikaId` (пока `REPLACE_ME_METRIKA_ID` — инертно, пока не задан реальный ID). `site-config.js` сам вставляет тег Метрики. После активации добавьте раскрытие в `privacy.html`.
- Внешние статьи и ссылки (Product Hunt, Habr, VC.ru, AlternativeTo) — отдельная задача продвижения.

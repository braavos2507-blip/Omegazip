# Безопасность OmegaZip

Краткий ориентир для разработчиков и приёмки релиза. Полный «security program» не заменяет.

## Угрозы и контуры доверия

| Область | Риск | Меры в продукте |
|--------|------|-----------------|
| **Входные архивы** | злонамеренный ZIP/.oz/tar и т.д. | Ограничение path traversal (ZIP), проверки в тестах (`compat_roundtrip`, `archive_hardening`); распаковка — не следовать за симлинками при упаковке. |
| **Пути на диск** | выход за целевой каталог | CLI/GUI должны использовать канонические пути; при сомнениях — доработать и зафиксировать тестами. |
| **Пароль / ключ** | утечка в логах | Поддержка `OMEGAZIP_PASSWORD`, `--password-file`; ключи обнуляются (`zeroize`) где применимо. |
| **Зависимости** | известные CVE | `cargo audit` (корень + `src-tauri`), `npm audit`; workflow **Security audit** в GitHub Actions. |
| **Сеть** | поставка подмены | Подпись и нотаризация macOS (`docs/DIST-01-MACOS-SIGNING.md`); релизы только с доверенных артефактов. |

## Что запускать локально

```bash
bash scripts/security-audit.sh
# или по отдельности:
# cargo audit && (cd src-tauri && cargo audit) && npm audit --audit-level=high
```

## Fuzzing (опционально)

Требуется [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz):

```bash
cargo install cargo-fuzz
cd /path/to/OmegaZip
cargo fuzz run huffman_decode -- -runs=10000
```

Цель `huffman_decode` вызывает `huffman::decode` и `analyze_bytes` на случайных байтах — не должно быть паник.

## SBOM (опционально)

- Rust: [`cyclonedx-cargo`](https://github.com/CycloneDX/cyclonedx-rust-cargo) или контейнерный сканер по образу.
- Node: `npm sbom` (при поддержке вашей версии npm) или Syft/Grype по `node_modules`.

## Сообщить об уязвимости

Не открывайте публичный issue с эксплойтом. Напишите владельцу репозитория приватно (email в профиле GitHub или согласованный канал). Укажите версию (`cargo pkgid`, версия GUI из `tauri.conf.json`), ОС и шаги воспроизведения.

## Ограничения

- Нет отдельного bug bounty и формального pentest в этом документе.
- GUI (Tauri/WebView) наследует поверхность атаки движка; следите за обновлениями `@tauri-apps/*` и системных WebView.

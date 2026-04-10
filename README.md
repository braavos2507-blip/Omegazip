# OmegaZip 0.3

Конвейер из 4 модулей + **chunked dedup**, **solid**, **шифрование**, **recovery (Reed-Solomon)**, **экспорт в ZIP**, **пресеты**, **репозиторий бэкапов**, прогресс в GUI и CLI.

## Майлстоуны

- **v1.0** (ядро, GUI, Android, CI, документация) — закрыт; журнал: [.planning/MILESTONES.md](.planning/MILESTONES.md), индекс: [.planning/MILESTONE-V1.0.md](.planning/MILESTONE-V1.0.md).
- **v1.1** (интеграция в ОС по ТЗ) — планы 4–7 сданы; п.8 в ТЗ не входил.  
- **v1.2** (строгое ТЗ, только пробелы: тихая распаковка, ПКМ из коробки, vs топ-5, установка, опц. трей) — [.planning/MILESTONE-V1.2.md](.planning/MILESTONE-V1.2.md), [.planning/ROADMAP.md](.planning/ROADMAP.md).

## Сборка и запуск

```bash
cargo build --release
# Обычное сжатие (v1)
./target/release/omegazip compress <файл|директория> <архив.oz>
# Пресеты: fast | balanced | max | ultra (XZ-9 + recovery)
./target/release/omegazip compress --preset ultra ./data archive.oz
# Chunked, solid, recovery, пароль (или OMEGAZIP_PASSWORD, --password-file)
./target/release/omegazip compress --chunked --solid --recovery --password "secret" ./data archive.oz
./target/release/omegazip decompress --password "secret" archive.oz ./out
# Информация и список файлов
./target/release/omegazip info archive.oz
./target/release/omegazip list archive.oz
# Экспорт в ZIP
./target/release/omegazip export-zip archive.oz archive.zip
# Репозиторий бэкапов (как Borg)
./target/release/omegazip repo init ./myrepo
./target/release/omegazip repo backup ./myrepo ./data
./target/release/omegazip repo list ./myrepo
./target/release/omegazip repo restore ./myrepo 1 ./restored
# Проверка 7-Zip / p7zip и подсказки по установке (нужен для .7z, RAR, ISO, MSI…)
./target/release/omegazip deps
```

## Форматы и 7-Zip

- **Нативно (без 7-Zip):** OmegaZip открывает ZIP, tar (включая .gz/.xz/.bz2/.zst), одиночные .gz/.xz/.bz2/.zst, CAB и `.oz`. Подробная матрица: [docs/FORMATS.md](docs/FORMATS.md). Честное сравнение с ориентиром «топ-5 архиваторов» (7-Zip, WinRAR, …): [docs/VERSUS-TOP5.md](docs/VERSUS-TOP5.md).
- **С [7-Zip](https://www.7-zip.org/) или p7zip** (`7z` / `7zz` / `7za` в `PATH`; на Windows часто находится `Program Files\7-Zip\7z.exe`): создание и чтение **.7z**, распаковка **RAR, ISO, WIM, MSI** и других форматов, которые открывает ваш 7-Zip.
- **Установка OmegaZip и 7-Zip**, автообнаружение: [docs/INSTALL.md](docs/INSTALL.md) (сквозной сценарий «от установки до первой проверки» и чеклист — в начале файла). Проверка: `omegazip deps` или баннер в GUI.
- **RAR:** только распаковка через 7-Zip. Создание `.rar` не поддерживается.

**GUI (сборка):** `npm run build` или `npm run tauri build` → артефакты в `src-tauri/target/release/bundle/` (на macOS при необходимости также `dist/OmegaZip.app`). Статус 7-Zip, чанки, solid, пароль, прогресс, экспорт в ZIP.

**Ассоциации файлов и двойной щелчок:** [docs/FILE-ASSOCIATIONS.md](docs/FILE-ASSOCIATIONS.md).

**Контекстное меню** (сжатие в `.oz` / `.zip` без окна): [docs/CONTEXT-MENU.md](docs/CONTEXT-MENU.md). Кратко: установка, логи, переустановка — [docs/MACOS-QUICKSTART.md](docs/MACOS-QUICKSTART.md). Чеклист приёмки ПКМ на Windows/Linux (когда появятся машины): [docs/QA-WIN-LINUX-PREP.md](docs/QA-WIN-LINUX-PREP.md).

**Умные пресеты .oz** (авто по типу файлов, CLI `--preset auto`): [docs/SMART-PRESETS.md](docs/SMART-PRESETS.md).

## Модули

1. **Семантический анализ** — тип по содержимому: таблица magic-байт, энтропия, доля текста. Опционально скрипт `OMEGAZIP_ANALYZER_SCRIPT` (stdout: text|binary|realtime|archive) и `OMEGAZIP_MIME_SCRIPT` для MIME.

2. **Препроцессор** — только через скрипт из `OMEGAZIP_PREPROCESS_PDF` (переменные `INPUT`, `OUTPUT`).

3. **Выбор кодека** — по контексту выбирается предпочтительный кодек, затем в памяти сравниваются несколько вариантов и берётся лучший по размеру. Кодеки: Dense (энтропийный), Balanced, Fast, MaxRatio, Store (без сжатия).

4. **Глобальный дедуп** — блоки по хешу хранятся в одном экземпляре; перед полным хешем используется фильтр по набору (раздел 1.1 документа) для быстрого отсева.

## Скрипты

- **`npm run test:context-menu`** — автотест логики ПКМ (`pick_ext` / stem для Win/Linux-скриптов); в CI также проверка синтаксиса PowerShell (`omega-context-helper.ps1`).
- `scripts/detect_context.sh` — по MIME и первым байтам выдаёт контекст.
- `scripts/preprocess_pdf.sh` — вызов внешней команды для оптимизации PDF (команда задаётся через переменные).
- `scripts/entropy.sh` — энтропия файла.

Пример с собственным анализатором:

```bash
export OMEGAZIP_ANALYZER_SCRIPT="$(pwd)/scripts/detect_context.sh"
./target/release/omegazip compress ./data archive.oz
```

## Формат архива `.oz`

- **v1:** `OMEGAZIP\x01`, длина манифеста (u32 LE), JSON-манифест, блоки (32 байта hash + 1 байт кодек + 4 байта длина + данные).
- **v2:** `OMEGAZIP\x02`, флаги (bit0=chunked, bit1=encrypted, bit2=solid), при шифровании — соль 16 байт, далее манифест и блоки. В манифесте: при chunked — `chunks: [{hash_hex, algo, len}]`, при solid — `solid: {stream_id, offset, length}`. Зашифрованные блоки: nonce 12 + ciphertext+tag (ChaCha20-Poly1305), ключ из пароля (Argon2).

## Новое в 0.2

| Фича | Описание |
|------|----------|
| **Chunked dedup** | Разбиение на чанки 64 KiB, дедуп по хешу чанка (как Borg/restic) — выигрыш на VM-образах и видео с повторами |
| **Solid** | Все файлы в один сжатый поток (как 7-Zip) — лучше сжатие при похожих файлах |
| **Шифрование** | Пароль → Argon2 → ключ, каждый блок шифруется ChaCha20-Poly1305 |
| **Экспорт в ZIP** | Конвертация .oz в обычный ZIP — открывается в любом архиваторе |
| **Прогресс в GUI** | Прогресс-бар и текущий файл при сжатии |

# Установка OmegaZip и 7-Zip

Краткий **happy path** для десктопа. Полный список форматов: [FORMATS.md](FORMATS.md). Ассоциации файлов и ПКМ: [FILE-ASSOCIATIONS.md](FILE-ASSOCIATIONS.md), [CONTEXT-MENU.md](CONTEXT-MENU.md).

## OmegaZip

### Разработка из репозитория

1. [Rust](https://rustup.rs/) (stable), [Node.js](https://nodejs.org/) для фронта Tauri 2.  
2. Клонировать репозиторий, в корне:
   ```bash
   cargo build --release -p omegazip
   npm install
   npm run dev
   ```
   Сборка установочного пакета приложения:
   ```bash
   npm run build
   ```
   На macOS для подстановки сервисов в `.app` может использоваться `npm run build:macos` (см. `package.json`).

### Конечный пользователь

Установите готовый артефакт из **релиза** вашего дистрибутива (формат зависит от CI: MSI/AppImage/DMG/DEB и т.д.). Конкретные ссылки не зашиваются в документ — смотрите страницу релизов проекта.

**Зависимости:** десктопное приложение — self-contained bundle Tauri (отдельно ставить Rust пользователю не нужно). Для расширенных форматов см. раздел про **7-Zip** ниже.

## 7-Zip / p7zip (FMT-03)

**Стратегия v1.1:** OmegaZip **не встраивает** бинарник 7-Zip в репозиторий. Используется **внешняя установка** и **автообнаружение**:

- Поиск `7z`, `7zz`, `7za` в `PATH`;
- на **Windows** — дополнительно типичные пути `Program Files\7-Zip\` и `Program Files (x86)\7-Zip\`;
- на **macOS** — дополнительно `/opt/homebrew/bin`, `/usr/local/bin` (Homebrew).

После установки перезапустите OmegaZip (или терминал для `omegazip`), чтобы обновился `PATH`.

### Как установить по ОС

- **Windows:** установщик с [7-zip.org](https://www.7-zip.org/). При установке в стандартную папку OmegaZip часто находит `7z.exe` даже без правки PATH.  
- **macOS:** `brew install p7zip` (обычно появляется `7zz`) или установка 7-Zip с сайта и добавление каталога с бинарником в PATH.  
- **Linux:** например `p7zip-full` / `p7zip-plugins` (Debian/Ubuntu, Fedora, Arch — см. подсказку в приложении или вывод `omegazip deps`).

### Проверка

```bash
omegazip deps
```

В GUI статус 7-Zip показывается в баннере при старте.

**OmegaZip Android:** внешний 7-Zip на устройстве не подключается; см. [FORMATS.md](FORMATS.md).

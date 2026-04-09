# OmegaZip Android

Отдельная **мобильная** сборка на базе того же Rust-ядра (`omegazip`) и Tauri 2. Имя приложения и пакет: **OmegaZip Android** (`app.omegazip.android`). Интерфейс: каталог **`ui-android/`** (не смешивается с десктопным `ui/`).

## Чем отличается от ПК

- Нет вызова внешнего **7-Zip** → **нет** распаковки RAR / нативного `.7z` / ISO / MSI и т.п. (только форматы из Rust: ZIP, tar\*, zst, CAB, `.oz`).
- Нет **rclone** (команды возвращают «не поддерживается»).
- Подпись и публикация в Google Play — настройте сами в Android Studio.

## Требования

- [Android Studio](https://developer.android.com/studio) или только SDK + NDK
- Переменная **`ANDROID_HOME`** (путь к SDK), например `~/Library/Android/sdk`
- Rust + `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`
- Установленный **`cargo tauri`** (Tauri CLI v2):  
  `cargo install tauri-cli --version "^2"`

## Первоначальная генерация Gradle-проекта

Каталог **`src-tauri/gen/android`** создаётся командой **init** (один раз, при наличии SDK):

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"   # свой путь
export PATH="$HOME/.cargo/bin:$PATH"
cd /path/to/OmegaZip
cargo tauri android init
```

Если `gen/android` уже есть, init обычно не нужен повторно.

## Сборка и запуск

```bash
export ANDROID_HOME=...
export PATH="$HOME/.cargo/bin:$PATH"
cd /path/to/OmegaZip

# Отладка на эмуляторе/устройстве
npm run android:dev

# APK/AAB (релиз)
npm run android:build
```

Открыть проект в Android Studio:  
`npm run android:open` или `cargo tauri android dev --open`.

## Конфигурация

| Файл | Назначение |
|------|------------|
| `src-tauri/tauri.android.conf.json` | Merge: имя **OmegaZip Android**, `app.omegazip.android`, `frontendDist` → `ui-android` |
| `src-tauri/capabilities/android.json` | Разрешения для мобильной сборки |
| `ui-android/index.html` | Веб-UI с пометкой Android |

Десктопная сборка по-прежнему читает **`tauri.conf.json`** и **`ui/`**; Android-патч подмешивается при сборке под Android.

## Устранение проблем

- **`ANDROID_HOME not set`** — задайте путь к SDK и перезапустите терминал.
- Ошибки **линковки NDK** — в Android Studio: SDK Manager → установите **NDK** и **CMake**.
- После смены зависимостей Rust иногда нужно: `cargo clean` в `src-tauri` и пересборка.

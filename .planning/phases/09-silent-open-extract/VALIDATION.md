# Validation — Phase 9 (GAP-01)

**Requirements:** GAP-01

## Чеклист

- [x] Код: один архив → ветка silent + доки (ручной двойной щелчок — на целевой ОС)
- [x] Запуск без аргументов → окно видимо (логика не трогает пустой argv)
- [x] Ошибка/пароль → показ окна + события (код)
- [x] Чекбокс + `gui-prefs.json` + `OMEGAZIP_NO_SILENT_EXTRACT`
- [x] `cargo test` + `src-tauri` `cargo check`
- [x] `docs/SILENT-EXTRACT.md`, ссылки README / FILE-ASSOCIATIONS

## Команды

```bash
cargo test
cd src-tauri && cargo check
```

## Ручной тест

Двойной щелчок по тестовому архиву в проводнике/Finder на доступной ОС.

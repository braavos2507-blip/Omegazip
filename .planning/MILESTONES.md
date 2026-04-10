# Milestones

Журнал закрытых майлстоунов. Активная работа — в [ROADMAP.md](ROADMAP.md) и [STATE.md](STATE.md).

---

## v1.0 Baseline / Android / Docs (Shipped: 2026-03-29)

**Phases completed:** 3 phases, 3 plans (01-01, 02-01, 03-01)

**Archives**

- [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md)
- [milestones/v1.0-REQUIREMENTS.md](milestones/v1.0-REQUIREMENTS.md)
- Индекс: [MILESTONE-V1.0.md](MILESTONE-V1.0.md)

**Key accomplishments**

- Ядро, CLI, безопасная распаковка, интеграционные тесты; GUI Tauri; CI с `cargo test` и `cargo check` в `src-tauri`.
- OmegaZip Android: отдельный `ui-android/`, `ANDROID_BUILD.md`, сообщения об ограничениях без 7-Zip/rclone.
- Согласованность README / `docs/` / `Правки.md` с кодом; закрытие v1.0 зафиксировано в GSD-фазах 01–03.

**Pre-flight (GSD)**

- Отдельный `v1.0-MILESTONE-AUDIT.md` не создавался; ретро-закрытие после фактической поставки.

**Git tag:** `v1.0` (плановый майлстоун; версия приложения в bundle может отличаться).

---

## v1.2 — ТЗ: GAP (Shipped: 2026-04-09)

**Phases completed:** GAP-01…05 (фазы 9–13); фаза 8 (п.8 ТЗ) **не ведётся** — в спецификации пункта не было.

**Index**

- [MILESTONE-V1.2.md](MILESTONE-V1.2.md)

**Key accomplishments**

- Тихое открытие / распаковка без главного окна (GAP-01); ПКМ Win/Linux с общей логикой и CI (GAP-02); `docs/VERSUS-TOP5.md` (GAP-03); сквозной сценарий установки в `docs/INSTALL.md` (GAP-04); трей отложен, задокументирован (GAP-05).
- CI: `cargo test`, `cargo clippy` на `omegazip`, проверка bash-инсталлеров и AST PowerShell для контекстного меню.

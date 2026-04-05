# Roadmap: OmegaZip

## Overview

**v1.0** — **закрыт и заархивирован** — [MILESTONES.md](MILESTONES.md), [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md), [MILESTONE-V1.0.md](MILESTONE-V1.0.md).  
**v1.1** — ТЗ от 2026-03-30: интеграция в ОС (двойной щелчок, ПКМ), умолчания путей и имён, матрица форматов «как топ-5», лёгкая установка, фон + выбор .oz/.zip, умные пресеты (**в работе** до закрытия фазы 8 / TBD-01).

---

<details>
<summary>✅ v1.0 (фазы 1–3) — SHIPPED 2026-03-29 — архив</summary>

### Phases

- [x] **Phase 1: Baseline & quality gates** — `phases/01-baseline-quality-gates/`
- [x] **Phase 2: Android delivery** — `phases/02-android-delivery/`
- [x] **Phase 3: Hardening & docs** — `phases/03-hardening-docs/`

Полный снимок дорожной карты v1.0: [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md).

#### Phase 1: Baseline & quality gates

**Goal**: Репозиторий имеет полный GSD-контур; ядро и GUI соответствуют требованиям CORE/GUI/QA baseline.  
**Depends on**: Nothing  
**Requirements**: CORE-01 — CORE-03, GUI-01 — GUI-02, QA-01  
**Plans**: [01-01-PLAN.md](phases/01-baseline-quality-gates/01-01-PLAN.md) (ретро, completed)

#### Phase 2: Android delivery

**Goal**: Предсказуемая сборка **OmegaZip Android**.  
**Depends on**: Phase 1 (желательно)  
**Requirements**: AND-01 — AND-03  
**Plans**: [02-01-PLAN.md](phases/02-android-delivery/02-01-PLAN.md) (ретро, completed)

#### Phase 3: Hardening & docs

**Goal**: Документация и edge-case.  
**Depends on**: Phase 2  
**Requirements**: QA-02  
**Plans**: [03-01-PLAN.md](phases/03-hardening-docs/03-01-PLAN.md) (ретро, completed)

</details>

---

## Milestone v1.1 — Интеграция в ОС и умолчания (ТЗ)

### Phases

- [x] **Phase 4: Ассоциации и двойной щелчок** — `phases/04-associations-double-click/`
- [x] **Phase 5: ПКМ и фон** — `phases/05-context-menu-background/`
- [x] **Phase 6: Форматы и установка** — `phases/06-formats-install/`
- [x] **Phase 7: Умные пресеты** — `phases/07-smart-presets/`
- [ ] **Phase 8 (gate): ТЗ п.8** — TBD-01 — закрыть после уточнения требований

### Phase Details (v1.1)

#### Phase 4: Ассоциации и двойной щелчок

**Goal**: Поддерживаемые архивы открываются OmegaZip по двойному щелчку и распаковываются в каталог архива по умолчанию.  
**Depends on**: Желательно завершение критичных задач v1.0 или параллельно по ресурсу  
**Requirements**: SHELL-01, PATH-01  
**Success Criteria**: Документированная установка ассоциаций; ручная проверка на 2 ОС из {Windows, macOS, Linux}.

#### Phase 5: ПКМ и фон

**Goal**: Контекстное меню «Сжать в .oz / .zip»; приложение не требует главного окна для базового сценария.  
**Depends on**: Phase 4 (желательно)  
**Requirements**: SHELL-02, SHELL-03, PATH-02, PATH-03  
**Success Criteria**: Сценарий из ТЗ воспроизводится без открытия полного окна (или с минимальным прогрессом).

#### Phase 6: Форматы и установка

**Goal**: Матрица форматов + план паритета; лёгкий установочный сценарий.  
**Depends on**: —  
**Requirements**: FMT-01 — FMT-03, INST-01, INST-02  
**Success Criteria**: `FORMATS.md` или раздел в README; список известных ограничений; один «happy path» установки.

#### Phase 7: Умные пресеты

**Goal**: По умолчанию выбор режима сжатия по типу файла с балансом скорости/размера без потерь для lossless.  
**Depends on**: Ядро `.oz` без изменений контракта  
**Requirements**: SMART-01, SMART-02  
**Success Criteria**: Таблица эвристик в коде или конфиге; тесты на 2–3 класса файлов.

#### Phase 8: Уточнение п.8 ТЗ

**Goal**: Заполнить пропуск в ТЗ (TBD-01).  
**Depends on**: заказчик  
**Requirements**: TBD-01  

---

*Roadmap updated: 2026-03-29 — `/gsd:close-milestone`: v1.0 свёрнут в `<details>` + архив `milestones/`; активен v1.1 (фаза 8 — gate).*

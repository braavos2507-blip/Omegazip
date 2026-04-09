# Roadmap: OmegaZip

## Overview

**v1.0** — **закрыт и заархивирован** — [MILESTONES.md](MILESTONES.md), [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md), [MILESTONE-V1.0.md](MILESTONE-V1.0.md).  
**v1.1** — планы **4–7** доставлены; остаётся **фаза 8** (**TBD-01**). Честные пробелы ТЗ вынесены в **v1.2**.  
**v1.2** — **строгое ТЗ** (`GAP-01`…`GAP-05`): тихая распаковка при открытии, ПКМ «из коробки» Win/Linux, честный vs топ-5, установка, опционально трей.

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

*Итог v1.1:* ассоциации, доки ПКМ, матрица форматов, INSTALL, smart preset — **есть**; формулировки «без окна / как топ-5 из коробки» — **v1.2 (GAP-*)**. См. [REQUIREMENTS.md](REQUIREMENTS.md).

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

## Milestone v1.2 — ТЗ: строгая доводка (только GAP)

**Goal:** Закрыть расхождения между **дословным ТЗ** и текущим продуктом, не переделывая уже сданные PATH/SMART/базовую документацию.

### Phases

- [x] **Phase 9: Тихое открытие архива** — GAP-01 — `phases/09-silent-open-extract/` ([09-01-SUMMARY.md](phases/09-silent-open-extract/09-01-SUMMARY.md))
- [ ] **Phase 10: ПКМ Win/Linux «из коробки»** — GAP-02 (implementation complete, manual QA pending; см. `phases/10-pkm-win-linux/10-01-SUMMARY.md`)
- [ ] **Phase 11: Честный паритет форматов** — GAP-03 (таблица vs топ-5 + решение по одному нативному пробелу или явный non-goals)
- [ ] **Phase 12: Установка end-to-end** — GAP-04 (README/INSTALL, 7-Zip, первый «внешний» формат)
- [ ] **Phase 13 (опционально): Трей / фон** — GAP-05

Папки фаз создаются при **`/gsd:plan-phase 9`** … **`13`**.

### Phase Details (v1.2)

#### Phase 9: Тихое открытие архива

**Goal:** Один архив из ОС → распаковка рядом, **без** обязательного показа главного окна Tauri.  
**Depends on:** —  
**Requirements:** GAP-01  
**Success Criteria:** Сценарий воспроизводится на ≥1 ОС из {Windows, macOS, Linux}; ошибки/пароль — определённое поведение в доке.

#### Phase 10: ПКМ Win/Linux «из коробки»

**Goal:** Типовой пользователь получает пункты «Сжать в .oz/.zip» без ручной правки реестра (Windows) и с ясным шагом на Linux.  
**Depends on:** Phase 9 (желательно параллельно)  
**Requirements:** GAP-02  
**Success Criteria:** Обновлённый `CONTEXT-MENU.md` + артефакт установки или инсталляторный хук (по выбранной платформе в плане).

#### Phase 11: Честный паритет форматов

**Goal:** Документ vs топ-5 с колонками Pass/Partial/N/A; явные non-goals; при необходимости один маленький нативный инкремент.  
**Depends on:** —  
**Requirements:** GAP-03  
**Success Criteria:** Новый или расширенный раздел в `docs/` + ссылка из README.

#### Phase 12: Установка end-to-end

**Goal:** Один сценарий «скачал → установил → сжал/распаковал» с учётом 7-Zip.  
**Depends on:** —  
**Requirements:** GAP-04  
**Success Criteria:** Приёмочный чеклист в `INSTALL.md` или `docs/`.

#### Phase 13 (опционально): Трей / фон

**Goal:** Поведение ближе к «архиватор в фоне».  
**Depends on:** GAP-01 желательно  
**Requirements:** GAP-05  
**Success Criteria:** Трей или эквивалент согласован с ограничениями Tauri на целевых ОС.

---

*Roadmap updated: 2026-03-29 — фаза 9 (GAP-01) выполнена; далее фаза 10.*

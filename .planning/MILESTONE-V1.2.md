# Майлстоун v1.2 — ТЗ: строгая доводка (GAP)

**Статус:** **закрыт по GAP** (2026-04-09) — все GAP-01…05 выполнены; **п.8 ТЗ** в объёме спецификации нет (TBD-01 снят).  
**Цель (достигнута):** пробелы ТЗ v1.2 закрыты документами и кодом.

## Требования

- Живой чеклист: [REQUIREMENTS.md](REQUIREMENTS.md) — раздел **v1.2**, идентификаторы **GAP-01** … **GAP-05**.

## Фазы

| Фаза | Фокус | REQ |
|------|--------|-----|
| **9** | Тихое открытие / распаковка без главного окна | GAP-01 ✅ |
| **10** | ПКМ Win/Linux «из коробки» | GAP-02 ✅ (CI + `test-context-menu-logic.sh`) |
| **11** | Честный vs топ-5 | GAP-03 ✅ |
| **12** | Установка end-to-end | GAP-04 ✅ |
| **13** | Трей (опционально) | GAP-05 ✅ (отложено, док.) |

Дорожная карта: [ROADMAP.md](ROADMAP.md). Фаза **9** выполнена: [09-01-SUMMARY.md](phases/09-silent-open-extract/09-01-SUMMARY.md).  
По фазе **10:** [10-01-SUMMARY.md](phases/10-pkm-win-linux/10-01-SUMMARY.md).  
Фаза **11** (GAP-03): [11-01-SUMMARY.md](phases/11-format-parity/11-01-SUMMARY.md), документ [docs/VERSUS-TOP5.md](../docs/VERSUS-TOP5.md).  
Фаза **12** (GAP-04): [12-01-SUMMARY.md](phases/12-install-e2e/12-01-SUMMARY.md), сквозной сценарий в [docs/INSTALL.md](../docs/INSTALL.md).  
Фаза **13** (GAP-05): [13-01-SUMMARY.md](phases/13-tray-optional/13-01-SUMMARY.md), [docs/GAP05-TRAY-DEFERRED.md](../docs/GAP05-TRAY-DEFERRED.md).

## Параллельно

- **Фаза 8** — не ведётся: пункта 8 в ТЗ нет (см. [ROADMAP.md](ROADMAP.md)).

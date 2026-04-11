# Go-to-market: конкурентные заявления и доказательства

Этот документ — не блокер кода, а **маркетинг и аудит**: что можно честно говорить пользователям и где лежат цифры.

## Сжатие vs 7-Zip / ZIP

| Заявление | Факт в продукте | Где смотреть цифры / код |
|-----------|-----------------|---------------------------|
| Сильное сжатие в `.oz` (solid, ultra + XZ-9) | Пресеты `max` / `ultra`, multi-segment solid (`--solid-block-mi`) | `src/codec_backend.rs`, `ROADMAP_DOMINANCE.md` §1.1 |
| Паритет «как 7-Zip по формату LZMA2» | Не заявляем: свой контейнер `.oz`, не `.7z` | Сравнение размеров — свои бенчи |
| Сравнение с ZIP на корпусах | Скрипты measure / отчёты | `tests/manual-files/results-auto/OZ-ZIP-7Z-LATEST.md`, `npm run measure:competitive`, `docs/VERSUS-TOP5.md` |

**Формулировка для сайта:** «На наших корпусах .oz даёт лучший размер, чем deflate-ZIP; для максимума используйте пресет Ultra (solid + XZ-9). Это не тот же алгоритм, что LZMA2 в 7-Zip.»

## Бэкапы vs Borg / Restic

| Заявление | Факт |
|-----------|------|
| Chunk store + снапшоты + дедуп между снапшотами | Да: `repo init` / `backup` / `restore` / `list`, `repo prune` |
| Облако SFTP/S3 из коробки | Нативного SDK нет; **`repo rclone-sync`** + [rclone](https://rclone.org/) (S3, SFTP, …); либо `repo push` в смонтированный каталог |

**Формулировка:** «Локальный репозиторий с дедупом; облако — синхронизация папки репо.»

## Надёжность vs WinRAR-стиль

| Заявление | Факт |
|-----------|------|
| Recovery + CRC по блокам в `.oz` v2 | Да |
| Восстановление при частичной порче | В пределах parity Reed–Solomon |

**Формулировка:** «Запись восстановления в архиве .oz; ограничения — число parity-блоков на полосу.»

## Обязательные артефакты перед публичной рекламой цифр

- Свежий `npm run measure:everything-local` и при необходимости `measure:oz-repo-corpora` с `CORPUS_EXTRA`.
- Обновить `tests/manual-files/results-auto/OZ-ZIP-7Z-LATEST.md` или аналог под ваш корпус.
- Публичный release gate: `docs/RELEASE-READINESS.md` в режиме **GO-PUBLIC** (см. `docs/DIST-01-MACOS-SIGNING.md`).

# OmegaZip .oz Format Specification (v2)

Краткая спецификация формата архива OmegaZip для совместимости и аудита.

## Магические байты и заголовок

- **Magic:** `OMEGAZIP\x02` (9 байт) — версия 2.
- **Flags (1 byte):**
  - bit 0 (0x1): encrypted
  - bit 1 (0x2): reserved (ранее salt present)
  - bit 2 (0x4): chunked dedup
  - bit 3 (0x8): recovery (Reed-Solomon)
- Если encrypted: **salt** 16 байт.
- **Manifest:** JSON в UTF-8, до первого `\n}\n` (или аналог конца объекта).

## Manifest (JSON)

- **Legacy:** JSON-массив записей `ManifestEntry` (как раньше).
- **Solid multi-block:** JSON-объект `{ "files": [ ... ], "solid_segments": [ { hash_hex, algo, len }, ... ] }` — порядок сегментов совпадает с порядком физических блоков solid-потока; логические смещения файлов (`solid.offset` / `length`) отсчитываются в объединённом распакованном потоке.
- `version`: 2 (в коде версия в magic, не в JSON)
- `files`: массив `{ path, size, [chunks], [solid] }`
  - **Обычный режим:** `chunks`: `[{ hash_hex, algo, len }]`
  - **Solid:** `solid`: `{ stream_id, offset, length }`
- При recovery в v2 после манифеста в бинарном потоке: **num_blocks** (u32), затем блоки с CRC и опционально recovery-секция.

## Блоки данных

- Порядок: как в манифесте (chunks по порядку или один solid-поток).
- **Без recovery:** каждый блок: hash(32) + algo(1) + len(4) + data.
- **С recovery:** блок: hash(32) + algo(1) + len(4) + **crc32(4)** + data; после всех блоков — recovery-секция: num_stripes (u32), для каждой полосы: max_len (u32), 2 parity-блока (Reed-Solomon, 16 data + 2 parity).

## Шифрование

- Ключ: Argon2id(salt, password) → 32 байта.
- Payload (манифест + блоки, опционально recovery): ChaCha20-Poly1305 (nonce 12 байт, передаётся/хранится по соглашению).

## Алгоритмы (algo byte)

- 0: stored
- 1: zstd
- 2: lz4
- 3: xz (ultra, level 9)
- 4: huffman (text)

## Репозиторий (repo)

- **Корень:** `chunks/`, `snapshots/`.
- **Чанки:** `chunks/<first2>/<next2>/<hashhex>` — содержимое чанка.
- **Снапшоты:** `snapshots/snapshot_<id>.json` — `{ id, files: [ { path, chunks: [ { hash_hex, algo, len } ] } ] }`.

---

*Документ актуален для OmegaZip 0.3.0.*

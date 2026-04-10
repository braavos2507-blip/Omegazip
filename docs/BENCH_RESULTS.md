# OmegaZip workflow benchmark

- Generated: 2026-04-10
- Binary: `/Users/renat/01Project/OmegaZip/target/release/omegazip`
- Mode: **real-only (samples from TEST_DIR)**
- Test dir source: `/Users/renat/01Project/OmegaZip/tests/manual-files/downloads`
- Work dir: `/tmp/omegazip-full-bench`

## Format selection (install-context-menu `pick_ext_auto`)

По умолчанию **`.oz`** (текст, разметка, PDF/EPUB/Office-XML, неизвестные суффиксы).

**`.zip`** только для чёрного списка: архивы (`zip`, `7z`, `rar`, `gz`, …), изображения, видео, аудио, шрифты, типичные бинарники (`exe`, `so`, `dmg`, …), образы дисков, `sqlite`.

## Results

Compress: **Input** = исходный файл/папка; **Archive** = размер архива; **Ratio** = архив / вход.

Decompress: **Archive** = размер файла архива; **Extracted** = сумма размеров извлечённых файлов; **Ratio** = extracted / archive.

| Case | Input MB | Archive MB | Extracted MB | Ratio | Real(s) | User(s) | Sys(s) | Status |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| real_sample_docx_docx | 0.1363 | 0.1343 | — | 0.9856 | 0.25 | 0.00 | 0.00 | ok |
| real_sample_docx_docx_decompress | 0.1343 | — | 0.1363 | 1.0146 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_jpg_jpg | 0.6856 | 0.6832 | — | 0.9965 | 0.02 | 0.01 | 0.00 | ok |
| real_sample_jpg_jpg_decompress | 0.6832 | — | 0.6856 | 1.0035 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_mp4_mp4 | 2.7163 | 2.7018 | — | 0.9947 | 0.06 | 0.06 | 0.00 | ok |
| real_sample_mp4_mp4_decompress | 2.7018 | — | 2.7163 | 1.0054 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_pdf_pdf | 4.8939 | 2.5254 | — | 0.5160 | 0.57 | 0.55 | 0.01 | ok |
| real_sample_pdf_pdf_decompress | 2.5254 | — | 4.8939 | 1.9379 | 0.13 | 0.12 | 0.00 | ok |
| real_sample_png_png | 0.2142 | 0.2127 | — | 0.9933 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_png_png_decompress | 0.2127 | — | 0.2142 | 1.0067 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_zip_zip | 2.8412 | 2.8275 | — | 0.9952 | 0.06 | 0.05 | 0.00 | ok |
| real_sample_zip_zip_decompress | 2.8275 | — | 2.8412 | 1.0048 | 0.00 | 0.00 | 0.00 | ok |

CSV: `/tmp/omegazip-full-bench/results.csv`

Источник данных: `tests/manual-files/results-auto/BENCH-WORKFLOW-LATEST.md` (автопрогон из `scripts/run-full-local-qa.sh`).

## Competitive bench (ZIP vs .oz vs 7z)

См. полный отчёт: `tests/manual-files/results-auto/OZ-ZIP-7Z-LATEST.md`.

- **versioned corpus** (jQuery/Bootstrap multi-version): `.oz` меньше ZIP на **~18.0%**.
- **mixed corpus** (docs+code+assets): после режима `--preset competitive` и оптимизации solid-decompress `.oz` меньше ZIP на **~4.9%**.

Вывод: преимущество `.oz` наиболее выражено на versioned/dedup-сценариях; на mixed-наборах также достигнут выигрыш по размеру, при этом скорость компрессии `.oz` остаётся выше ZIP/7z.

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
| real_sample_jpg_jpg | 0.0360 | 0.0332 | — | 0.9220 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_jpg_jpg_decompress | 0.0332 | — | 0.0360 | 1.0846 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_mp4_mp4 | 2.7163 | 2.7018 | — | 0.9947 | 0.04 | 0.04 | 0.00 | ok |
| real_sample_mp4_mp4_decompress | 2.7018 | — | 2.7163 | 1.0054 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_pdf_pdf | 0.0126 | 0.0119 | — | 0.9398 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_pdf_pdf_decompress | 0.0119 | — | 0.0126 | 1.0640 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_png_png | 0.2142 | 0.2127 | — | 0.9933 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_png_png_decompress | 0.2127 | — | 0.2142 | 1.0067 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_zip_zip | 2.7018 | 2.7023 | — | 1.0002 | 0.04 | 0.03 | 0.00 | ok |
| real_sample_zip_zip_decompress | 2.7023 | — | 2.7018 | 0.9998 | 0.00 | 0.00 | 0.00 | ok |

CSV: `/tmp/omegazip-full-bench/results.csv`

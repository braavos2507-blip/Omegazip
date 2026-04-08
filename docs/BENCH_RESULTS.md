# OmegaZip workflow benchmark

- Generated: 2026-04-08
- Binary: `/Applications/OmegaZip.app/Contents/MacOS/omegazip`
- Mode: **real-only (samples from TEST_DIR)**
- Test dir source: `/Users/renat/Documents/Project/Для тестов`
- Work dir: `/tmp/omegazip-full-bench`

## Format selection (install-context-menu `pick_ext_auto`)

| Input | Archive |
|---|---|
| Folder | `.oz` |
| `txt`, `md`, `csv`, `json`, code sources (`rs`, `js`, `ts`, …), markup/config (`xml`, `yaml`, …) | `.oz` |
| Other extensions (images, video, `pdf`, `zip`, `docx`, …) | `.zip` |

## Results

| Case | Input MB | Output MB | Ratio | Real(s) | User(s) | Sys(s) | Status |
|---|---:|---:|---:|---:|---:|---:|---|
| real_sample_docx_docx | 0.14 | 0.13 | 0.9535 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_docx_docx_decompress | 0.13 | 0.00 | 0 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_jpg_jpg | 0.69 | 0.68 | 0.9965 | 0.01 | 0.01 | 0.00 | ok |
| real_sample_jpg_jpg_decompress | 0.68 | 0.00 | 0 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_mp4_mp4 | 2.72 | 2.70 | 0.9947 | 0.04 | 0.04 | 0.00 | ok |
| real_sample_mp4_mp4_decompress | 2.70 | 0.00 | 0 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_pdf_pdf | 4.89 | 2.84 | 0.5806 | 0.07 | 0.07 | 0.00 | ok |
| real_sample_pdf_pdf_decompress | 2.84 | 0.00 | 0 | 0.01 | 0.00 | 0.00 | ok |
| real_sample_png_png | 0.21 | 0.21 | 0.9933 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_png_png_decompress | 0.21 | 0.00 | 0 | 0.00 | 0.00 | 0.00 | ok |
| real_sample_zip_zip | 2.84 | 2.83 | 0.9952 | 0.04 | 0.04 | 0.00 | ok |
| real_sample_zip_zip_decompress | 2.83 | 0.00 | 0 | 0.00 | 0.00 | 0.00 | ok |

CSV: `/tmp/omegazip-full-bench/results.csv`

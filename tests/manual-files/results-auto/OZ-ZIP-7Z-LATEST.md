# Competitive bench: ZIP vs .oz vs 7z

- Generated: 2026-04-10 16:45:10
- OmegaZip binary: `/Users/renat/01Project/OmegaZip/target/release/omegazip`
- 7z binary: `/opt/homebrew/bin/7zz`

| Corpus | Files | Input MB | ZIP MB | .oz MB | 7z MB | .oz vs ZIP | .oz vs 7z | ZIP c s | .oz c s | 7z c s | ZIP d s | .oz d s | 7z d s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| versioned | 3083 | 99.17 | 58.31 | 52.72 | 23.98 | +9.6% | -119.9% | 2.286 | 0.528 | 6.518 | 0.461 | 0.278 | 0.689 |
| mixed | 6756 | 40.25 | 33.89 | 32.22 | 23.65 | +4.9% | -36.2% | 1.017 | 0.348 | 2.371 | 0.589 | 0.516 | 0.898 |

`+.oz vs ZIP` означает, что .oz меньше. Отрицательное значение — .oz больше.
`c` = compress, `d` = decompress.

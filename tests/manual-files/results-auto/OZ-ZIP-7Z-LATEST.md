# Competitive bench: ZIP vs .oz vs 7z

- Generated: 2026-04-15 18:04:41
- OmegaZip binary: `/Users/renat/01Project/OmegaZip/target/release/omegazip`
- 7z binary: `/opt/homebrew/bin/7zz`

| Corpus | Files | Input MB | ZIP MB | .oz MB | 7z MB | .oz vs ZIP | .oz vs 7z | ZIP c s | .oz c s | 7z c s | ZIP d s | .oz d s | 7z d s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| versioned | 3083 | 99.17 | 58.31 | 52.72 | 23.98 | +9.6% | -119.9% | 2.011 | 1.697 | 7.034 | 0.577 | 0.309 | 0.721 |
| mixed | 6756 | 40.25 | 33.89 | 32.22 | 23.65 | +4.9% | -36.2% | 1.610 | 0.669 | 2.534 | 0.610 | 0.469 | 0.923 |

`+.oz vs ZIP` означает, что .oz меньше. Отрицательное значение — .oz больше.
`c` = compress, `d` = decompress.

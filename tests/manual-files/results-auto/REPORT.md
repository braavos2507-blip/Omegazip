# Manual Files Auto-Test Report

- Binary: `/var/folders/1_/m403l98566bg591g5ntty2200000gn/T/cursor-sandbox-cache/c42e5b2d9569bef721be0c4826a6de59/cargo-target/debug/omegazip`
- Source dir: `/Users/renat/01Project/OmegaZip/tests/manual-files/downloads`

## Cases

| Case | Source | Src bytes | Archive | Archive bytes | Compress | Extract | Match |
|---|---|---:|---|---:|---|---|---|
| file-auto-oz | text_ascii_rust_gitignore.txt | 684 | text_ascii_rust_gitignore.auto.oz | 648 | OK (0.018s) | OK (0.005s) | YES |
| file-auto-oz | doc_pdf_dummy.pdf | 13264 | doc_pdf_dummy.auto.oz | 12464 | OK (0.01s) | OK (0.004s) | YES |
| file-auto-oz | book_epub_alice.epub | 188707 | book_epub_alice.auto.oz | 186048 | OK (0.018s) | OK (0.004s) | YES |
| file-auto-oz | image_jpg_example.jpg | 37763 | image_jpg_example.auto.oz | 35114 | OK (0.008s) | OK (0.004s) | YES |
| file-auto-oz | image_png_transparency.png | 224566 | image_png_transparency.auto.oz | 224777 | OK (0.018s) | OK (0.005s) | YES |
| file-auto-oz | video_mp4_sample5s.mp4 | 2848208 | video_mp4_sample5s.auto.oz | 2848394 | OK (0.143s) | OK (0.006s) | YES |
| file-auto-oz | archive_zip_hellogitworld.zip | 2833028 | archive_zip_hellogitworld.auto.oz | 2833221 | OK (0.149s) | OK (0.005s) | YES |
| file-zip | text_ascii_rust_gitignore.txt | 684 | text_ascii_rust_gitignore.zip | 542 | OK (0.005s) | OK (0.004s) | YES |
| file-zip | image_png_transparency.png | 224566 | image_png_transparency.zip | 223078 | OK (0.029s) | OK (0.008s) | YES |
| folder-auto-oz | large-folder/ | 34178496 | large-folder.auto.oz | 2849750 | OK (1.542s) | OK (0.015s) | YES |

## Context Preset Stress

| Label | Context | MB threshold | Archive bytes | Status |
|---|---|---:|---:|---|
| ctx_max | max | - | 34007411 | OK |
| ctx_ultra | ultra | - | 2841285 | OK |
| ctx_auto_low_threshold | auto | 20 | 2849750 | OK |

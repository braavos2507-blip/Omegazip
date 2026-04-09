# Manual Test Assets

Prepared dataset for local manual/automated smoke checks of OmegaZip formats and context-workflow behavior.

## Source files

Directory: `tests/manual-files/downloads/`

- `text_ascii_rust_gitignore.txt`
- `doc_pdf_dummy.pdf`
- `book_epub_alice.epub`
- `image_jpg_example.jpg`
- `image_png_transparency.png`
- `video_mp4_sample5s.mp4`
- `archive_zip_hellogitworld.zip`
- `hellogitworld-master/` (unzipped sample project)

## Generated outputs

Ignored by git:

- `workflow-run/`
- `results/`
- `results-auto/extract/`
- `large-folder/`

Kept in git:

- `results-auto/REPORT.md`
- `results-auto/report.json`

## Notes

- Names are explicit by format to avoid collisions when producing `*.oz` archives by stem.
- `large-folder/` is generated locally for threshold tests and is intentionally excluded from version control.

//! Solid с несколькими сырыми блоками (manifest wrapped + roundtrip).

use std::fs;

use omegazip::{compress_to_path_with_options, decompress_to_path, CompressOptions, Preset};

#[test]
fn solid_two_raw_blocks_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    // ~700 KiB each → два сегмента при --solid-block-mi 1
    fs::write(src.join("a.bin"), vec![0xabu8; 700 * 1024]).unwrap();
    fs::write(src.join("b.bin"), vec![0x5du8; 700 * 1024]).unwrap();
    let oz = dir.path().join("out.oz");
    let opts = CompressOptions {
        solid: true,
        preset: Some(Preset::Max),
        solid_block_size_bytes: Some(1024 * 1024),
        ..CompressOptions::default()
    };
    compress_to_path_with_options(&src, &oz, opts).expect("compress");
    let out = dir.path().join("extracted");
    fs::create_dir_all(&out).unwrap();
    let n = decompress_to_path(&oz, &out).expect("decompress");
    assert_eq!(n, 2);
    assert_eq!(
        fs::read(out.join("a.bin")).unwrap(),
        vec![0xabu8; 700 * 1024]
    );
    assert_eq!(
        fs::read(out.join("b.bin")).unwrap(),
        vec![0x5du8; 700 * 1024]
    );
}

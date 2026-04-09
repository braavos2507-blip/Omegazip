//! Интеграционные проверки совместимости: ZIP, tar.gz, безопасные пути.

use std::fs;
use std::io::Write;
use std::path::Path;

use omegazip::compat;

#[test]
fn zip_compress_and_extract_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("in");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("hello.txt"), b"omega").unwrap();

    let zip_path = dir.path().join("pack.zip");
    let n = compat::compress_to_zip(&src, &zip_path).expect("compress_to_zip");
    assert!(n >= 1);

    let out = dir.path().join("out");
    let extracted = compat::extract_foreign(&zip_path, &out).expect("extract");
    assert!(extracted >= 1);
    assert_eq!(fs::read_to_string(out.join("hello.txt")).unwrap(), "omega");

    let names = compat::list_foreign(&zip_path).expect("list");
    assert!(names.iter().any(|s| s.ends_with("hello.txt")));
}

#[test]
fn tar_gz_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("data");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("a.bin"), &[1u8, 2, 3]).unwrap();

    let tgz = dir.path().join("bundle.tar.gz");
    {
        let enc = flate2::write::GzEncoder::new(fs::File::create(&tgz).unwrap(), flate2::Compression::default());
        let mut ar = tar::Builder::new(enc);
        ar.append_path_with_name(&src.join("a.bin"), "a.bin").unwrap();
        ar.finish().unwrap();
    }

    let out = dir.path().join("untar");
    let n = compat::extract_foreign(&tgz, &out).expect("extract tgz");
    assert_eq!(n, 1);
    assert_eq!(fs::read(out.join("a.bin")).unwrap(), vec![1u8, 2, 3]);
}

#[test]
fn zstd_single_file_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let zst = dir.path().join("payload.zst");
    let enc = zstd::encode_all(&b"omega-zst"[..], 3).expect("zstd encode");
    fs::write(&zst, enc).unwrap();
    let out = dir.path().join("out");
    let n = compat::extract_foreign(&zst, &out).expect("zst extract");
    assert_eq!(n, 1);
    assert_eq!(
        fs::read_to_string(out.join("payload")).unwrap(),
        "omega-zst"
    );
}

#[test]
fn tar_gz_compress_and_extract() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("note.txt"), b"tg").unwrap();
    let tgz = dir.path().join("bundle.tgz");
    let n = compat::compress_to_tar_gz(&src, &tgz).expect("compress tar.gz");
    assert!(n >= 1);
    let out = dir.path().join("extracted");
    let m = compat::extract_foreign(&tgz, &out).expect("extract tgz");
    assert!(m >= 1);
    assert_eq!(fs::read_to_string(out.join("note.txt")).unwrap(), "tg");
}

#[test]
fn zip_rejects_path_traversal() {
    let dir = tempfile::tempdir().expect("tempdir");
    let zip_path = dir.path().join("evil.zip");
    build_zip_with_raw_path(&zip_path, "../escape.txt", b"x");

    let out = dir.path().join("safe_out");
    let err = compat::extract_foreign(&zip_path, &out).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("traversal") || msg.contains("absolute"),
        "unexpected error: {msg}"
    );
}

/// Минимальный ZIP (stored, без шифрования) с произвольным именем в центральном каталоге.
fn build_zip_with_raw_path(zip_path: &Path, file_name: &str, body: &[u8]) {
    use std::io::Seek;

    let mut f = fs::File::create(zip_path).unwrap();
    let name_bytes = file_name.as_bytes();
    let crc = crc32fast::hash(body);
    let local_header_off = f.stream_position().unwrap();

    // Local file header
    f.write_all(&0x04034b50u32.to_le_bytes()).unwrap(); // signature
    f.write_all(&20u16.to_le_bytes()).unwrap(); // version needed
    f.write_all(&0u16.to_le_bytes()).unwrap(); // flags
    f.write_all(&0u16.to_le_bytes()).unwrap(); // method = store
    f.write_all(&0u16.to_le_bytes()).unwrap(); // time
    f.write_all(&0u16.to_le_bytes()).unwrap(); // date
    f.write_all(&crc.to_le_bytes()).unwrap();
    f.write_all(&(body.len() as u32).to_le_bytes()).unwrap(); // compressed
    f.write_all(&(body.len() as u32).to_le_bytes()).unwrap(); // uncompressed
    f.write_all(&(name_bytes.len() as u16).to_le_bytes()).unwrap();
    f.write_all(&0u16.to_le_bytes()).unwrap(); // extra len
    f.write_all(name_bytes).unwrap();
    f.write_all(body).unwrap();

    let central_off = f.stream_position().unwrap();
    // Central directory header
    f.write_all(&0x02014b50u32.to_le_bytes()).unwrap();
    f.write_all(&20u16.to_le_bytes()).unwrap(); // version made by
    f.write_all(&20u16.to_le_bytes()).unwrap(); // version needed
    f.write_all(&0u16.to_le_bytes()).unwrap(); // flags
    f.write_all(&0u16.to_le_bytes()).unwrap(); // method
    f.write_all(&0u16.to_le_bytes()).unwrap(); // mod time
    f.write_all(&0u16.to_le_bytes()).unwrap(); // mod date
    f.write_all(&crc.to_le_bytes()).unwrap();
    f.write_all(&(body.len() as u32).to_le_bytes()).unwrap();
    f.write_all(&(body.len() as u32).to_le_bytes()).unwrap();
    f.write_all(&(name_bytes.len() as u16).to_le_bytes()).unwrap(); // name len
    f.write_all(&0u16.to_le_bytes()).unwrap(); // extra len
    f.write_all(&0u16.to_le_bytes()).unwrap(); // comment len
    f.write_all(&0u16.to_le_bytes()).unwrap(); // disk number start
    f.write_all(&0u16.to_le_bytes()).unwrap(); // internal attrs
    f.write_all(&0u32.to_le_bytes()).unwrap(); // external attrs
    f.write_all(&(local_header_off as u32).to_le_bytes()).unwrap();
    f.write_all(name_bytes).unwrap();

    let central_size = f.stream_position().unwrap() - central_off;
    let central_entries = 1u16;

    // EOCD
    f.write_all(&0x06054b50u32.to_le_bytes()).unwrap();
    f.write_all(&0u16.to_le_bytes()).unwrap();
    f.write_all(&0u16.to_le_bytes()).unwrap();
    f.write_all(&central_entries.to_le_bytes()).unwrap();
    f.write_all(&central_entries.to_le_bytes()).unwrap();
    f.write_all(&(central_size as u32).to_le_bytes()).unwrap();
    f.write_all(&(central_off as u32).to_le_bytes()).unwrap();
    f.write_all(&0u16.to_le_bytes()).unwrap(); // comment len
}

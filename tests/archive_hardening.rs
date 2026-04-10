//! Надёжность: шифрованный .oz, обрезанный .oz (чеклист MEASURABLE-QUALITY D5, D7).

use std::fs;
use omegazip::{
    compress_to_path_with_options, decompress_to_path, decompress_to_path_with_password,
    CompressOptions, Preset,
};

#[test]
fn encrypted_oz_requires_password() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("note.txt");
    fs::write(&src, b"confidential").unwrap();
    let oz = dir.path().join("locked.oz");
    let opts = CompressOptions {
        password: Some("correct-horse".to_string()),
        preset: Some(Preset::Fast),
        ..CompressOptions::default()
    };
    compress_to_path_with_options(&src, &oz, opts).expect("compress encrypted");

    let out = dir.path().join("out_nopw");
    fs::create_dir_all(&out).unwrap();
    let err = decompress_to_path(&oz, &out).expect_err("must require password");
    let msg = err.to_string();
    assert!(
        msg.contains("password") || msg.contains("Password") || msg.contains("Encrypted"),
        "unexpected: {msg}"
    );
}

#[test]
fn encrypted_oz_wrong_password_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("data.bin");
    fs::write(&src, vec![7u8; 2048]).unwrap();
    let oz = dir.path().join("enc.oz");
    let opts = CompressOptions {
        password: Some("secret".to_string()),
        preset: Some(Preset::Balanced),
        ..CompressOptions::default()
    };
    compress_to_path_with_options(&src, &oz, opts).expect("compress");

    let out = dir.path().join("out_wrong");
    fs::create_dir_all(&out).unwrap();
    let err = decompress_to_path_with_password(&oz, &out, Some("wrong-password"))
        .expect_err("wrong password must fail");
    let msg = err.to_string();
    assert!(
        !msg.is_empty(),
        "expected non-empty error"
    );
}

#[test]
fn truncated_oz_errors_cleanly() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("plain.txt");
    fs::write(&src, b"truncate me").unwrap();
    let oz = dir.path().join("full.oz");
    compress_to_path_with_options(&src, &oz, CompressOptions::default()).expect("compress");

    let mut bytes = fs::read(&oz).expect("read oz");
    let cut = (bytes.len() / 2).max(32);
    bytes.truncate(cut);
    let bad = dir.path().join("cut.oz");
    fs::write(&bad, bytes).expect("write truncated");

    let out = dir.path().join("out_trunc");
    fs::create_dir_all(&out).unwrap();
    let r = decompress_to_path(&bad, &out);
    assert!(r.is_err(), "truncated .oz must error, got {:?}", r.ok());
}

#[cfg(unix)]
#[test]
fn symlink_in_source_dir_is_skipped_not_followed() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("src");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("real.txt"), b"kept").unwrap();
    symlink("real.txt", root.join("via_link.txt")).expect("symlink");

    let oz = dir.path().join("sym.oz");
    compress_to_path_with_options(&root, &oz, CompressOptions::default()).expect("compress");
    let out = dir.path().join("ext");
    fs::create_dir_all(&out).unwrap();
    decompress_to_path(&oz, &out).expect("decompress");
    assert!(out.join("real.txt").is_file());
    assert!(
        !out.join("via_link.txt").exists(),
        "symlink must not be archived by default"
    );
}

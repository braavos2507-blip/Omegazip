//! Шифрование архива: Argon2 (ключ из пароля) + ChaCha20-Poly1305 по блокам.

use argon2::Argon2;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::ChaCha20Poly1305;
use rand::RngCore;
use rand::rngs::OsRng;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
const KEY_LEN: usize = 32;

/// Генерирует соль для KDF.
pub fn random_salt() -> [u8; SALT_LEN] {
    let mut s = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut s);
    s
}

/// Выводит ключ из пароля и соли (Argon2id).
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("argon2");
    key
}

/// Шифрует один блок (nonce генерируется случайно, дописывается в начало).
pub fn encrypt_block(key: &[u8; KEY_LEN], plaintext: &[u8]) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new_from_slice(key).expect("key len");
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt((&nonce).into(), plaintext)
        .expect("encrypt");
    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    out
}

/// Расшифровывает блок (первые 12 байт — nonce, далее ciphertext+tag).
pub fn decrypt_block(key: &[u8; KEY_LEN], payload: &[u8]) -> Result<Vec<u8>, String> {
    if payload.len() < NONCE_LEN + TAG_LEN {
        return Err("payload too short".into());
    }
    let cipher = ChaCha20Poly1305::new_from_slice(key).expect("key len");
    let (nonce, ct) = payload.split_at(NONCE_LEN);
    cipher
        .decrypt(nonce.into(), ct)
        .map_err(|e| e.to_string())
}

//! Определение типа по magic bytes (таблица сигнатур).

const SIGNATURES: &[(&[u8], &str)] = &[
    // Видео/мультимедиа (уже сжаты кодеком; для .oz — режим Store)
    (b"\x1a\x45\xdf\xa3", "video/webm"), // EBML: WebM / Matroska (.mkv)
    (b"ID3", "audio/mpeg"),             // MP3 с ID3-тегом
    (b"OggS", "application/ogg"),
    (b"%PDF", "application/pdf"),
    (b"\x89PNG\r\n\x1a\n", "image/png"),
    (b"GIF87a", "image/gif"),
    (b"GIF89a", "image/gif"),
    (b"\xff\xd8\xff", "image/jpeg"),
    (b"PK\x03\x04", "application/zip"),
    (b"PK\x05\x06", "application/zip"),
    (b"\x1f\x8b", "application/gzip"),
    (b"<!DOCTYPE", "text/html"),
    (b"<!doctype", "text/html"),
    (b"<html", "text/html"),
    (b"<?xml", "application/xml"),
    (b"\xef\xbb\xbf", "text/plain"), // UTF-8 BOM
];

pub fn mime_from_magic(data: &[u8]) -> Option<String> {
    for (sig, mime) in SIGNATURES {
        if data.len() >= sig.len() && data[..sig.len()] == **sig {
            return Some((*mime).to_string());
        }
    }
    iso_bmff_like(data)
}

/// MP4/MOV (ftyp) и AVI (RIFF…AVI ) без ложных срабатываний на случайные бинарники.
fn iso_bmff_like(data: &[u8]) -> Option<String> {
    if data.len() >= 12 && &data[4..8] == b"ftyp" {
        return Some("video/mp4".to_string());
    }
    if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"AVI " {
        return Some("video/x-msvideo".to_string());
    }
    None
}
